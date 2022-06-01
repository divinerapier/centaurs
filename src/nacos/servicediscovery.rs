use std::sync::Arc;

use nacos_rust_client::client::{
    naming_client::{Instance, NamingClient},
    HostInfo,
};

use super::Nacos;

pub struct Registry(Arc<NamingClient>);

impl Nacos {
    pub fn registry<T: ToString>(&self, namespace: T) -> Registry {
        Registry(NamingClient::new(
            HostInfo::new(&self.server, self.port),
            namespace.to_string(),
        ))
    }
}

#[async_trait::async_trait]
impl crate::servicediscovery::Registry for Registry {
    type Instance = Instance;

    async fn register(&self, instance: Arc<Self::Instance>) {
        self.0.register(Instance::clone(&instance));
    }

    async fn deregister(&self, instance: Arc<Self::Instance>) {
        self.0.unregister(Instance::clone(&instance));
    }
}
