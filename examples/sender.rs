extern crate rosc;

use std::{env, process, thread, f32};
use std::net::{UdpSocket, SocketAddrV4};
use std::str::FromStr;
use std::time::Duration;
use rosc::types::{OscPacket, OscMessage, OscType};
use rosc::encoder;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage ./sender HOST_IP:HOST_PORT CLIENT_IP:CLIENT_PORT");
        process::exit(1);
    }

    let host_addr = SocketAddrV4::from_str(&args[1]).unwrap();
    let to_addr = SocketAddrV4::from_str(&args[2]).unwrap();
    let sock = UdpSocket::bind(host_addr).unwrap();

    // switch view
    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                      addr: "/3".to_string(),
                      args: None,
                  }))
                      .unwrap();

    sock.send_to(&msg_buf, to_addr).unwrap();

    // send random values to xy fields
    let steps = 128;
    let step_size: f32 = 2.0 * f32::consts::PI / steps as f32;
    for i in 0.. {
        let x = 0.5 + (step_size * (i % steps) as f32).sin() / 2.0;
        let y = 0.5 + (step_size * (i % steps) as f32).cos() / 2.0;
        let mut msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                              addr: "/3/xy1".to_string(),
                              args: Some(vec![OscType::Float(x), OscType::Float(y)]),
                          }))
                              .unwrap();

        sock.send_to(&msg_buf, to_addr).unwrap();
        msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                          addr: "/3/xy2".to_string(),
                          args: Some(vec![OscType::Float(y), OscType::Float(x)]),
                      }))
                          .unwrap();
        sock.send_to(&msg_buf, to_addr).unwrap();
        thread::sleep(Duration::from_millis(20));
    }

    drop(sock);
}
