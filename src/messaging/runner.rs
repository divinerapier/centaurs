use std::fmt::Debug;

use futures::{select, Future, FutureExt};

pub struct Runner<C, P> {
    consumer: C,
    processor: P,
}

impl<C, P> Runner<C, P> {
    pub fn new(consumer: C, processor: P) -> Runner<C, P> {
        Runner {
            consumer,
            processor,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<C, P> {
    #[error("consumer: {0}")]
    Consumer(C),
    #[error("processor: {0}")]
    Processor(P),
}

impl<C, P> Runner<C, P>
where
    C: super::Consumer,
    C::Error: Debug,

    P: super::Processor<Item = C::Output>,
{
    pub async fn run<S: Future>(
        self,
        topics: &[&str],
        signal: S,
    ) -> Result<(), Error<C::Error, P::Error>> {
        let _guard = self
            .consumer
            .subscribe(topics)
            .map_err(|e| Error::<C::Error, P::Error>::Consumer(e))?;

        let mut signal = Box::pin(signal).fuse();

        loop {
            select! {
                _ = signal => {
                    tracing::warn!("Runner receive a signal. Quit!");
                    break Ok(());
                }
                default => {}
            };
            match self.consumer.poll().await {
                Ok(Some(message)) => match self.processor.process(&message).await {
                    Ok(_output) => {
                        self.consumer
                            .commit(message)
                            .await
                            .map_err(|e| Error::Consumer(e))?;
                    }
                    Err(e) => return Err(Error::Processor(e)),
                },
                Ok(None) => {}
                Err(e) => {
                    tracing::error!("poll message with error: {:?}", e)
                }
            }
        }
    }
}
