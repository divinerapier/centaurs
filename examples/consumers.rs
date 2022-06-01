use std::time::Duration;

use centaurs::messaging::{
    kafka::Consumer, Failover, FailoverProcessor, Processor, RetriableProcessor,
    RetriableProcessorError, Runner,
};
use futures::{Future, FutureExt};
use rdkafka::{error::KafkaError, message::BorrowedMessage, Message};
use tokio::signal::unix::SignalKind;

pub struct TestKafkaProcessor {}

#[async_trait::async_trait]
impl<'a> Processor for &'a TestKafkaProcessor {
    type Item = BorrowedMessage<'a>;
    type Error = rdkafka::error::KafkaError;
    type Output = ();

    async fn process(&self, item: &Self::Item) -> Result<Self::Output, Self::Error> {
        let payload = item
            .payload()
            .map(|payload| unsafe { std::str::from_utf8_unchecked(payload) });
        let partition = item.partition();
        let offset = item.offset();
        tracing::info!(
            "partition: {} offset: {} - payload: {:?}",
            partition,
            offset,
            payload
        );
        Ok(())
    }
}

pub(crate) struct TestKafkaFailover {}

#[async_trait::async_trait]
impl<'a> Failover for &'a TestKafkaFailover {
    type Item = BorrowedMessage<'a>;
    type InputError = RetriableProcessorError<KafkaError>;
    type Error = KafkaError;

    async fn failover(&self, item: &Self::Item, _ie: &Self::InputError) -> Result<(), Self::Error> {
        let _payload = item.payload();
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    basic_consumer().await;
    // retriable_consumer().await;
    // failover_consumer().await;
}

async fn basic_consumer() {
    let consumer = Consumer::builder()
        .set_bootstrap(std::env::var("BOOTSTRAP").unwrap())
        .set_group_id(std::env::var("GROUP_ID").unwrap())
        .set_client_id(std::env::var("CLIENT_ID").unwrap_or_default())
        .build()
        .unwrap();
    let processor = TestKafkaProcessor {};
    let retries = vec![Duration::from_secs(1)];
    // &TestKafkaProcessor 类型实现了 trait Processor, 因此下面需要传递 &processor
    let processor = RetriableProcessor::new(&processor, retries.into_iter());
    let runner = Runner::new(&consumer, processor);
    let result = runner
        .run(&[&std::env::var("TOPIC").unwrap()], catch_signal())
        .await;
    tracing::info!("final result: {:?}", result);
}

async fn retriable_consumer() {
    let consumer = Consumer::builder()
        .set_bootstrap(std::env::var("BOOTSTRAP").unwrap())
        .set_group_id(std::env::var("GROUP_ID").unwrap())
        .set_client_id(std::env::var("CLIENT_ID").unwrap_or_default())
        .build()
        .unwrap();
    let processor = TestKafkaProcessor {};
    let retries = vec![Duration::from_secs(1)];
    // &TestKafkaProcessor 类型实现了 trait Processor, 因此下面需要传递 &processor
    let processor = centaurs::messaging::RetriableProcessor::new(&processor, retries.into_iter());
    let runner = Runner::new(&consumer, processor);
    runner
        .run(&[&std::env::var("TOPIC").unwrap()], catch_signal())
        .await
        .unwrap();
}

async fn failover_consumer() {
    let consumer = Consumer::builder()
        .set_bootstrap(std::env::var("BOOTSTRAP").unwrap())
        .set_group_id(std::env::var("GROUP_ID").unwrap())
        .build()
        .unwrap();
    let processor = TestKafkaProcessor {};
    let retries = vec![Duration::from_secs(1)];
    // &TestKafkaProcessor 类型实现了 trait Processor, 因此下面需要传递 &processor
    let processor = RetriableProcessor::new(&processor, retries.into_iter());
    let processor = FailoverProcessor::new(processor, &TestKafkaFailover {});
    let runner = Runner::new(&consumer, processor);
    runner
        .run(&[&std::env::var("TOPIC").unwrap()], catch_signal())
        .await
        .unwrap();
}

fn catch_signal() -> impl Future {
    async {
        let mut signal2 = tokio::signal::unix::signal(SignalKind::interrupt()).unwrap();
        let mut signal15 = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
        let mut signal_user1 = tokio::signal::unix::signal(SignalKind::user_defined1()).unwrap();
        let mut signal_user2 = tokio::signal::unix::signal(SignalKind::user_defined2()).unwrap();
        futures::select! {
            _ = signal2.recv().fuse() => {
                tracing::warn!("receive signal 2");
            }
            _ = signal15.recv().fuse()=> {
                tracing::warn!("receive signal 15");
            }
            _ = signal_user1.recv().fuse() => {
                tracing::warn!("receive signal user1");
            }
            _ = signal_user2.recv().fuse()=> {
                tracing::warn!("receive signal user2");
            }
        }
    }
}
