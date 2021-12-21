use std::collections::HashMap;

use crate::{profile::{Profile, Protocol}, errors, session::ssh::SshProfile};



pub trait Storage: Send + Sync {
    fn get(&self, id: &String) -> Result<Profile, errors::Error>;
    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &Profile)> + '_>;
}

pub struct MockStorage {
    map: HashMap<String, Profile>,
}
impl MockStorage {
    pub fn new() -> Self {
        let mut s = Self {
            map: HashMap::new(),
        };

        let profile_1 = Profile {
            username: "root".to_string(),
            address: "localhost".to_string(),
            port: 22,
            transport: None,
            protocol: Protocol::Ssh(SshProfile{
                password: "123456".to_string(),
            }),
        };
        let profile_2 = Profile {
            username: "root".to_string(),
            address: "localhost".to_string(),
            port: 8022,
            transport: Some(profile_1.id()),
            protocol: Protocol::Ssh(SshProfile{
                password: "123456".to_string(),
            }),
        };

        s.map.insert(profile_1.id(), profile_1);
        s.map.insert(profile_2.id(), profile_2);
        
        s
    }
}

impl Storage for MockStorage {
    fn get(&self, id: &String) -> Result<Profile, errors::Error> {
        if let Some(p) = self.map.get(id) {
            Ok(p.clone())
        } else {
            Err(errors::Error::ProfileNotFound(id.to_owned()))
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &Profile)> + '_> {
        Box::new(self.map.iter())
    }
}
