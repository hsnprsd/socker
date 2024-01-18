mod cgroup;
mod container;

use container::{Container, ResourceLimits};
use log::{error, info};
use std::{env, process::exit};

use crate::container::ContainerError;

fn main() {
    env_logger::init();

    let mut args = env::args();
    args.next();
    let program = args.next().expect("Usage: [program]");

    info!("executing {} in a container", program);

    let resource_limits = ResourceLimits {
        memory_limit: Some(512_000_000),
        memory_swap_limit: Some(0),
    };
    let container = Container::new(program, resource_limits);
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
