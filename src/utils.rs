use std::net;

pub fn parse_ip_v4(raw_addr: &String) -> net::Ipv4Addr {
    let octets: Vec<u8> = raw_addr.splitn(4, '.')
                                  .map(|octet| {
                                      octet.parse::<u8>()
                                           .ok()
                                           .expect("Octet must be in range [0, 255]")
                                  })
                                  .collect();
    net::Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
}
