use crate::{profile::{Profile, Protocol}, errors, ssh::SshProfile};



pub trait Storage {
    fn get(&self, id: &String) -> Result<Profile, errors::Error>;
}

pub struct MockStorage;
impl MockStorage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Storage for MockStorage {
    fn get(&self, id: &String) -> Result<Profile, errors::Error> {
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
        

        if id == &profile_1.id() {
            Ok(profile_1)
        } else if id == &profile_2.id() {
            Ok(profile_2)
        } else {
            Err(errors::Error::ProfileNotFound(id.to_owned()))
        }
    }
}
