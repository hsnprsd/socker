use std::{
    ffi::CString,
    fs,
    io::{self, Read},
    os::{fd::AsRawFd, raw::c_void},
};

use crate::{
    cgroup::CGroup,
    network::{NetNs, VETHPair},
};
use libc::{self};
use log::info;

const STACK_SIZE: usize = 1000_000; // 1MB

pub struct Container {
    name: String,
    executable: String,
    cgroup: CGroup,
    netns: NetNs,
    veth_pair: VETHPair,
}

pub struct ResourceLimits {
    pub memory_limit: Option<usize>,
    pub memory_swap_limit: Option<usize>,
}

pub struct ContainerResult {}

#[derive(Debug)]
pub enum ContainerError {
    CGroupCreationFailed(io::Error),
    Failed(i32),
    Unknown(String),
}

extern "C" fn cb(arg: *mut c_void) -> i32 {
    unsafe {
        let container: Container = (arg as *mut Container).read();
        let prog = CString::new(container.executable).unwrap();

        // let run_dir = path::Path::new("./run/");
        // let fd = fs::File::create(run_dir.join(&container.name).join("stdout"))
        //     .unwrap()
        //     .as_raw_fd();
        // let rc = libc::dup2(fd, 1);

        // let fd = fs::File::create(run_dir.join(&container.name).join("stderr"))
        //     .unwrap()
        //     .as_raw_fd();
        // let rc = libc::dup2(fd, 2);

        container.cgroup.write_pid(0).unwrap();

        // setns network
        let fd = fs::File::open(container.netns.path()).unwrap();
        libc::setns(fd.as_raw_fd(), libc::CLONE_NEWNET);

        container.veth_pair.setup_peer().unwrap();

        let argv = [prog.as_ptr()];
        libc::execv(prog.as_ptr(), argv.as_ptr())
    }
}

fn random_hex_encoded_string() -> String {
    let mut random = fs::File::open("/dev/random").unwrap();
    let mut buf: [u8; 5] = [0; 5];
    random.read_exact(&mut buf).unwrap();

    return hex::encode(&buf);
}

impl Container {
    pub fn new(executable: String, resource_limits: ResourceLimits) -> Self {
        let id = random_hex_encoded_string();
        let name = format!("container-{}", id);

        let cgroup = CGroup::new(
            format!("{}.slice", &name),
            resource_limits.memory_limit,
            resource_limits.memory_swap_limit,
        );

        let netns = NetNs::new(format!("{}", name));

        let veth_pair = VETHPair::new(
            format!("v-{}", &id),
            None,
            format!("v-p-{}", &id),
            String::from("10.10.0.2/16"),
            Some(netns.name()),
        )
        .unwrap();

        Self {
            name,
            executable,
            cgroup,
            netns,
            veth_pair,
        }
    }

    pub fn execute(mut self) -> Result<ContainerResult, ContainerError> {
        // create the cgroup
        if let Err(e) = self.cgroup.create() {
            return Err(ContainerError::CGroupCreationFailed(e));
        }
        info!("created cgroup {}", self.cgroup.name());

        // set veth up
        self.veth_pair.setup().unwrap();

        unsafe {
            // stack
            let stack = libc::mmap(
                std::ptr::null_mut::<c_void>(),
                STACK_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_STACK,
                -1,
                0,
            );

            // flags
            let flags = libc::SIGCHLD;

            // arg
            let pid = libc::clone(
                cb,
                stack.byte_add(STACK_SIZE),
                flags,
                (&mut self) as *mut Self as *mut c_void,
            );
            info!("container started with PID {}", pid);

            // wait for pid
            let mut status: i32 = -1;
            info!("waiting for the container to finish...");
            let pid = libc::waitpid(pid, &mut status, 0);
            if pid <= 0 {
                return Err(ContainerError::Unknown(
                    io::Error::last_os_error().to_string(),
                ));
            }
            if status == 0 {
                return Ok(ContainerResult {});
            } else {
                return Err(ContainerError::Failed(status));
            }
        }
    }
}
