use centaurs::nacos::Nacos;
use nacos_rust_client::client::naming_client::Instance;

#[tokio::test]
async fn test_discovery() -> Result<(), String> {
    let nacos = Nacos::from_env().expect("failed to create nacos client");
    let service_name = std::env::var("SERVICE_NAME").unwrap();
    let registry = nacos.registry("namespace");
    let discovery = centaurs::servicediscovery::Discovery::new(registry);
    let _guard = discovery
        .register(Instance::new(
            &centaurs::datalink::local_address("docker0")
                .ok_or_else(|| "nic not found".to_string())?,
            6789,
            &service_name,
            "group_name",
            "",
            "",
            None,
        ))
        .await;
    Ok(())
}
