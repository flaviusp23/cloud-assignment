// A tiny bookmarks API: Axum + Postgres.
// Config comes from environment variables: DB_URL (required, holds the password) and PORT.

use std::env;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tower_http::cors::CorsLayer;

#[derive(Serialize, FromRow)]
struct Bookmark {
    id: i64,
    title: String,
    url: String,
}

#[derive(Deserialize)]
struct NewBookmark {
    title: String,
    url: String,
}

// Any database error becomes a 500.
fn db_error<E>(_: E) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn list(State(db): State<PgPool>) -> Result<Json<Vec<Bookmark>>, StatusCode> {
    sqlx::query_as("SELECT id, title, url FROM bookmarks ORDER BY id DESC")
        .fetch_all(&db)
        .await
        .map(Json)
        .map_err(db_error)
}

async fn create(
    State(db): State<PgPool>,
    Json(new): Json<NewBookmark>,
) -> Result<(StatusCode, Json<Bookmark>), StatusCode> {
    let bookmark =
        sqlx::query_as("INSERT INTO bookmarks (title, url) VALUES ($1, $2) RETURNING id, title, url")
            .bind(new.title)
            .bind(new.url)
            .fetch_one(&db)
            .await
            .map_err(db_error)?;
    Ok((StatusCode::CREATED, Json(bookmark)))
}

async fn remove(State(db): State<PgPool>, Path(id): Path<i64>) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM bookmarks WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await
        .map_err(db_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let db_url = env::var("DB_URL").expect("DB_URL is required");
    let port = env::var("PORT").unwrap_or("8080".to_string());

    let db = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("cannot connect to the database");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            id BIGSERIAL PRIMARY KEY,
            title TEXT NOT NULL,
            url TEXT NOT NULL
        )",
    )
    .execute(&db)
    .await
    .expect("cannot create the table");

    let app = Router::new()
        .route("/api/bookmarks", get(list).post(create))
        .route("/api/bookmarks/{id}", delete(remove))
        .route("/health", get(health))
        .layer(CorsLayer::permissive()) // lets the frontend, on another URL, call the API
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
