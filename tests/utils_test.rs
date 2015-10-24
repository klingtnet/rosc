extern crate rosc;

use std::net;

#[test]
fn test_parse_ip_and_port() {
    let valid_addr = "127.0.0.1:8080".to_string();
    let mut addr = rosc::utils::parse_ip_and_port(&valid_addr);
    assert!(addr.is_ok());
    let (ip, port) = addr.unwrap();
    assert_eq!(port, 8080u16);
    assert_eq!(ip, net::Ipv4Addr::new(127u8, 0u8, 0u8, 1u8));

    let bad_addr = "127..1:8080".to_string();
    addr = rosc::utils::parse_ip_and_port(&bad_addr);
    assert!(addr.is_err());

    let bad_addr_port = "192.168.0.10:99999".to_string();
    addr = rosc::utils::parse_ip_and_port(&bad_addr_port);
    assert!(addr.is_err());
}