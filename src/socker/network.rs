use std::{
    io::{self},
    process::Command,
};

use log::info;

#[derive(Debug, Clone)]
pub struct NetNs {
    name: String,
}

impl NetNs {
    pub fn new(name: String) -> io::Result<Self> {
        let output = Command::new("ip").args(["netns", "add", &name]).output()?;
        assert!(output.status.success());

        Ok(Self { name })
    }

    pub fn path(&self) -> String {
        format!("/run/netns/{}", self.name)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Drop for NetNs {
    fn drop(&mut self) {
        let output = Command::new("ip")
            .args(["netns", "del", &self.name])
            .output()
            .unwrap();
        assert!(output.status.success());
    }
}

pub struct VETHPair {
    pub name: String,
    netns: Option<String>,
    pub peer_name: String,
    peer_addr: String,
    peer_netns: Option<String>,
}

impl VETHPair {
    pub fn new(
        name: String,
        netns: Option<String>,
        peer_name: String,
        peer_addr: String,
        peer_netns: Option<String>,
    ) -> io::Result<Self> {
        // create veth pair
        let output = Command::new("ip")
            .args([
                "link", "add", &name, "type", "veth", "peer", "name", &peer_name,
            ])
            .output()?;
        assert!(output.status.success());

        // setns on veth pair
        if let Some(netns) = &netns {
            let output = Command::new("ip")
                .args(["link", "set", "dev", &name, "netns", &netns])
                .output()?;
            assert!(output.status.success());
        }
        if let Some(netns) = &peer_netns {
            let output = Command::new("ip")
                .args(["link", "set", "dev", &peer_name, "netns", &netns])
                .output()?;
            assert!(output.status.success());
        }

        Ok(Self {
            name,
            netns,
            peer_name,
            peer_addr,
            peer_netns,
        })
    }

    // sets ip addr, set up
    pub fn setup(&self) -> io::Result<()> {
        // set master socker0
        let output = Command::new("ip")
            .args(["link", "set", &self.name, "master", "socker0"])
            .output()?;
        assert!(output.status.success());

        let output = Command::new("ip")
            .args(["link", "set", "dev", &self.name, "up"])
            .output()?;
        assert!(output.status.success());
        Ok(())
    }

    // sets ip addr, set up
    pub fn setup_peer(&self) -> Result<(), io::Error> {
        let output = Command::new("ip")
            .args(["addr", "add", &self.peer_addr, "dev", &self.peer_name])
            .output()?;
        assert!(output.status.success());
        let output = Command::new("ip")
            .args(["link", "set", "dev", &self.peer_name, "up"])
            .output()?;
        assert!(output.status.success());
        Ok(())
    }
}

impl Drop for VETHPair {
    fn drop(&mut self) {
        let _ = Command::new("ip")
            .args(["link", "del", &self.name])
            .output()
            .unwrap();

        let _ = Command::new("ip")
            .args(["link", "del", &self.peer_name])
            .output()
            .unwrap();

        info!("deleted veth pair {} peer {}", self.name, self.peer_name);
    }
}
