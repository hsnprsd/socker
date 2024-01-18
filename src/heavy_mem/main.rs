fn main() {
    let n = 1000_000_000;
    let mut arr = vec![];
    for i in 0..n {
        arr.push(i);
    }
    println!("{}", arr.iter().sum::<u64>());
}
