mod cgroup;
mod container;
mod network;

use clap::Parser;
use container::{Container, ResourceLimits};
use log::info;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    executable: String,

    #[arg(short, long)]
    memory_limit: Option<usize>,
    #[arg(long)]
    memory_swap_limit: Option<usize>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    // setup sigint handler
    ctrlc::set_handler(|| panic!("SIGINT")).unwrap();

    let resource_limits = ResourceLimits {
        memory_limit: args.memory_limit,
        memory_swap_limit: args.memory_swap_limit,
    };
    let container = Container::new(args.executable, resource_limits).unwrap();
    let result = container.execute().unwrap();
    info!("container finished, status: {}", result.status);
}
