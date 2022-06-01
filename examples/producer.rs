use std::time::{SystemTime, UNIX_EPOCH};

use rdkafka::{producer::FutureRecord, util::Timeout};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let producer: rdkafka::producer::FutureProducer = rdkafka::ClientConfig::new()
        .set("bootstrap.servers", &std::env::var("BOOTSTRAP").unwrap())
        .create()
        .unwrap();

    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let tasks = (0..8).map(|tid| {
        let producer = producer.clone();
        async move {
            for i in 0.. {
                let result = producer
                    .send(
                        FutureRecord::to(&std::env::var("TOPIC").unwrap())
                            .payload(&format!("{}-{}-{}", tid, start, i))
                            .key(""),
                        Timeout::Never,
                    )
                    .await;
                tracing::info!("result: {:?}", result);
            }
        }
    });

    futures::future::join_all(tasks).await;
}
