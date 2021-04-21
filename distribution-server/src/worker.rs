extern crate regex;

use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct Worker {
    pub name: String,
}

enum Status {
    Running,
    Stopped,
}

impl Worker {
    pub fn new(name: &str) -> Worker {
        // lxc::create(name);
        Worker {
            name: name.to_string(),
        }
    }

    pub fn load(name: &str) -> Worker {
        Worker {
            name: name.to_string(),
        }
    }

    pub fn from_image(image: &str, name: &str) -> Worker {
        // lxc::create(name, image);
        // Worker {
        //     name: name.to_string(),
        // }
        unimplemented!("Initialize new container from image is not implemented yet!");
    }

    pub fn copy(&self, name: &str) -> Worker {
        // TODO: needs a check for already existing
        lxc::copy(self.name.as_str(), name);
        Worker {
            name: name.to_string(),
        }
    }

    pub fn info(&self) {
        lxc::info(self.name.as_str());
    }

    pub fn ip(&self) -> IpAddr {
        lxc::ip(self.name.as_str())
    }

    pub fn start(&self) {
        lxc::start(self.name.as_str());
    }

    pub fn stop(&self) {
        lxc::stop(self.name.as_str());
    }

    pub fn reset(&self) {
        unimplemented!("This method is not implemented yet!");
    }
}

pub mod lxc {
    use regex::Regex;
    use std::net::IpAddr;
    use std::process::Command;
    use std::str::FromStr;

    pub fn create(name: &str) {
        unimplemented!("Create is not implemented yet.");
    }

    pub fn copy(from: &str, to: &str) {
        // needs a check: from must be stopped
        Command::new("lxc-copy")
            .args(&["-n", from])
            .args(&["-N", to])
            .status()
            .expect(format!("LXC Error: Failed to copy from {} to {}", from, to).as_str());
    }
    pub fn destroy(name: &str) {
        Command::new("lxc-destroy")
            .args(&["-n", name])
            .status()
            .expect(format!("LXC Error: Failed to destroy '{}'", name).as_str());
    }
    pub fn start(name: &str) {
        Command::new("lxc-start")
            .args(&["-n", name, "-d"])
            .status()
            .expect(format!("LXC Error: Failed to start '{}'", name).as_str());
    }
    pub fn stop(name: &str) {
        Command::new("lxc-stop")
            .args(&["-n", name])
            .status()
            .expect(format!("LXC Error: Failed to stop '{}'", name).as_str());
    }
    pub fn state(name: &str) {
        let state = Command::new("lxc-info").args(&["-n", name, "-s"]).output();
        print!("{}", String::from_utf8(state.unwrap().stdout).unwrap())
    }
    pub fn ip(name: &str) -> IpAddr {
        // TODO: If the ip address is not available it crashes.
        let state = Command::new("lxc-info").args(&["-n", name, "-i"]).output();
        let ip = String::from_utf8(state.unwrap().stdout).unwrap();
        let re = Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
        let caps = re.captures(&ip).unwrap();
        let ip = caps.get(0).unwrap().as_str();
        IpAddr::from_str(&ip).unwrap()
    }
    pub fn info(name: &str) {
        Command::new("lxc-info").output();
    }

    pub fn reset(name: &str) {
        unimplemented!();
    }
}
