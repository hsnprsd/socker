use std::{
    ffi::{CStr, CString},
    io,
    os::raw::c_void,
    process::Command,
    thread::sleep,
    time::Duration,
};

use crate::cgroup::CGroup;
use libc::{self, c_char};
use log::{debug, info};

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
    Failed(i32),
    Unknown(String),
}

struct CBArg {
    prog: CString,
}

extern "C" fn cb(arg: *mut c_void) -> i32 {
    unsafe {
        let arg: CBArg = (arg as *mut CBArg).read();
        libc::execv(arg.prog.as_ptr(), [arg.prog.as_ptr()].as_ptr())
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

    pub fn execute(self) -> Result<ContainerResult, ContainerError> {
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
            let mut arg = CBArg {
                prog: CString::new(self.executable.clone()).unwrap(),
            };

            let pid = libc::clone(
                cb,
                stack.byte_add(STACK_SIZE),
                flags,
                (&mut arg) as *mut CBArg as *mut c_void,
            );
            debug!("container started with PID {}", pid);

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
