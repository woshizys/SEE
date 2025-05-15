use lru::http::axum_serve;


#[tokio::main]
async fn main() {
    axum_serve(2345).await;
}
