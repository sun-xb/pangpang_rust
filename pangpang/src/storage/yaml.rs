use std::str::FromStr;




pub struct YamlStorage {
    path: std::path::PathBuf,
    map: std::collections::HashMap<String, crate::profile::Profile>,
}

impl Default for YamlStorage {
    fn default() -> Self {
        let path = std::path::PathBuf::from_str("./config.yaml").unwrap();
        let reader = std::fs::File::open(path.as_path()).unwrap();
        let mut map: std::collections::HashMap<String, crate::profile::Profile> = Default::default();
        let list: Vec<crate::profile::Profile> = serde_yaml::from_reader(reader).unwrap();
        list.into_iter().for_each(|x| { map.insert(x.id(), x); });
        Self {
            path,
            map
        }
    }
}

impl super::Storage for YamlStorage {
    fn get(&self, id: &String) -> Result<&crate::profile::Profile, crate::errors::Error> {
        self.map.get(id).ok_or(crate::errors::Error::ProfileNotFound(id.to_owned()))
    }

    fn put(&mut self, profile: crate::profile::Profile) -> Result<(), crate::errors::Error> {
        self.map.insert(profile.id(), profile);
        self.flush()
    }

    fn remove(&mut self, id: &String) -> Result<crate::profile::Profile, crate::errors::Error> {
        let removed = self.map.remove(id).ok_or(crate::errors::Error::ProfileNotFound(id.to_string()))?;
        self.flush()?;
        Ok(removed)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &crate::profile::Profile> + '_> {
        Box::new(self.map.values())
    }

    fn flush(&self) -> Result<(), crate::errors::Error>{
        let writer = std::fs::File::create(self.path.as_path())?;
        let mut output: Vec<&crate::profile::Profile> = Vec::new();
        for x in self.map.values() {
            output.push(x);
        }
        serde_yaml::to_writer(writer, &output).unwrap();
        Ok(())
    }
}