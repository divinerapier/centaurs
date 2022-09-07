use nacos_rust_client::client::config_client::ConfigInnerRequestClient;
use nacos_rust_client::client::HostInfo;

use super::{Key, Nacos};

impl Nacos {
    pub fn configuration(&self) -> Configuration {
        Configuration(ConfigInnerRequestClient::new(HostInfo::new(
            &self.server,
            self.port,
        )))
    }
}

pub struct Configuration(ConfigInnerRequestClient);

#[async_trait::async_trait]
impl crate::configuration::Loader for Configuration {
    type Key = Key;

    async fn load(&self, key: Self::Key) -> Result<String, crate::configuration::Error> {
        let key = key.to_nacos();
        let content = self
            .0
            .get_config(&key)
            .await
            .map_err(crate::configuration::Error::Other)?;

        Ok(content)
    }
}
