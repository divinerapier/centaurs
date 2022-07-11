use std::time::Duration;

use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{BaseConsumer, CommitMode, Consumer as _},
    message::BorrowedMessage,
    util::Timeout,
    ClientConfig,
};

pub struct Consumer {
    inner: BaseConsumer,
    poll_timeout: Timeout,
    _auto_commit: bool,
}

pub struct ConsumerBuilder {
    // https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md
    bootstrap: Option<String>,
    group_id: Option<String>,
    client_id: Option<String>,

    // http://blog.51yip.com/hadoop/2130.html
    auto_offset_reset: String,

    /// Automatically store offset of last message provided to application.
    /// The offset store is an in-memory store of the next offset to
    /// (auto-)commit for each partition.
    enable_auto_offset_store: bool,

    /// Automatically and periodically commit offsets in the background.
    /// Note: setting this to false does not prevent the consumer from
    /// fetching previously committed start offsets.
    /// To circumvent this behaviour set specific start offsets per
    /// partition in the call to assign().
    /// Default: true
    enable_auto_commit: bool,

    /// Emit RD_KAFKA_RESP_ERR__PARTITION_EOF event whenever the consumer
    /// reaches the end of a partition.
    /// Default: false
    enable_partition_eof: bool,

    /// The frequency in milliseconds that the consumer offsets are committed (written)
    /// to offset storage. (0 = disable). This setting is used by the high-level consumer.
    /// Default: 5000
    auto_commit_interval_ms: i32,

    /// Maximum allowed time between calls to consume messages (e.g., rd_kafka_consumer_poll())
    /// for high-level consumers. If this interval is exceeded the consumer is
    /// considered failed and the group will rebalance in order to reassign the
    /// partitions to another consumer group member. Warning: Offset commits
    /// may be not possible at this point.
    ///
    /// Note: It is recommended to set `enable.auto.offset.store=false` for long-time
    /// processing applications and then explicitly store offsets (using `offsets_store()`)
    /// after message processing, to make sure offsets are not auto-committed prior
    /// to processing has finished. The interval is checked two times per second.
    /// See KIP-62 for more information.
    // default: 300000ms 300s 5m
    max_poll_interval_ms: i32,

    /// Client group session and failure detection timeout. The consumer sends
    /// periodic heartbeats (`heartbeat.interval.ms`) to indicate its liveness to
    /// the broker. If no hearts are received by the broker for a group member
    /// within the session timeout, the broker will remove the consumer from the
    /// group and trigger a rebalance. The allowed range is configured with the
    /// broker configuration properties `group.min.session.timeout.ms` and
    /// `group.max.session.timeout.ms`. Also see `max.poll.interval.ms`.
    /// Default: 45000
    session_timeout_ms: i32,
    /// Group session keepalive heartbeat interval. Default 3000
    heartbeat_interval_ms: i32,
}

impl Default for ConsumerBuilder {
    fn default() -> Self {
        Self {
            bootstrap: Default::default(),
            group_id: Default::default(),
            client_id: Default::default(),
            auto_offset_reset: "earliest".to_string(),
            enable_auto_offset_store: false,
            enable_auto_commit: true,
            enable_partition_eof: false,
            auto_commit_interval_ms: 1000,
            max_poll_interval_ms: 300000,
            session_timeout_ms: 10000,
            heartbeat_interval_ms: 3000,
        }
    }
}

impl ConsumerBuilder {
    pub fn set_bootstrap<S: Into<String>>(mut self, bootstrap: S) -> Self {
        self.bootstrap = Some(bootstrap.into());
        self
    }

    pub fn set_group_id<S: Into<String>>(mut self, group_id: S) -> Self {
        self.group_id = Some(group_id.into());
        self
    }

    pub fn set_client_id<S: Into<String>>(mut self, client_id: S) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn set_auto_offset_reset<S: Into<String>>(mut self, auto_offset_reset: S) -> Self {
        self.auto_offset_reset = auto_offset_reset.into();
        self
    }

    pub fn set_enable_auto_offset_store(mut self, enable_auto_offset_store: bool) -> Self {
        self.enable_auto_offset_store = enable_auto_offset_store;
        self
    }

    pub fn set_enable_auto_commit(mut self, enable_auto_commit: bool) -> Self {
        self.enable_auto_commit = enable_auto_commit;
        self
    }

    pub fn set_enable_partition_eof(mut self, enable_partition_eof: bool) -> Self {
        self.enable_partition_eof = enable_partition_eof;
        self
    }

    pub fn set_auto_commit_interval_ms(mut self, auto_commit_interval_ms: i32) -> Self {
        self.auto_commit_interval_ms = auto_commit_interval_ms;
        self
    }

    pub fn set_max_poll_interval_ms(mut self, max_poll_interval_ms: i32) -> Self {
        self.max_poll_interval_ms = max_poll_interval_ms;
        self
    }

    pub fn set_heartbeat_interval_ms(mut self, heartbeat_interval_ms: i32) -> Self {
        self.heartbeat_interval_ms = heartbeat_interval_ms;
        self
    }

    pub fn set_session_timeout_ms(mut self, session_timeout_ms: i32) -> Self {
        self.session_timeout_ms = session_timeout_ms;
        self
    }

    pub fn build(self) -> Result<Consumer, rdkafka::error::KafkaError> {
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", self.bootstrap.unwrap_or_default())
            .set("group.id", self.group_id.unwrap_or_default())
            .set("auto.offset.reset", self.auto_offset_reset)
            .set(
                "enable.auto.offset.store",
                self.enable_auto_offset_store.to_string(),
            )
            .set("enable.auto.commit", self.enable_auto_commit.to_string())
            .set(
                "enable.partition.eof",
                self.enable_partition_eof.to_string(),
            )
            .set(
                "auto.commit.interval.ms",
                self.auto_commit_interval_ms.to_string(),
            )
            .set("session.timeout.ms", self.session_timeout_ms.to_string())
            .set(
                "heartbeat.interval.ms",
                self.heartbeat_interval_ms.to_string(),
            )
            .set(
                "enable.partition.eof",
                self.enable_partition_eof.to_string(),
            )
            .set("session.timeout.ms", &self.session_timeout_ms.to_string())
            .set_log_level(RDKafkaLogLevel::Warning)
            .create()?;
        let poll_timeout = Timeout::After(Duration::from_micros(self.heartbeat_interval_ms as u64));
        Ok(Consumer {
            inner: consumer,
            poll_timeout,
            _auto_commit: self.enable_auto_commit,
        })
    }
}

impl Consumer {
    pub fn builder() -> ConsumerBuilder {
        Default::default()
    }
}

#[async_trait::async_trait]
impl<'a> super::Consumer for &'a Consumer {
    type Output = BorrowedMessage<'a>;

    type Error = rdkafka::error::KafkaError;

    async fn poll(&self) -> Result<Option<Self::Output>, Self::Error> {
        match self.inner.poll(self.poll_timeout).transpose()? {
            Some(message) => Ok(Some(message)),
            None => Ok(None),
        }
    }

    fn subscribe(
        &self,
        topics: &[&str],
    ) -> Result<super::SubscribeGuard<Self::Output, Self::Error>, Self::Error> {
        self.inner.subscribe(topics)?;
        Ok(super::SubscribeGuard::<_, _> { consumer: self })
    }

    fn unsubscribe(&self) {
        self.inner.unsubscribe();
    }

    async fn commit(&self, message: Self::Output) -> Result<(), Self::Error> {
        if self.auto_commit() {
            self.inner.store_offset_from_message(&message)?;
        } else {
            self.inner.commit_message(&message, CommitMode::Async)?;
        }
        Ok(())
    }

    fn auto_commit(&self) -> bool {
        self._auto_commit
    }
}
