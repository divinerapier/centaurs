use rdkafka::{producer::FutureRecord, util::Timeout};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let producer: rdkafka::producer::FutureProducer = rdkafka::ClientConfig::new()
        .set("bootstrap.servers", &std::env::var("BOOTSTRAP").unwrap())
        .create()
        .unwrap();

    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for i in 0.. {
        let result = producer
            .send(
                FutureRecord::to(&std::env::var("TOPIC").unwrap())
                    .payload(&format!("{}-{}", start, i))
                    .key(""),
                Timeout::Never,
            )
            .await;
        tracing::info!("result: {:?}", result);
    }
}
