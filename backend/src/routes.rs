use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use rusqlite::params;
use serde_json::Value;
use std::sync::Arc;

use crate::models::*;
use crate::AppState;
use crate::db::get_schema_description;

pub async fn query_nl(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
    let schema = get_schema_description();

    let llm_result = match state.llm.generate_sql(&schema, &req.question).await {
        Ok(r) => r,
        Err(e) => {
            return Json(QueryResponse {
                question: req.question,
                sql: String::new(),
                columns: vec![],
                rows: vec![],
                error: Some(format!("LLM error: {}", e)),
                provider: None,
            });
        }
    };

    let provider = Some(llm_result.provider);
    let db = state.db.lock().unwrap();
    match execute_select(&db, &llm_result.sql) {
        Ok((columns, rows)) => Json(QueryResponse {
            question: req.question,
            sql: llm_result.sql,
            columns,
            rows,
            error: None,
            provider,
        }),
        Err(e) => Json(QueryResponse {
            question: req.question,
            sql: llm_result.sql,
            columns: vec![],
            rows: vec![],
            error: Some(format!("SQL error: {}", e)),
            provider,
        }),
    }
}

fn execute_select(
    db: &rusqlite::Connection,
    sql: &str,
) -> Result<(Vec<String>, Vec<Vec<Value>>), rusqlite::Error> {
    let mut stmt = db.prepare(sql)?;
    let col_count = stmt.column_count();
    let columns: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let rows = stmt.query_map([], |row| {
        let mut vals = Vec::with_capacity(col_count);
        for i in 0..col_count {
            let val: Value = match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                Ok(rusqlite::types::ValueRef::Integer(n)) => Value::Number(n.into()),
                Ok(rusqlite::types::ValueRef::Real(f)) => {
                    serde_json::Number::from_f64(f)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                }
                Ok(rusqlite::types::ValueRef::Text(s)) => {
                    Value::String(String::from_utf8_lossy(s).to_string())
                }
                Ok(rusqlite::types::ValueRef::Blob(b)) => {
                    Value::String(format!("<blob {} bytes>", b.len()))
                }
                Err(_) => Value::Null,
            };
            vals.push(val);
        }
        Ok(vals)
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }

    Ok((columns, result))
}

const BASE_TABLES: &[&str] = &["airships", "flights", "incidents"];

pub async fn list_tables() -> Json<Vec<&'static str>> {
    Json(BASE_TABLES.to_vec())
}

pub async fn table_data(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if !BASE_TABLES.contains(&name.as_str()) {
        return (
            StatusCode::NOT_FOUND,
            Json(QueryResponse {
                question: format!("Tabel: {}", name),
                sql: String::new(),
                columns: vec![],
                rows: vec![],
                error: Some(format!("Onbekende tabel: {}", name)),
                provider: None,
            }),
        );
    }

    let sql = format!("SELECT * FROM {}", name);
    let db = state.db.lock().unwrap();
    match execute_select(&db, &sql) {
        Ok((columns, rows)) => (
            StatusCode::OK,
            Json(QueryResponse {
                question: format!("Tabel: {}", name),
                sql,
                columns,
                rows,
                error: None,
                provider: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(QueryResponse {
                question: format!("Tabel: {}", name),
                sql,
                columns: vec![],
                rows: vec![],
                error: Some(format!("SQL error: {}", e)),
                provider: None,
            }),
        ),
    }
}

pub async fn execute_direct(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DirectSqlRequest>,
) -> Json<QueryResponse> {
    let trimmed = req.sql.trim().to_uppercase();
    if !trimmed.starts_with("SELECT") {
        return Json(QueryResponse {
            question: req.question,
            sql: req.sql,
            columns: vec![],
            rows: vec![],
            error: Some("Alleen SELECT queries zijn toegestaan".to_string()),
            provider: None,
        });
    }

    let db = state.db.lock().unwrap();
    match execute_select(&db, &req.sql) {
        Ok((columns, rows)) => Json(QueryResponse {
            question: req.question,
            sql: req.sql,
            columns,
            rows,
            error: None,
            provider: None,
        }),
        Err(e) => Json(QueryResponse {
            question: req.question,
            sql: req.sql,
            columns: vec![],
            rows: vec![],
            error: Some(format!("SQL error: {}", e)),
            provider: None,
        }),
    }
}

pub async fn get_schema() -> Json<SchemaResponse> {
    Json(SchemaResponse {
        schema: get_schema_description(),
    })
}

// === CRUD for saved queries ===

fn read_saved_query(row: &rusqlite::Row) -> rusqlite::Result<SavedQuery> {
    Ok(SavedQuery {
        id: Some(row.get(0)?),
        question: row.get(1)?,
        sql_query: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        color: row.get(5)?,
    })
}

const SELECT_SAVED: &str = "SELECT id, question, sql_query, created_at, updated_at, color FROM saved_queries";

pub async fn list_saved_queries(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare(&format!("{} ORDER BY updated_at DESC", SELECT_SAVED))
        .unwrap();

    let queries: Vec<SavedQuery> = stmt
        .query_map([], |row| read_saved_query(row))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    Json(queries)
}

pub async fn create_saved_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SavedQuery>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO saved_queries (question, sql_query, color) VALUES (?1, ?2, ?3)",
        params![req.question, req.sql_query, req.color],
    )
    .unwrap();

    let id = db.last_insert_rowid();
    let mut stmt = db
        .prepare(&format!("{} WHERE id = ?1", SELECT_SAVED))
        .unwrap();

    let query = stmt.query_row(params![id], |row| read_saved_query(row)).unwrap();
    (StatusCode::CREATED, Json(query))
}

pub async fn update_saved_query(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<SavedQuery>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let rows = db
        .execute(
            "UPDATE saved_queries SET question = ?1, sql_query = ?2, color = ?3, updated_at = datetime('now') WHERE id = ?4",
            params![req.question, req.sql_query, req.color, id],
        )
        .unwrap();

    if rows == 0 {
        return (StatusCode::NOT_FOUND, Json(None));
    }

    let mut stmt = db
        .prepare(&format!("{} WHERE id = ?1", SELECT_SAVED))
        .unwrap();

    let query = stmt.query_row(params![id], |row| read_saved_query(row)).unwrap();
    (StatusCode::OK, Json(Some(query)))
}

pub async fn delete_saved_query(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let rows = db
        .execute("DELETE FROM saved_queries WHERE id = ?1", params![id])
        .unwrap();

    if rows == 0 {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::NO_CONTENT
    }
}
