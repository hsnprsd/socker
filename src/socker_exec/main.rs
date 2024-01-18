use std::{env, fs, io::Write, path, process::Command};

fn main() {
    let mut args = env::args();
    args.next();
    let program = args.next().unwrap();
    let cg = args.next().unwrap();

    println!("executing {} in cg {}", program, cg);

    let cg_root = path::Path::new("/sys/fs/cgroup/");

    // move to cgroup
    // println!("moving self to cg {}", cg_root.join(&cg).to_str().unwrap());
    fs::File::create(cg_root.join(cg).join("cgroup.procs"))
        .expect("could not open cgroup.procs file")
        .write_all("0".as_bytes())
        .expect("could not write to cgroup.procs file");

    let output = Command::new(program).output().unwrap();
    println!("status: {}", output.status);
}
