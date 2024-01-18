use std::{
    fs,
    io::{self, Read, Write},
    path::{self},
};

use libc::pid_t;

pub enum Bytes {
    GB,
    MB,
    KB,
}

impl std::ops::Mul<usize> for Bytes {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        let bytes: usize = self.into();
        rhs * bytes
    }
}

impl Into<usize> for Bytes {
    fn into(self) -> usize {
        match self {
            Bytes::GB => 1000_000_000,
            Bytes::MB => 1000_000,
            Bytes::KB => 1000,
        }
    }
}

static CG_ROOT: &str = "/sys/fs/cgroup";

#[derive(Debug, Clone)]
pub struct CGroup {
    name: String,
    memory_limit: Option<usize>,
    memory_swap_limit: Option<usize>,
}

fn random_hex_encoded_string() -> String {
    let mut random = fs::File::open("/dev/random").unwrap();
    let mut buf: [u8; 16] = [0; 16];
    random.read_exact(&mut buf).unwrap();

    return hex::encode(&buf);
}

impl CGroup {
    pub fn new(
        name: Option<String>,
        memory_limit: Option<usize>,
        memory_swap_limit: Option<usize>,
    ) -> Self {
        Self {
            name: name.unwrap_or(random_hex_encoded_string()),
            memory_limit: memory_limit,
            memory_swap_limit: memory_swap_limit,
        }
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

    pub fn remove(self) -> Result<(), io::Error> {
        let root = path::Path::new(CG_ROOT);
        fs::remove_dir(root.join(&self.name))
    }

    pub fn write_pid(&self, pid: pid_t) -> Result<(), io::Error> {
        let root = path::Path::new(CG_ROOT);
        fs::File::create(root.join(&self.name).join("cgroup.procs"))
            .expect("could not open cgroup.procs file")
            .write_all(pid.to_string().as_bytes())
    }
}
