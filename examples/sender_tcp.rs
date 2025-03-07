extern crate rosc;

use rosc::encoder::{self, Output};
use rosc::{OscMessage, OscPacket, OscType};
use std::io::Write;
use std::net::{SocketAddrV4, TcpStream};
use std::str::FromStr;
use std::time::Duration;
use std::{env, f32, thread};

fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
    SocketAddrV4::from_str(arg).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage: {} IP:PORT", &args[0]);
    if args.len() < 2 {
        panic!("{}", usage);
    }
    let addr = get_addr_from_arg(&args[1]);
    let mut stream = TcpStream::connect(addr).unwrap();

    // switch view
    let msg_buf = encoder::encode_tcp(&OscPacket::Message(OscMessage {
        addr: "/3".to_string(),
        args: vec![],
    }))
    .unwrap();

    stream.write_all(&msg_buf).unwrap();

    // send random values to xy fields
    let steps = 128;
    let step_size: f32 = 2.0 * f32::consts::PI / steps as f32;
    for i in 0.. {
        let x = 0.5 + (step_size * (i % steps) as f32).sin() / 2.0;
        let y = 0.5 + (step_size * (i % steps) as f32).cos() / 2.0;
        let mut msg_buf = encoder::encode_tcp(&OscPacket::Message(OscMessage {
            addr: "/3/xy1".to_string(),
            args: vec![OscType::Float(x), OscType::Float(y)],
        }))
        .unwrap();

        stream.write_all(&msg_buf).unwrap();
        msg_buf = encoder::encode_tcp(&OscPacket::Message(OscMessage {
            addr: "/3/xy2".to_string(),
            args: vec![OscType::Float(y), OscType::Float(x)],
        }))
        .unwrap();
        stream.write_all(&msg_buf).unwrap();
        thread::sleep(Duration::from_millis(20));
    }
}
