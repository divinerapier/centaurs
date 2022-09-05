use std::{borrow::Cow, sync::Arc};

use nacos_rust_client::client::{
    config_client::ConfigKey,
    naming_client::{Instance as NNI, NamingClient},
    ConfigClient, HostInfo,
};

use crate::servicediscovery::Instance;

pub mod servicediscovery;

pub struct API {
    config: ConfigClient,
    naming: Arc<NamingClient>,
    tenant: String,
    data_id: String,
    group: String,
}

#[derive(derive_builder::Builder)]
pub struct Config {
    server: String,
    port: u16,
    tenant: String,
    data_id: String,
    group: String,
    server_name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("find environment variable: {0}. {1}")]
    VarNotFound(Cow<'static, str>, std::env::VarError),

    #[error("parse nacos port: {0}")]
    ParsePort(String),

    #[error("nacos is disabled")]
    Disable,
}

impl Config {
    pub fn from_env() -> Result<Config, Error> {
        std::env::var("NACOS_CONFIG_ENABLED")
            .unwrap_or_default()
            .parse::<bool>()
            .ok()
            .filter(|x| *x)
            .ok_or(Error::Disable)?;

        let server = Self::read_env("NACOS_SERVER")?;
        let port = Self::read_env("NACOS_PORT")
            .map(|v| v.parse::<u16>().unwrap_or(8848))
            .unwrap_or(8848);
        let tenant = Self::read_env("NACOS_NAMESPACE").unwrap_or_default();
        let data_id = Self::read_env("NACOS_DATA_ID")?;
        let group = Self::read_env("NACOS_GROUP").unwrap_or_else(|_| "DEFAULT_GROUP".to_string());
        let server_name = Self::read_env("KUBERNETES_SERVICE_NAME")?;
        Ok(Config {
            server,
            port,
            tenant,
            data_id,
            group,
            server_name,
        })
    }
    fn read_env(env: &'static str) -> Result<String, Error> {
        std::env::var(env).map_err(|e| Error::VarNotFound(Cow::Borrowed(env), e))
    }

    pub fn into_api(self) -> API {
        API::new(self)
    }
}

impl API {
    pub fn new(cfg: Config) -> Self {
        let server = cfg.server;
        let port = cfg.port;
        let host = HostInfo::new(&server, port as u32);
        let naming = NamingClient::new(host.clone(), cfg.server_name);
        let config = ConfigClient::new(host, cfg.tenant.clone());

        API {
            naming,
            config,
            tenant: cfg.tenant,
            data_id: cfg.data_id,
            group: cfg.group,
        }
    }
}

impl API {
    /// new_from_env 通过环境变量初始化
    pub fn new_from_env() -> Result<API, Error> {
        Ok(Config::from_env()?.into_api())
    }

    pub async fn load(&self) -> Result<String, Box<dyn std::error::Error + Send>> {
        let key = ConfigKey::new(&self.data_id, &self.group, &self.tenant);
        Ok(self.config.get_config(&key).await?)
    }

    /// register 注册服务到 nacos 服务发现中心
    pub fn register(&self, instance: &Instance) {
        self.naming.register(NNI::new(
            &instance.ip,
            instance.port,
            &instance.service_name,
            &self.group,
            "",
            &self.tenant,
            None,
        ));
    }

    /// deregister 从 nacos 服务发现中心注销服务
    pub fn deregister(&self, instance: &Instance) {
        self.naming.unregister(NNI::new(
            &instance.ip,
            instance.port,
            &instance.service_name,
            &self.group,
            "",
            &self.tenant,
            None,
        ));
    }
}

/// register_factory 注册初始化函数到 servicediscovery::FACTORIES
pub fn register_factory() -> Result<(), crate::servicediscovery::Error> {
    crate::servicediscovery::register_factory("nacos".to_string(), || {
        let nacos = API::new_from_env()?;
        Ok(Box::new(nacos))
    })
}
