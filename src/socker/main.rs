mod cgroup;
mod container;

use cgroup::Bytes;
use clap::Parser;
use container::{Container, ResourceLimits};
use log::{error, info};
use std::process::exit;

use crate::container::ContainerError;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    executable: String,

    #[arg(short, long)]
    memory_limit: Option<Bytes>,
    #[arg(long)]
    memory_swap_limit: Option<Bytes>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    info!("executing {} in a container", args.executable);

    let resource_limits = ResourceLimits {
        memory_limit: args.memory_limit.map(|b| b._bytes),
        memory_swap_limit: args.memory_swap_limit.map(|b| b._bytes),
    };
    let container = Container::new(args.executable, resource_limits);
    let result = container.execute();
    match result {
        Ok(_) => info!("container finished successfully :)"),
        Err(ContainerError::Failed(exit_code)) => {
            error!("container failed, exit code: {}", exit_code);
            exit(1);
        }
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}
