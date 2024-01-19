use std::{
    fs,
    io::{self, Write},
    path::{self},
};

use libc::pid_t;
use log::info;

static CG_ROOT: &str = "/sys/fs/cgroup";

#[derive(Debug, Clone)]
pub struct CGroup {
    name: String,
    memory_limit: Option<usize>,
    memory_swap_limit: Option<usize>,
}

impl CGroup {
    pub fn new(
        name: String,
        memory_limit: Option<usize>,
        memory_swap_limit: Option<usize>,
    ) -> Self {
        Self {
            name,
            memory_limit: memory_limit,
            memory_swap_limit: memory_swap_limit,
        }
    }

    pub fn name(&self) -> String {
        return self.name.clone();
    }

    pub fn create(&self) -> Result<(), io::Error> {
        let root = path::Path::new(CG_ROOT);
        fs::create_dir(root.join(&self.name))?;

        // memory limits
        if let Some(memory_limit) = self.memory_limit {
            fs::File::create(root.join(&self.name).join("memory.max"))?
                .write_all(memory_limit.to_string().as_bytes())?;
        }
        if let Some(memory_swap_limit) = self.memory_swap_limit {
            fs::File::create(root.join(&self.name).join("memory.swap.max"))?
                .write_all(memory_swap_limit.to_string().as_bytes())?;
        }
        fs::File::create(root.join(&self.name).join("memory.oom.group"))?
            .write_all("1".as_bytes())?;

        Ok(())
    }

    pub fn write_pid(&self, pid: pid_t) -> Result<(), io::Error> {
        let root = path::Path::new(CG_ROOT);
        fs::File::create(root.join(&self.name).join("cgroup.procs"))?
            .write_all(pid.to_string().as_bytes())
    }
}

impl Drop for CGroup {
    fn drop(&mut self) {
        let root = path::Path::new(CG_ROOT);
        let _ = fs::remove_dir(root.join(&self.name));
        info!("deleted cgroup {}", self.name);
    }
}
