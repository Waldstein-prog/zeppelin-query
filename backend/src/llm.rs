use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct LlmResult {
    pub sql: String,
    pub provider: String,   // "local" or "groq"
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate_sql(&self, schema: &str, question: &str) -> Result<LlmResult>;
}

fn build_system_prompt(schema: &str) -> String {
    format!(
        r#"You are a SQL query generator for a SQLite database. Given a natural language question, generate ONLY a valid SQLite SELECT query. Do NOT include any explanation, markdown formatting, or code blocks — return ONLY the raw SQL query.

Rules:
- Only generate SELECT statements. Never generate INSERT, UPDATE, DELETE, DROP, ALTER, or any other modifying statement.
- Use only the tables and columns defined in the schema below.
- Use proper JOIN syntax when combining tables.
- Use aliases for readability when appropriate.
- For boolean columns (like airship_survived), 1 = true/yes, 0 = false/no.
- Dates are stored as TEXT in YYYY-MM-DD format. Use SQLite date functions when needed.
- Return ONLY the SQL query, nothing else.

Schema:
{schema}"#
    )
}

// === OpenAI-compatible Provider (works with vLLM, Groq, etc.) ===

pub struct OpenAiCompatibleProvider {
    api_key: Option<String>,
    base_url: String,
    model: String,
    label: String,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: String, model: String, label: String) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(3))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            api_key: None,
            base_url,
            model,
            label,
            client,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
}

// === Fallback Provider: tries primary, falls back to secondary ===

pub struct FallbackProvider {
    primary: OpenAiCompatibleProvider,
    fallback: OpenAiCompatibleProvider,
}

impl FallbackProvider {
    pub fn new(primary: OpenAiCompatibleProvider, fallback: OpenAiCompatibleProvider) -> Self {
        Self { primary, fallback }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<ChatChoice>>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[async_trait]
impl LlmProvider for FallbackProvider {
    async fn generate_sql(&self, schema: &str, question: &str) -> Result<LlmResult> {
        match self.primary.generate_sql(schema, question).await {
            Ok(result) => Ok(result),
            Err(primary_err) => {
                eprintln!("Primary LLM ({}) failed: {}, trying fallback ({})",
                    self.primary.label, primary_err, self.fallback.label);
                self.fallback.generate_sql(schema, question).await
            }
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn generate_sql(&self, schema: &str, question: &str) -> Result<LlmResult> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: build_system_prompt(schema),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: question.to_string(),
                },
            ],
            temperature: 0.0,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let mut req = self.client.post(&url);
        if let Some(ref api_key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        let response = req.json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("LLM API error ({}): {}", status, error_text));
        }

        let chat_response: ChatResponse = response.json().await?;

        let sql = chat_response
            .choices
            .and_then(|c| c.into_iter().next())
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow!("No response from LLM"))?;

        // Clean up: remove markdown code blocks if present
        let sql = sql.trim();
        let sql = sql.strip_prefix("```sql").unwrap_or(sql);
        let sql = sql.strip_prefix("```").unwrap_or(sql);
        let sql = sql.strip_suffix("```").unwrap_or(sql);
        let sql = sql.trim().to_string();

        // Validate: only SELECT allowed
        let upper = sql.to_uppercase();
        let first_keyword = upper.trim_start().split_whitespace().next().unwrap_or("");
        if first_keyword != "SELECT" && first_keyword != "WITH" {
            return Err(anyhow!("Generated query is not a SELECT statement: {}", sql));
        }

        Ok(LlmResult {
            sql,
            provider: self.label.clone(),
        })
    }
}
