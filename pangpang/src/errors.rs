




#[derive(Debug)]
pub enum Error {
    StdIoError(std::io::Error),
    SshError(thrussh::Error),
    PpStreamError(String),
}


impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::StdIoError(e)
    }
}

impl From<thrussh::Error> for Error {
    fn from(e: thrussh::Error) -> Self {
        Self::SshError(e)
    }
}

impl Into<std::io::Error> for Error {
    fn into(self) -> std::io::Error {
        std::io::ErrorKind::Other.into()
    }
}
