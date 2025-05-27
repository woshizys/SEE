use crate::http::router::axum_router;
use crate::lru::lru_cache::LRUCache;
use config::Config;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

mod router;
mod data;
mod common;
mod dtos;

#[derive(Debug, Clone)]
struct Tools {
    lru_cache: Arc<RwLock<LRUCache<String, Vec<u8>>>>,
}

pub async fn axum_serve(config: Config) {
    let port = config.get::<u16>("server_port").unwrap();
    let cache_mode = config.get::<String>("cache_mode").unwrap();
    let cache_size = config.get::<usize>("cache_size").unwrap();

    let lru_cache = match cache_mode.as_str() {
        "item" | "default" => {
            LRUCache::new(NonZeroUsize::new(cache_size).unwrap())
        }
        "capacity" => {
            LRUCache::storage(NonZeroUsize::new(cache_size).unwrap())
        }
        "unlimited" => {
            LRUCache::unbounded()
        }
        _ => {
            LRUCache::new(NonZeroUsize::new(cache_size).unwrap())
        }
    };
    let lru_cache: Arc<RwLock<LRUCache<String, Vec<u8>>>> = Arc::new(RwLock::new(lru_cache));

    let axum_app = axum_router(Tools { lru_cache: lru_cache.clone() });
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, axum_app).await.unwrap();
}