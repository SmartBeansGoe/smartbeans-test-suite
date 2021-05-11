extern crate regex;

use lxd::{Container, Location};
use regex::Regex;
use std::net::Ipv4Addr;
use std::process::Command;
use std::str::FromStr;

pub struct Worker {
    pub container: Container,
}

impl Worker {
    pub fn new(location: Location, name: &str, base: &str) -> Worker {
        Command::new("lxc")
            .args(&["delete", "--force", &name])
            .status()
            .expect(format!("Err during destroying container: {}", &name).as_str());
        let container = Container::new(
            location,
            name,
            base,
            Some(true),
            Some("lxdbr0"),
            Some("tester"),
            None,
        )
        .unwrap();
        Worker {
            container: container,
        }
    }

    pub fn ipv4(&self) -> Ipv4Addr {
        let info = Command::new("lxc")
            .args(&["info", self.container.name()])
            .output()
            .unwrap();

        let ip = String::from_utf8(info.stdout).unwrap();
        let re = Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
        let caps = re.captures(&ip).unwrap();
        let ip = caps.get(0).unwrap().as_str();
        Ipv4Addr::from_str(&ip).unwrap()
    }

    pub fn profile(&self, name: &str) {
        Command::new("lxc")
            .args(&["profile", "remove", self.container.name(), "default"])
            .status()
            .expect("Error when removing default profile");
        Command::new("lxc")
            .args(&["profile", "assign", self.container.name(), name])
            .status()
            .expect(format!("Error when assigning {} profile", name).as_str());
    }

    pub fn snapshot(&self, name: &str) {
        Command::new("lxc")
            .args(&["snapshot", self.container.name(), name])
            .status()
            .expect(format!("Error when creating snapshot {}", name).as_str());
    }

    pub fn restore(&self, snapshot: &str) {
        Command::new("lxc")
            .args(&["restore", self.container.name(), snapshot])
            .status()
            .expect(format!("Error when restoring snapshot {}", snapshot).as_str());
        // Hack to wait for network up and running

        Command::new("lxc")
            .args(&[
                "exec",
                self.container.name(),
                "--mode=non-interactive",
                "-n",
                "--",
                "dhclient",
            ])
            .status()
            .expect(format!(
                "dhcclient of {} is not working",
                self.container.name()
            ).as_str());
    }
}
