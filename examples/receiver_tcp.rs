extern crate rosc;

use rosc::decoder::decode_tcp;
use rosc::OscPacket;
use std::env;
use std::io::Read;
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;

fn main() {
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} [bind]|[conn] IP:PORT\n Use bind to open a local TCP socket at address, use conn to connect to the given addres", args[0]);
    if args.len() < 3 {
        println!("{}", usage);
        ::std::process::exit(1)
    }
    println!("{:?}", args);
    let addr = match SocketAddrV4::from_str(&args[2]) {
        Ok(addr) => addr,
        Err(_) => panic!("{}", usage),
    };

    if let Some(mut stream) = if args[1].as_str() == "bind" {
        let listener = TcpListener::bind(addr).unwrap();
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("Connected to {}", addr);
                Some(stream)
            }
            Err(e) => {
                println!("Error accepting TCP: {}", e);
                None
            }
        }
    } else if args[1].as_str() == "conn" {
        let stream = Some(TcpStream::connect(addr).unwrap());
        println!("Connected to {}", addr);
        stream
    } else {
        panic!("{}", usage);
    } {
        let mut buf = [0u8; rosc::decoder::MTU];

        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    // End-Of-File
                }
                Ok(size) => {
                    println!("Received packet with size {} from: {}", size, addr);

                    let mut slice = &buf[0..size];

                    while let Some(remainder) = match decode_tcp(slice) {
                        Ok((remainder, None)) => {
                            if remainder.is_empty() {
                                None
                            } else {
                                Some(remainder)
                            }
                        }
                        Ok((remainder, Some(packet))) => {
                            handle_packet(packet);
                            if remainder.is_empty() {
                                None
                            } else {
                                Some(remainder)
                            }
                        }
                        Err(e) => {
                            println!("Error parsing OscPacket: {}", e);
                            None
                        }
                    } {
                        slice = remainder;
                    }
                }
                Err(e) => {
                    println!("Error reading TCP stream: {}", e);
                    break;
                }
            }
        }
    }
}

fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            println!("OSC arguments: {:?}", msg.args);
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}
