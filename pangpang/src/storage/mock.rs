

use crate::profile::ProfileId;




pub struct MockStorage(std::collections::HashMap<String, crate::profile::Profile>);

impl Default for MockStorage {
    fn default() -> Self {
        let mut map = std::collections::HashMap::new();
        let mut profile = crate::profile::Profile {
            transport: None,
            protocol: crate::profile::Protocol::SSH(crate::profile::SshProfile {
                address: String::from("localhost"),
                port: 22,
                username: String::from("sun"),
                host_key: Default::default(),
                auth_method: crate::profile::SshAuthMethod::Password(String::from("pangpang"))
            })
        };
        map.insert(profile.id(), profile.clone());
        profile.transport = Some(profile.id());
        let crate::profile::Protocol::SSH(ref mut ssh) = profile.protocol;
        ssh.address = String::from("localhost");
        ssh.port = 6022;
        ssh.username = String::from("root");
        ssh.auth_method = crate::profile::SshAuthMethod::Password(String::from("123456"));
        map.insert(profile.id(), profile);
        MockStorage(map)
    }
}

#[async_trait::async_trait]
impl super::Storage for MockStorage {
    async fn get(&self, id: &String) -> anyhow::Result<&crate::profile::Profile> {
        self.0.get(id).ok_or(anyhow::anyhow!("not found"))
    }

    async fn put(&mut self, profile: crate::profile::Profile) -> anyhow::Result<()> {
        self.0.insert(profile.id(), profile);
        Ok(())
    }

    async fn delete(&mut self, id: &String) -> anyhow::Result<crate::profile::Profile> {
        self.0.remove(id).ok_or(anyhow::anyhow!("not found"))
    }
}


