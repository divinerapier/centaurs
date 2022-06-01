use super::Instance;

impl super::Registry for crate::nacos::API {
    fn register(&self, instance: &Instance) {
        crate::nacos::API::register(&self, &instance);
    }

    fn deregister(&self, instance: &Instance) {
        crate::nacos::API::deregister(&self, instance);
    }
}

#[tokio::test]
async fn test_register() -> Result<(), String> {
    let nacos = crate::nacos::API::new_from_env().expect("failed to create nacos client");
    let service_name = std::env::var("SERVICE_NAME").unwrap().to_string();
    nacos.register(&Instance {
        ip: crate::datalink::local_address("docker0").ok_or_else(|| "nic not found".to_string())?,
        port: 6789,
        service_name: service_name.clone(),
    });
    nacos.register(&Instance {
        ip: crate::datalink::local_address("enp4s0f4u2u4")
            .ok_or_else(|| "nic not found".to_string())?,
        port: 5678,
        service_name: service_name.clone(),
    });
    nacos.register(&Instance {
        ip: crate::datalink::local_address("wlan0").ok_or_else(|| "nic not found".to_string())?,
        port: 4567,
        service_name: service_name.clone(),
    });
    Ok(())
}
