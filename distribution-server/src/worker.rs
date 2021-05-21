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
            None,
            Some("tester"),
            None,
        )
        .unwrap();
        Worker {
            container: container,
        }
    }

    pub fn ipv4(&self) -> Ipv4Addr {
        let ip_call = Command::new("lxc")
            .args(&["exec", self.container.name(), "--", "hostname", "-I"])
            .output()
            .unwrap();

        let ip_output = String::from_utf8(ip_call.stdout).unwrap();
        let regex = Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
        let captures = regex.captures(&ip_output).unwrap();
        let ip = captures.get(0).unwrap().as_str();
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
        // Hack to wait for network up and running: Laeuft bisher auch nicht so richtig.
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
            .expect(format!("dhcclient of {} is not working", self.container.name()).as_str());
        // TODO: Check needed for systemd service/tester rocket server started in container
        // Vielleicht in dem man eine Testanfrage an diesen schickt.
    }
}
