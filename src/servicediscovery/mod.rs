use std::sync::Arc;

#[async_trait::async_trait]
pub trait Registry {
    type Instance;

    async fn register(&self, instance: Arc<Self::Instance>);

    async fn unregister(&self, instance: Arc<Self::Instance>);
}

pub struct Discovery<T> {
    registry: Box<dyn Registry<Instance = T>>,
}

impl<T> Discovery<T> {
    pub async fn register(self, instance: Option<T>) -> Option<RegisterGuard<T>> {
        let registry = self.registry;
        let instance = Arc::new(instance?);
        registry.register(instance.clone()).await;
        Some(RegisterGuard { registry, instance })
    }
}

pub struct RegisterGuard<T> {
    pub(crate) registry: Box<dyn Registry<Instance = T>>,
    pub(crate) instance: Arc<T>,
}

impl<T> Drop for RegisterGuard<T> {
    fn drop(&mut self) {
        futures::executor::block_on(self.registry.unregister(self.instance.clone()));
    }
}
