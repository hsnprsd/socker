use std::{io::Read, net, process::Command};

fn main() {
    let mut proc = Command::new("ip").args(["addr"]).spawn().unwrap();
    let _ = proc.wait().unwrap();

    let mut stream = net::TcpStream::connect("10.10.0.2:8000").unwrap();
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    println!("{}", buf);
}
