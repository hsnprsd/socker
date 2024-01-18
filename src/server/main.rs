use std::{io::Write, net, process::Command};

fn main() {
    let mut proc = Command::new("ip").args(["addr"]).spawn().unwrap();
    let _ = proc.wait().unwrap();

    // listen on port 8000
    let listener = net::TcpListener::bind("0.0.0.0:8000").unwrap();
    println!("listening on port 8000 on all interfaces...");
    loop {
        let (mut stream, _) = listener.accept().unwrap();
        stream.write(b"hello world\n").unwrap();
    }
}
