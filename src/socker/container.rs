use std::{
    ffi::CString,
    fs,
    io::{self, Read},
    os::fd::AsRawFd,
};

use libc::{self};
use log::info;

use crate::{
    cgroup::CGroup,
    network::{NetNs, VETHPair},
};

#[derive(Debug)]
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

pub struct ContainerResult {
    pub status: i32,
}

fn random_hex_encoded_string() -> io::Result<String> {
    let mut random = fs::File::open("/dev/random")?;
    let mut buf: [u8; 5] = [0; 5];
    random.read_exact(&mut buf)?;

    return Ok(hex::encode(&buf));
}

impl Container {
    pub fn new(executable: String, resource_limits: ResourceLimits) -> io::Result<Self> {
        let id = random_hex_encoded_string()?;
        let name = format!("container-{}", id);

        let cgroup = CGroup::new(
            format!("{}.slice", &name),
            resource_limits.memory_limit,
            resource_limits.memory_swap_limit,
        );

        let netns = NetNs::new(format!("{}", name))?;

        let veth_pair = VETHPair::new(
            format!("v-{}", &id),
            None,
            format!("v-p-{}", &id),
            String::from("10.10.0.2/16"),
            Some(netns.name()),
        )?;

        Ok(Self {
            name,
            executable,
            cgroup,
            netns,
            veth_pair,
        })
    }

    pub fn execute(self) -> io::Result<ContainerResult> {
        // create the cgroup
        self.cgroup.create()?;
        info!("created cgroup {}", self.cgroup.name());

        // set veth up
        self.veth_pair.setup()?;

        let pid;
        unsafe {
            pid = libc::fork();
        }
        if pid == 0 {
            let program = CString::new(self.executable)?;

            self.cgroup.write_pid(0)?;

            // setns network
            let fd = fs::File::open(self.netns.path())?;
            unsafe {
                libc::setns(fd.as_raw_fd(), libc::CLONE_NEWNET);
            }

            self.veth_pair.setup_peer()?;

            let argv = [program.as_ptr(), std::ptr::null()];
            println!("{:?}", program);
            println!("{:?}", argv);
            unsafe {
                let _ = libc::execv(program.as_ptr(), argv.as_ptr());
            }

            return Err(io::Error::last_os_error());
        } else {
            info!("container started with PID {}", pid);
            // wait for pid
            let mut status: i32 = -1;
            info!("waiting for the container to finish...");
            unsafe {
                let rc = libc::waitpid(pid, &mut status, 0);
                if rc < 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            // TODO: check pid < 0
            return Ok(ContainerResult { status });
        }
    }
}
