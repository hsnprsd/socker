use std::{env, ffi::CString, fs, io, os::fd::AsRawFd};

mod cgroup;

use cgroup::{Bytes, CGroup};
use libc::{pid_t, waitpid};

unsafe fn execute_in_cgroup(program: String, cgroup: &CGroup) -> Result<pid_t, io::Error> {
    let pid = libc::fork();
    if pid != 0 {
        // parent
        Ok(pid)
    } else {
        cgroup.write_pid(pid).expect("could not move to cgroup");
        let program_cstr = CString::new(program).unwrap();
        let stdout = fs::File::create("./log/stdout").expect("could not create stdout");
        let stderr = fs::File::create("./log/stderr").expect("could not create stderr");
        libc::dup2(stdout.as_raw_fd(), 1);
        libc::dup2(stderr.as_raw_fd(), 2);
        let rc = libc::execv(program_cstr.as_ptr(), [program_cstr.as_ptr()].as_ptr());
        if rc < 0 {
            Err(io::Error::last_os_error())
        } else {
            todo!() // should not happen
        }
    }
}

fn main() {
    let mut args = env::args();
    args.next();
    let program = args.next().expect("Usage: [program]");
    println!("program: {}", program);

    let cgroup = CGroup::new(None, Some(Bytes::GB * 4), Some(Bytes::GB * 0));
    cgroup.create().expect("could not create cgroup");

    unsafe {
        let child = execute_in_cgroup(program.to_string(), &cgroup)
            .expect("failed to run program in cgroup");
        println!("child pid: {}", child);
        let mut status: i32 = 0;
        waitpid(child, &mut status, 0);
        println!("status: {}", status);

        cgroup.remove().expect("couldnt remove cgroup");
    }
}
