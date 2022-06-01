use nacos_rust_client::client::{config_client::{ConfigKey, ConfigInnerRequestClient}, HostInfo};

use super::{Key, Nacos};

pub struct Configuration(ConfigInnerRequestClient);

impl Key {
    fn to_nacos(&self) -> ConfigKey {
        ConfigKey::new(&self.data_id, &self.group, &self.tenant)
    }
}

impl Nacos {
    pub fn configuration(&self) -> Configuration {
        Configuration(ConfigInnerRequestClient::new(HostInfo::new(
            &self.server,
            self.port,
        )))
    }
}

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
