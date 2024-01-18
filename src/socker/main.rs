mod cgroup;
mod container;

use container::{Container, ResourceLimits};
use log::info;
use std::env;

fn main() {
    env_logger::init();

    let mut args = env::args();
    args.next();
    let program = args.next().expect("Usage: [program]");

    info!("executing {} in a container", program);

    let resource_limits = ResourceLimits {
        memory_limit: Some(512_000_000),
        memory_swap_limit: None,
    };
    let container = Container::new(program, resource_limits);
    let _ = container.execute().expect("execution of container failed");
}
