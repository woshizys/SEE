use lru::http::axum_serve;
use lru::load_from_file;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let config = load_from_file(PathBuf::from("config/config.toml"));
    axum_serve(config).await;
}
