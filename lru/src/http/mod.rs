use crate::http::router::axum_router;
use crate::lru::lru_cache::LRUCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

mod router;
mod data;

#[derive(Debug, Clone)]
struct Tools {
    lru_cache: Arc<RwLock<LRUCache<String, Vec<u8>>>>,
}

pub async fn axum_serve(port: u64) {
    let lru_cache: Arc<RwLock<LRUCache<String, Vec<u8>>>> = Arc::new(RwLock::new(LRUCache::new(NonZeroUsize::new(500).unwrap())));

    let axum_app = axum_router(Tools { lru_cache: lru_cache.clone() });
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, axum_app).await.unwrap();
}