use serde::{Deserialize, Serialize};

use crate::session::{ssh, socks, telnet};




bitflags::bitflags! {
    pub struct Capacity: u32 {
        const SESSION_CACHE                 = 0b0000_0000_0000_0000_0001;
        const TERMINAL                      = 0b0000_0000_0000_0000_0010;
        const LOCAL_TUNNEL                  = 0b0000_0000_0000_0000_0100;
        const REMOTE_TUNNEL                 = 0b0000_0000_0000_0000_1000;
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Protocol {
    Ssh(ssh::Profile),
    Socks(socks::Profile),
    Telnet(telnet::Profile),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub transport: Option<String>,
    pub protocol: Protocol,
}

impl Profile {
    pub fn id(&self) -> String {
        match &self.protocol {
            Protocol::Ssh(profile) => format!("ssh://{}@{}:{}", profile.username, profile.address, profile.port),
            Protocol::Socks(profile) => format!("socks://{}:{}", profile.address, profile.port),
            Protocol::Telnet(_) => todo!(),
        }
    }
    pub fn capacity(&self) -> Capacity {
        match self.protocol {
            Protocol::Ssh(_) => Capacity::all(),
            Protocol::Socks(_) => Capacity::LOCAL_TUNNEL,
            Protocol::Telnet(_) => Capacity::TERMINAL,
        }
    }
}

