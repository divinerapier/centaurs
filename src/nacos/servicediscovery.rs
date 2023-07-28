use std::sync::Arc;

use nacos_rust_client::client::{
    naming_client::{Instance, NamingClient, QueryInstanceListParams},
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

    pub async fn query(&self, service_name: &str, group_name: &str) -> Vec<Arc<Instance>> {
        let client = NamingClient::new(HostInfo::new(&self.server, self.port), "".to_string());
        let instances = client
            .query_instances(QueryInstanceListParams::new_simple(
                service_name,
                group_name,
            ))
            .await
            .unwrap();

        instances
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

    async fn query(&self, service_name: &str, group_name: &str) -> Vec<Arc<Self::Instance>> {
        self.0
            .query_instances(QueryInstanceListParams::new_simple(
                service_name,
                group_name,
            ))
            .await
            .unwrap()
    }
}
