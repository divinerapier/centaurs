use std::sync::Arc;

use nacos_rust_client::client::{
    config_client::ConfigKey,
    naming_client::{Instance as NNI, NamingClient},
    ConfigClient, HostInfo,
};

use crate::servicediscovery::Instance;

pub struct API {
    config: ConfigClient,
    naming: Arc<NamingClient>,
    tenant: String,
    data_id: String,
    group: String,
}

impl Default for API {
    fn default() -> Self {
        let server = std::env::var("NACOS_SERVER").unwrap_or_else(|_| "nacos".to_string());
        let port = std::env::var("NACOS_PORT")
            .unwrap_or_else(|_| "8848".to_string())
            .parse::<u16>()
            .unwrap_or(8848);
        let host = HostInfo::parse(&format!("{server}:{port}"));
        let tenant = std::env::var("NACOS_NAMESPACE").unwrap_or_default();
        let data_id = std::env::var("NACOS_DATA_ID").unwrap_or_default();
        let group = std::env::var("NACOS_GROUP").unwrap_or_else(|_| "DEFAULT_GROUP".to_string());
        let server_name = std::env::var("NACOS_SERVER_NAME").unwrap();
        tracing::debug!("NACOS_SERVER: {}", server);
        tracing::debug!("NACOS_PORT: {}", port);
        tracing::debug!("NACOS_NAMESPACE: {}", tenant);
        tracing::debug!("NACOS_DATA_ID: {}", data_id);
        tracing::debug!("NACOS_GROUP: {}", group);
        tracing::debug!("NACOS_SERVER_NAME: {}", server_name);

        let naming = NamingClient::new(HostInfo::new(&server, port as u32), server_name);
        let config = ConfigClient::new(host, tenant.clone());

        API {
            naming,
            config,
            tenant,
            data_id,
            group,
        }
    }
}

impl API {
    /// new_from_env 通过环境变量初始化
    pub fn new_from_env() -> Option<API> {
        let enable = std::env::var("NACOS_CONFIG_ENABLED")
            .unwrap_or_default()
            .parse::<bool>()
            .unwrap_or_default();
        if !enable {
            return None;
        }
        Some(API::default())
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
pub fn register_factory() {
    crate::servicediscovery::register_factory(|| {
        let nacos = API::new_from_env()?;
        Some(Box::new(nacos))
    })
}
