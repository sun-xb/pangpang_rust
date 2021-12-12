


pub(crate) trait Storage {
    fn write(&mut self);
    fn read(&self);
}

pub(crate) struct MockStorage;


impl Storage for MockStorage {
    fn write(&mut self) {
        todo!()
    }

    fn read(&self) {
        todo!()
    }
}
