use std::{process::Command, thread::sleep, time::Duration};

fn main() {
    let mut proc = Command::new("ip").args(["link"]).spawn().unwrap();
    let status = proc.wait().unwrap();
    println!("{}", status);

    // sleep(Duration::from_secs(10));

    // let n = 1000_000_000;
    // let mut arr = vec![];
    // for i in 0..n {
    //     arr.push(i);
    // }
    // println!("{}", arr.iter().sum::<u64>());
}
