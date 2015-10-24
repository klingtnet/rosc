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
            let sock = net::UdpSocket::bind((ip, port)).unwrap();
            println!("Listening to {}:{}", ip, port);

            let mut buf: [u8; rosc::osc_server::MTP] = [0u8; rosc::osc_server::MTP];

            loop {
                match sock.recv_from(&mut buf) {
                    Ok((size, addr)) => {
                        println!("addr: {}, size: {}", addr, size);
                        rosc::osc_server::destruct(&mut buf, size);
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
