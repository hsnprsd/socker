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

    // sleep(Duration::from_secs(10));

    // let n = 1000_000_000;
    // let mut arr = vec![];
    // for i in 0..n {
    //     arr.push(i);
    // }
    // println!("{}", arr.iter().sum::<u64>());
}
