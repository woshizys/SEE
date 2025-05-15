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

    let api_router = Router::new()
        .route("/lru", get(download))
        .route("/lru", post(upload))
        .layer(Extension(tools))
        .layer(DefaultBodyLimit::disable())
        .layer(cors);

    Router::new().nest("/api", api_router)
}
