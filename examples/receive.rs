extern crate rosc;

use std::{net, env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage ./receive IP PORT");
        process::exit(1);
    }

    let addr: net::Ipv4Addr = parse_ip_v4(&args[1]);
    let port: u16 = args[2]
                        .parse()
                        .ok()
                        .expect("PORT must be in range [0, 65535]!");

    let sock = net::UdpSocket::bind((addr, port));
    println!("Listening to {}:{}", addr, port);
    drop(sock);
}

fn parse_ip_v4(raw_addr: &String) -> net::Ipv4Addr {
    let octets: Vec<u8> = raw_addr.splitn(4, '.')
                                  .map(|octet| {
                                      octet.parse::<u8>()
                                      .ok()
                                      .expect("Octet must be in range [0, 255]")
                                  })
                                  .collect();
    net::Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
}
