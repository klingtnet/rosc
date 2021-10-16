extern crate rosc;

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::time::Duration;
use std::{env, f32, thread};

fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
    SocketAddrV4::from_str(arg).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let usage = format!(
        "Usage: {} HOST_IP:HOST_PORT CLIENT_IP:CLIENT_PORT",
        &args[0]
    );
    if args.len() < 3 {
        panic!("{}", usage);
    }
    let host_addr = get_addr_from_arg(&args[1]);
    let to_addr = get_addr_from_arg(&args[2]);
    let sock = UdpSocket::bind(host_addr).unwrap();

    // switch view
    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
        addr: "/3".to_string(),
        args: vec![],
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
            args: vec![OscType::Float(x), OscType::Float(y)],
        }))
        .unwrap();

        sock.send_to(&msg_buf, to_addr).unwrap();
        msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/3/xy2".to_string(),
            args: vec![OscType::Float(y), OscType::Float(x)],
        }))
        .unwrap();
        sock.send_to(&msg_buf, to_addr).unwrap();
        thread::sleep(Duration::from_millis(20));
    }
}
