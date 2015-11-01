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
            let sock = match net::UdpSocket::bind((ip, port)) {
                Ok(sock_ok) => sock_ok,
                Err(e) => {
                    println!("Could not bind socket: {}", e);
                    process::exit(1);
                }
            };
            println!("Listening to {}:{}", ip, port);

            let mut buf: [u8; rosc::osc_decoder::MTP] = [0u8; rosc::osc_decoder::MTP];

            loop {
                match sock.recv_from(&mut buf) {
                    Ok((size, addr)) => {
                        println!("addr: {}, size: {}", addr, size);
                        rosc::osc_decoder::decode(&mut buf);
                    }
                    Err(e) => {
                        println!("Error receiving from socket: {}", e);
                        break;
                    }
                }
            }

            drop(sock);
        }
        Err(e) => println!("{}", e),
    }
}
