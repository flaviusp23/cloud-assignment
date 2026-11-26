// Bookmarks API: a tiny CRUD service on Axum + Postgres.
//
// Config comes from environment variables, never from the code:
//   DB_URL  Postgres connection string (required)
//   PORT    port to listen on (default 8080)

use std::env;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tower_http::cors::CorsLayer;

// A bookmark as stored in the database and sent back as JSON.
#[derive(Serialize, FromRow)]
struct Bookmark {
    id: i64,
    title: String,
    url: String,
}

// What the client sends to create or update a bookmark.
#[derive(Deserialize)]
struct NewBookmark {
    title: String,
    url: String,
}

// Any database error becomes a "500 Internal Server Error".
fn db_error<E>(_: E) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

// GET /api/bookmarks: all bookmarks, newest first.
async fn list(State(db): State<PgPool>) -> Result<Json<Vec<Bookmark>>, StatusCode> {
    let rows: Vec<Bookmark> = sqlx::query_as("SELECT id, title, url FROM bookmarks ORDER BY id DESC")
        .fetch_all(&db)
        .await
        .map_err(db_error)?;
    Ok(Json(rows))
}

// POST /api/bookmarks: add one.
async fn create(
    State(db): State<PgPool>,
    Json(new): Json<NewBookmark>,
) -> Result<(StatusCode, Json<Bookmark>), StatusCode> {
    let row: Bookmark = sqlx::query_as(
        "INSERT INTO bookmarks (title, url) VALUES ($1, $2) RETURNING id, title, url",
    )
    .bind(new.title)
    .bind(new.url)
    .fetch_one(&db)
    .await
    .map_err(db_error)?;
    Ok((StatusCode::CREATED, Json(row)))
}

// PUT /api/bookmarks/{id}: change one. 404 if it does not exist.
async fn update(
    State(db): State<PgPool>,
    Path(id): Path<i64>,
    Json(new): Json<NewBookmark>,
) -> Result<Json<Bookmark>, StatusCode> {
    let row: Option<Bookmark> = sqlx::query_as(
        "UPDATE bookmarks SET title = $1, url = $2 WHERE id = $3 RETURNING id, title, url",
    )
    .bind(new.title)
    .bind(new.url)
    .bind(id)
    .fetch_optional(&db)
    .await
    .map_err(db_error)?;
    row.map(Json).ok_or(StatusCode::NOT_FOUND)
}

// DELETE /api/bookmarks/{id}: remove one. 404 if it does not exist.
async fn remove(State(db): State<PgPool>, Path(id): Path<i64>) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM bookmarks WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await
        .map_err(db_error)?;
    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

// GET /health: lets us check the service is up.
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    // The database URL contains a password, so it comes from the environment.
    let db_url = env::var("DB_URL").expect("DB_URL is required");
    let port = env::var("PORT").unwrap_or("8080".to_string());

    // Connect to Postgres and make sure the table exists.
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

    // Map each URL to its handler. The database pool is shared with all of them.
    // CORS lets the frontend (another URL) call this API from the browser.
    let app = Router::new()
        .route("/api/bookmarks", get(list).post(create))
        .route("/api/bookmarks/{id}", put(update).delete(remove))
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    println!("API listening on port {port}");
    axum::serve(listener, app).await.unwrap();
}
