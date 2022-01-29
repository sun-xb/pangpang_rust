
use crate::{profile::Profile, errors};

mod mock;
mod yaml;

pub use mock::MockStorage;
pub use yaml::YamlStorage;


pub trait Storage: Send + Sync {
    fn get(&self, id: &String) -> Result<&Profile, errors::Error>;
    fn put(&mut self, profile: Profile) -> Result<(), errors::Error>;
    fn remove(&mut self, id: &String) -> Result<Profile, errors::Error>;
    fn flush(&self) -> Result<(), errors::Error>;
    fn iter(&self) -> Box<dyn Iterator<Item = &Profile> + '_>;
}
