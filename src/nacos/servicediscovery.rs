use std::sync::Arc;

use nacos_rust_client::client::{
    naming_client::{Instance, NamingClient},
    HostInfo,
};

use super::Nacos;

impl Nacos {
    pub fn registry(&self) -> Registry {
        Registry {
            inner: NamingClient::new(
                HostInfo::new(&self.server, self.port),
                "cfg.server_name".to_string(),
            ),
            group: "".to_string(),
            tenant: "".to_string(),
        }
    }
}

pub struct Registry {
    inner: Arc<NamingClient>,
    group: String,
    tenant: String,
}

#[async_trait::async_trait]
impl crate::servicediscovery::Registry for Registry {
    type Instance = Instance;

    async fn register(&self, instance: Arc<Self::Instance>) {
        self.inner.register(Instance::new(
            &instance.ip,
            instance.port,
            &instance.service_name,
            &self.group,
            "",
            &self.tenant,
            None,
        ));
    }

    async fn unregister(&self, instance: Arc<Self::Instance>) {
        self.inner.unregister(Instance::new(
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
