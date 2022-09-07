use std::borrow::Cow;

use nacos_rust_client::client::config_client::ConfigKey as CK;

#[cfg(feature = "nacos-configuration")]
pub mod configuration;

#[cfg(feature = "nacos-servicediscovery")]
pub mod servicediscovery;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("find environment variable: {0}. {1}")]
    EnvNotFound(Cow<'static, str>, std::env::VarError),

    #[error("parse nacos port: {0}")]
    ParsePort(String),

    #[error("nacos is disabled")]
    Disable,

    #[error("get config: {0:?}")]
    Config(Box<dyn std::fmt::Debug + Send>),
}

fn read_env(env: &'static str) -> Result<String, Error> {
    std::env::var(env).map_err(|e| Error::EnvNotFound(Cow::Borrowed(env), e))
}

pub struct Key {
    tenant: String,
    data_id: String,
    group: String,
}

impl Key {
    fn to_nacos(&self) -> CK {
        CK::new(&self.data_id, &self.group, &self.tenant)
    }
}

impl Key {
    pub fn from_env() -> Result<Key, Error> {
        std::env::var("NACOS_CONFIG_ENABLED")
            .unwrap_or_default()
            .parse::<bool>()
            .ok()
            .filter(|x| *x)
            .ok_or(Error::Disable)?;

        let tenant = read_env("NACOS_NAMESPACE").unwrap_or_default();
        let data_id = read_env("NACOS_DATA_ID")?;
        let group = read_env("NACOS_GROUP").unwrap_or_else(|_| "DEFAULT_GROUP".to_string());
        Ok(Key {
            tenant,
            data_id,
            group,
        })
    }
}

pub struct Nacos {
    server: String,
    port: u32,
}

impl Nacos {
    pub fn from_env() -> Result<Nacos, Error> {
        std::env::var("NACOS_CONFIG_ENABLED")
            .unwrap_or_default()
            .parse::<bool>()
            .ok()
            .filter(|x| *x)
            .ok_or(Error::Disable)?;

        let server = read_env("NACOS_SERVER")?;
        let port = read_env("NACOS_PORT")
            .map(|v| v.parse::<u32>().unwrap_or(8848))
            .unwrap_or(8848);

        Ok(Nacos { server, port })
    }
}
