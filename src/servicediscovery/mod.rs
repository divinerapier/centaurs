use std::sync::Arc;

#[async_trait::async_trait]
pub trait Registry {
    type Instance;

    async fn register(&self, instance: Arc<Self::Instance>);

    async fn deregister(&self, instance: Arc<Self::Instance>);

    async fn query(&self, service_name: &str, group_name: &str) -> Vec<Arc<Self::Instance>>;
}

pub struct Discovery<T> {
    registry: Arc<dyn Registry<Instance = T>>,
}

impl<T> Discovery<T> {
    pub fn new<R>(registry: R) -> Discovery<T>
    where
        R: Registry<Instance = T> + 'static,
    {
        Discovery {
            registry: Arc::new(registry),
        }
    }

    pub async fn register(&self, instance: T) -> RegisterGuard<T> {
        let registry = self.registry.clone();
        let instance = Arc::new(instance);
        registry.register(instance.clone()).await;
        RegisterGuard { registry, instance }
    }

    pub async fn query(&self, service_name: &str, group_name: &str) -> Vec<Arc<T>> {
        let registry = self.registry.clone();
        registry.query(service_name, group_name).await
    }
}

pub struct RegisterGuard<T> {
    pub(crate) registry: Arc<dyn Registry<Instance = T>>,
    pub(crate) instance: Arc<T>,
}

impl<T> Drop for RegisterGuard<T> {
    fn drop(&mut self) {
        futures::executor::block_on(self.registry.deregister(self.instance.clone()));
    }
}
