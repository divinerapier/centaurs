pub fn local_address(name: &str) -> Option<String> {
    if let Ok(ip) = std::env::var("KUBERNETES_POD_IP") {
        return Some(ip);
    }
    let ip = pnet_datalink::interfaces()
        .into_iter()
        .filter(|nic| nic.name.eq(&name))
        .take(1)
        .next()?
        .ips
        .into_iter()
        .find(|ip| ip.is_ipv4())?;
    Some(ip.ip().to_string())
}

#[test]
fn test_local_address() {
    assert_eq!(local_address("docker0"), Some("169.254.32.1".to_string()))
}
