#[async_trait::async_trait]
pub trait Failover {
    type Item;
    type InputError;
    type Error;

    async fn failover(&self, item: &Self::Item, ie: &Self::InputError) -> Result<(), Self::Error>;
}
