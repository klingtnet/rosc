extern crate rosc;

use std::{net, env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage ./receive IP PORT");
        process::exit(1);
    }

    let addr: net::Ipv4Addr = rosc::utils::parse_ip_v4(&args[1]);
    let port: u16 = args[2]
                        .parse()
                        .ok()
                        .expect("PORT must be in range [0, 65535]!");

    let sock = net::UdpSocket::bind((addr, port));
    println!("Listening to {}:{}", addr, port);
    drop(sock);
}
