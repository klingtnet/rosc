extern crate rosc;

use std::{net, env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage ./receive IP:PORT");
        process::exit(1);
    }

    match rosc::utils::parse_ip_and_port(&args[1]) {
        Ok((ip, port)) => {
            let sock = net::UdpSocket::bind((ip, port));
            println!("Listening to {}:{}", ip, port);
            drop(sock);
        }
        Err(e) => println!("{}", e),
    }
}
