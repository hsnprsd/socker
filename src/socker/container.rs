use std::{ffi::CString, io, os::raw::c_void};

use crate::cgroup::CGroup;
use libc::{self};
use log::info;

const STACK_SIZE: usize = 1000_000; // 1MB

pub struct Container {
    executable: String,
    cgroup: CGroup,
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

        container.cgroup.write_pid(0).unwrap();

        let argv = [prog.as_ptr()];
        libc::execv(prog.as_ptr(), argv.as_ptr())
    }
}

impl Container {
    pub fn new(executable: String, resource_limits: ResourceLimits) -> Self {
        Self {
            executable,
            cgroup: CGroup::new(
                resource_limits.memory_limit,
                resource_limits.memory_swap_limit,
            ),
        }
    }

    pub fn execute(mut self) -> Result<ContainerResult, ContainerError> {
        if let Err(e) = self.cgroup.create() {
            return Err(ContainerError::CGroupCreationFailed(e));
        }
        info!("created cgroup {}", self.cgroup.name());

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
            let flags = libc::SIGCHLD | libc::CLONE_NEWNET;

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
