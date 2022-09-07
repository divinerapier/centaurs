use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("io: {0}")]
    IO(#[from] std::io::Error),

    #[error("other: {0}")]
    Other(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait Loader {
    type Key;

    async fn load(&self, key: Self::Key) -> Result<String, Error>;
}

pub struct FileLoader {}

pub struct PathKeyParser {}

#[async_trait::async_trait]
impl Loader for FileLoader {
    type Key = String;
    async fn load(&self, key: Self::Key) -> Result<String, Error> {
        let path = Path::new(&key);
        if !path.exists() {
            return Err(Error::NotFound(key.to_string()));
        }
        Ok(std::fs::read_to_string(path)?)
    }
}

