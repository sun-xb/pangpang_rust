use std::collections::HashMap;

use crate::{profile::{Profile, Protocol}, session::ssh, errors};

use super::Storage;





pub struct MockStorage {
    map: HashMap<String, Profile>,
}
impl Default for MockStorage {
    fn default() -> Self {
        let mut s = Self {
            map: HashMap::new(),
        };

        let profile_1 = Profile {
            transport: None,
            protocol: Protocol::Ssh(ssh::Profile{
                password: "123456".to_string(),
                username: "root".to_string(),
                address: "localhost".to_string(),
                port: 22,
            }),
        };
        let profile_2 = Profile {
            transport: Some(profile_1.id()),
            protocol: Protocol::Ssh(ssh::Profile{
                password: "123456".to_string(),
                username: "root".to_string(),
                address: "localhost".to_string(),
                port: 8022,
            }),
        };

        s.put(profile_1).unwrap();
        s.put(profile_2).unwrap();
        
        s
    }
}

impl Storage for MockStorage {
    fn get(&self, id: &String) -> Result<&Profile, errors::Error> {
        self.map.get(id).ok_or(errors::Error::ProfileNotFound(id.to_owned()))
    }

    fn put(&mut self, profile: Profile) -> Result<(), errors::Error>{
        let id = profile.id();
        self.map.insert(id, profile);
        Ok(())
    }

    fn remove(&mut self, id: &String) -> Result<Profile, errors::Error> {
        self.map.remove(id).ok_or(errors::Error::ProfileNotFound(id.to_string()))
    }

    fn flush(&self) -> Result<(), crate::errors::Error> {
        Ok(())
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Profile> + '_> {
        Box::new(self.map.values())
    }

}

