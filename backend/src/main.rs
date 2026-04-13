mod db;
mod llm;
mod models;
mod routes;

use axum::{
    Router,
    routing::{get, post, put},
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub llm: Box<dyn llm::LlmProvider>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let llm_base_url = std::env::var("LLM_BASE_URL")
        .unwrap_or_else(|_| "http://10.50.198.160:11434/v1".to_string());
    let llm_model = std::env::var("LLM_MODEL")
        .unwrap_or_else(|_| "gemma4".to_string());

    let mut primary = llm::OpenAiCompatibleProvider::new(
        llm_base_url, llm_model, "local".to_string()
    );
    if let Ok(api_key) = std::env::var("LLM_API_KEY") {
        primary = primary.with_api_key(api_key);
    }

    let groq_api_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
    let fallback = llm::OpenAiCompatibleProvider::new(
        "https://api.groq.com/openai/v1".to_string(),
        "llama-3.3-70b-versatile".to_string(),
        "groq".to_string(),
    ).with_api_key(groq_api_key);

    let provider = llm::FallbackProvider::new(primary, fallback);

    let conn = Connection::open("zeppelin.db").expect("Failed to open database");
    db::init_db(&conn).expect("Failed to initialize database");
    db::seed_data(&conn).expect("Failed to seed data");

    let state = Arc::new(AppState {
        db: Mutex::new(conn),
        llm: Box::new(provider),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/query", post(routes::query_nl))
        .route("/api/schema", get(routes::get_schema))
        .route("/api/saved-queries", get(routes::list_saved_queries).post(routes::create_saved_query))
        .route("/api/execute", post(routes::execute_direct))
        .route("/api/tables", get(routes::list_tables))
        .route("/api/tables/:name", get(routes::table_data))
        .route("/api/saved-queries/:id", put(routes::update_saved_query).delete(routes::delete_saved_query))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    println!("Zeppelin Query backend running on http://localhost:3001");
    axum::serve(listener, app).await.unwrap();
}
