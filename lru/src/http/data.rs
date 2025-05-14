use crate::http::Tools;
use crate::lru::cache::Cache;
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hasher};

#[derive(Clone, Serialize, Deserialize)]
pub struct DownloadReq {
    key: String,
}

pub async fn download(
    Extension(tools): Extension<Tools>,
    Json(req): Json<DownloadReq>,
) -> impl IntoResponse {
    let key = req.key;
    let mut lru_cache = tools.lru_cache.write().await;
    let res = lru_cache.get(&key);
    let disposition_val = format!("attachment; filename=\"{}\"", key);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        disposition_val.parse().unwrap(),
    );
    match res {
        Some(buf) => Ok((headers, Bytes::from(buf.to_vec()))),
        None => Err((StatusCode::NOT_FOUND, "Data not found")),
    }
}

pub async fn upload(
    Extension(tools): Extension<Tools>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut lru_cache = tools.lru_cache.write().await;
    if let Some(field) = multipart.next_field().await.unwrap() {
        let buf = field.bytes().await.unwrap();
        let buf = buf.to_vec();
        let mut hasher = DefaultHasher::new();
        hasher.write(&buf);
        let key = hasher.finish().to_string();
        lru_cache.put(key.clone(), buf);
        Ok((StatusCode::OK, key))
    } else {
        Err((StatusCode::BAD_REQUEST, "No data has uploaded yet"))
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hasher};

    #[test]
    fn test_hasher() {
        let data1 = b"123232354234523525235235645654632423543643574567657575";
        let data2 = b"123232354234523525235235645654632423543643574567657575";
        let data3 = b"adsafasdfsadfsswer2r3ew5353eaesfsdfg3rt6345";
        let data4 = b"sdasfas9d0fas8sf90asfasddfojidashgfdsa09u";
        let mut hasher = DefaultHasher::new();
        hasher.write(data1);
        println!("{}", hasher.finish());
        let mut hasher = DefaultHasher::new();
        hasher.write(data2);
        println!("{}", hasher.finish());
        let mut hasher = DefaultHasher::new();
        hasher.write(data3);
        println!("{}", hasher.finish());
        let mut hasher = DefaultHasher::new();
        hasher.write(data4);
        println!("{}", hasher.finish());
    }
}