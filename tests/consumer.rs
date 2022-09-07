use std::sync::atomic::AtomicUsize;

use centaurs::messaging::{kafka::Consumer, Processor, Runner};
use rdkafka::message::BorrowedMessage;

struct BaseProcessor {
    counter: AtomicUsize,
    start: chrono::DateTime<chrono::Local>,
}

#[async_trait::async_trait]
impl<'a> Processor for &'a BaseProcessor {
    type Item = BorrowedMessage<'a>;
    type Output = ();
    type Error = String;

    async fn process(&self, _item: &Self::Item) -> Result<Self::Output, Self::Error> {
        let counter = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let elapsed = chrono::Local::now().signed_duration_since(self.start);
        if counter % 1000 == 0 {
            println!(
                "processed {} messages. elapsed: {}. speed: {}",
                counter,
                elapsed,
                counter as f64 / (elapsed.num_nanoseconds().unwrap() as f64 / 1_000_000_000.0)
            );
        }
        Ok(())
    }
}

#[tokio::test(worker_threads = 2, flavor = "multi_thread")]
async fn test() {
    let consumer = Consumer::builder()
        .set_bootstrap("localhost:9092")
        .set_group_id("test")
        .build()
        .unwrap();
    let processor = BaseProcessor {
        counter: AtomicUsize::new(0),
        start: chrono::Local::now(),
    };
    let runner = Runner::new(&consumer, &processor);
    let topics = &["topic"];
    let timer = async {
        println!("start timer: {}", chrono::Local::now());
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        println!("quit: {}", chrono::Local::now());
    };
    runner.run(topics, timer).await.unwrap();
}
