use std::{
    fs,
    io::{Read, Write},
    path,
    process::Command,
    thread::sleep,
    time::Duration,
};

fn main() {
    let mut random = fs::File::open("/dev/random").unwrap();
    let mut buf: [u8; 16] = [0; 16];
    random.read_exact(&mut buf).unwrap();

    let cg = hex::encode(&buf);

    let cg_root = path::Path::new("/sys/fs/cgroup");

    fs::create_dir(cg_root.join(&cg)).unwrap();

    let memory_max: u64 = 4 * 1024 * 1024 * 1024; // 1 GB

    fs::File::create(cg_root.join(&cg).join("memory.max"))
        .unwrap()
        .write_all(memory_max.to_string().as_bytes())
        .unwrap();
    fs::File::create(cg_root.join(&cg).join("memory.swap.max"))
        .unwrap()
        .write_all("max".as_bytes())
        .unwrap();
    fs::File::create(cg_root.join(&cg).join("memory.oom.group"))
        .unwrap()
        .write_all("1".as_bytes())
        .unwrap();

    let program = "./target/debug/heavy_mem";

    let result = Command::new("./target/debug/socker_exec")
        .arg(program)
        .arg(&cg)
        .output()
        .unwrap();
    print!(
        "status: {}\nstderr: {}\nstdout: {}\n",
        result.status,
        String::from_utf8(result.stderr).unwrap(),
        String::from_utf8(result.stdout).unwrap()
    );

    sleep(Duration::from_secs(1));

    fs::remove_dir(cg_root.join(&cg)).unwrap();
}
