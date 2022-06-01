use std::time::Duration;

use super::failover::Failover;

#[async_trait::async_trait]
pub trait Processor {
    type Item;
    type Error;
    type Output;

    async fn process(&self, item: &Self::Item) -> Result<Self::Output, Self::Error>;
}

pub struct RetriableProcessor<P, I> {
    processor: P,
    iterable: I,
}

impl<P, I> RetriableProcessor<P, I> {
    pub fn new(processor: P, iterable: I) -> RetriableProcessor<P, I> {
        RetriableProcessor {
            processor,
            iterable,
        }
    }
}

#[derive(Debug)]
pub enum RetriableProcessorError<E> {
    Retry(E),
}

#[async_trait::async_trait]
impl<P, I> Processor for RetriableProcessor<P, I>
where
    P: Processor + Send + Sync,
    P::Item: Send + Sync,
    P::Output: Send + Sync,
    P::Error: Send + Sync,

    I: Iterator<Item = Duration> + Send + Sync + Clone,
{
    type Item = P::Item;
    type Output = P::Output;
    type Error = RetriableProcessorError<P::Error>;

    async fn process(&self, item: &Self::Item) -> Result<Self::Output, Self::Error> {
        let mut iterable = self.iterable.clone();
        let mut index = 0;
        loop {
            index += 1;
            let result = match self.processor.process(item).await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    Result::<Self::Output, Self::Error>::Err(RetriableProcessorError::Retry(e))
                }
            };
            if let Some(duration) = iterable.next() {
                tracing::warn!("retry for {} times", index);
                tokio::time::sleep(duration).await;
                continue;
            }
            return result;
        }
    }
}

#[derive(Debug)]
pub enum FailoverError<PE, FE> {
    Process(PE),
    Failover(PE, FE),
}

pub struct FailoverProcessor<P, F> {
    processor: P,
    failover: F,
}

impl<P, F> FailoverProcessor<P, F> {
    pub fn new(processor: P, failover: F) -> FailoverProcessor<P, F> {
        FailoverProcessor {
            processor,
            failover,
        }
    }
}

#[async_trait::async_trait]
impl<P, F> Processor for FailoverProcessor<P, F>
where
    P: Processor + Send + Sync,
    P::Item: Send + Sync,
    P::Output: Send + Sync,
    P::Error: Send + Sync,

    F: Failover<Item = P::Item, InputError = P::Error> + Send + Sync,
{
    type Item = P::Item;
    type Output = P::Output;
    type Error = FailoverError<P::Error, F::Error>;

    async fn process(&self, item: &Self::Item) -> Result<Self::Output, Self::Error> {
        match self.processor.process(item).await {
            Ok(v) => Ok(v),
            Err(e) => match self.failover.failover(item, &e).await {
                Ok(_) => Err(FailoverError::Process(e)),
                Err(fe) => Err(FailoverError::Failover(e, fe)),
            },
        }
    }
}
