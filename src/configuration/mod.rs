#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("load config: {0}")]
    Load(String),

    #[error("decode config: {0}")]
    Decode(String),
}

pub async fn load<T>() -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    let enable = std::env::var("NACOS_CONFIG_ENABLED")
        .unwrap_or_default()
        .parse::<bool>()
        .unwrap_or_default();
    if enable {
        load_nacos().await
    } else {
        load_file().await
    }
}

pub async fn load_nacos<T>() -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    let nacos = crate::nacos::API::new_from_env().expect("failed to create nacos client");
    let config = nacos.load().await.map_err(|e| Error::Load(e.to_string()))?;
    tracing::info!("CONFIG: {}", config);
    Ok(decode(&config)?)
}

pub async fn load_file<T>() -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    unimplemented!("load config from file")
}

pub fn decode<T>(value: &str) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_yaml::from_str(value).map_err(|e| Error::Decode(e.to_string()))?)
}
