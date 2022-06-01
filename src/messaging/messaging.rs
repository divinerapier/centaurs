#[async_trait::async_trait]
pub trait Consumer {
    type Output;
    type Error;

    async fn poll(&self) -> Result<Option<Self::Output>, Self::Error>;

    async fn commit(&self, message: Self::Output) -> Result<(), Self::Error>;

    fn subscribe(
        &self,
        topics: &[&str],
    ) -> Result<SubscribeGuard<Self::Output, Self::Error>, Self::Error>;

    fn unsubscribe(&self);

    fn auto_commit(&self) -> bool;
}

pub struct SubscribeGuard<'a, O, E> {
    pub consumer: &'a dyn Consumer<Output = O, Error = E>,
}

impl<'a, O, E> Drop for SubscribeGuard<'a, O, E> {
    fn drop(&mut self) {
        tracing::info!("unsubscribe");
        self.consumer.unsubscribe();
    }
}
