use std::{
    io::{self},
    process::{exit, Command},
};

use log::{error, info};

#[derive(Debug, Clone)]
pub struct NetNs {
    name: String,
}

impl NetNs {
    pub fn new(name: String) -> Self {
        let output = Command::new("ip")
            .args(["netns", "add", &name])
            .output()
            .unwrap();
        assert!(output.status.success());

        Self { name }
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
        let r = Command::new("ip")
            .args(["netns", "del", &self.name])
            .output()
            .unwrap();
        if r.status.success() {
            info!("deleted netns {}", self.name);
        } else {
            error!(
                "could not delete netns {}, err: {}",
                self.name,
                String::from_utf8(r.stderr).unwrap(),
            );
        }
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
    ) -> Result<Self, io::Error> {
        // create veth pair
        let output = Command::new("ip")
            .args([
                "link", "add", &name, "type", "veth", "peer", "name", &peer_name,
            ])
            .output()?;
        if !output.status.success() {
            error!(
                "could not create veth pair, err: {}",
                String::from_utf8(output.stderr).unwrap()
            );
            exit(1);
        }

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
            if !output.status.success() {
                error!(
                    "could not set netns for veth pair peer {}, err: {}",
                    &peer_name,
                    String::from_utf8(output.stderr).unwrap()
                );
                exit(1);
            }
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
    pub fn setup(&self) -> Result<(), io::Error> {
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
