use std::path::PathBuf;

pub mod lru;
pub mod http;

pub fn load_from_file(path: PathBuf) -> config::Config {
    config::Config::builder()
        .add_source(config::File::with_name(path.to_str().unwrap()))
        .build()
        .unwrap()
}