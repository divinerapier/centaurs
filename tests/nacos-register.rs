use std::sync::Arc;

use centaurs::nacos::Nacos;
use centaurs::servicediscovery::Registry;
use nacos_rust_client::client::naming_client::Instance;

#[tokio::test]
async fn test_register() -> Result<(), String> {
    let nacos = Nacos::from_env().expect("failed to create nacos client");
    let service_name = std::env::var("SERVICE_NAME").unwrap();
    nacos
        .registry("namespace")
        .register(Arc::new(Instance::new(
            &centaurs::datalink::local_address("docker0")
                .ok_or_else(|| "nic not found".to_string())?,
            6789,
            &service_name,
            "group_name",
            "",
            "",
            None,
        )))
        .await;
    Ok(())
}
