use crate::http::data::{download, upload};
use crate::http::Tools;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::{Extension, Router};
use tower_http::cors::{Any, CorsLayer};

pub fn axum_router(tools: Tools) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/download", get(download))
        .route("/upload", post(upload))
        .layer(Extension(tools))
        .layer(DefaultBodyLimit::disable())
        .layer(cors)
}