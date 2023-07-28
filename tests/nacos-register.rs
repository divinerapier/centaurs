use centaurs::nacos::Nacos;
use centaurs::servicediscovery::Discovery;
use nacos_rust_client::client::naming_client::Instance;

#[tokio::test]
async fn test_register() -> Result<(), String> {
    const SERVICE_NAME: &str = "service_name";
    const GROUP_NAME: &str = "group_name";
    const NAMESPACE: &str = "namespace";

    let nacos = Nacos::from_env().expect("failed to create nacos client");
    let discovery = Discovery::new(nacos.registry(NAMESPACE));
    let instances = discovery.query(SERVICE_NAME, GROUP_NAME).await;

    println!("{:?}", instances);
    let ip =
        centaurs::datalink::local_address("eth0").ok_or_else(|| "nic not found".to_string())?;

    let _guard = discovery
        .register(Instance::new(
            &ip,
            6789,
            SERVICE_NAME,
            GROUP_NAME,
            "",
            "",
            None,
        ))
        .await;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let instances = discovery.query(SERVICE_NAME, GROUP_NAME).await;

    println!("{:?}", instances);

    assert_eq!(1, instances.len());
    assert_eq!(&ip, &instances[0].ip);
    assert_eq!(6789, instances[0].port);

    drop(_guard);

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let instances = discovery.query(SERVICE_NAME, GROUP_NAME).await;

    println!("{:?}", instances);

    Ok(())
}
