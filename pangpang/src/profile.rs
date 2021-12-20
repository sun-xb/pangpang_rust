use crate::ssh;




bitflags::bitflags! {
    pub struct Capacity: u32 {
        const SESSION_CACHE = 0b0000_0000_0000_0000_0001;
        const OPEN_PTY = 0b0000_0000_0000_0000_0010;
        const OPEN_TUNNEL   = 0b0000_0000_0000_0000_0100;
    }
}
pub enum Protocol {
    Ssh(ssh::SshProfile),
}

pub struct Profile {
    pub username: String,
    pub address: String,
    pub port: u16,
    pub transport: Option<String>,
    pub protocol: Protocol,
}

impl Profile {
    pub fn id(&self) -> String {
        format!("{}@{}:{}", self.username, self.address, self.port)
    }
    pub fn capacity(&self) -> Capacity {
        match self.protocol {
            Protocol::Ssh(_) => Capacity::all(),
        }
    }
}


