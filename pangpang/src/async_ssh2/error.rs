

#[derive(Debug)]
pub enum Error {
    SSH2(ssh2::Error),
    IO(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SSH2(e) => e.fmt(f),
            Self::IO(e) => e.fmt(f)
        }
    }
}

impl std::error::Error for Error {}

impl From<ssh2::Error> for Error {
    fn from(e: ssh2::Error) -> Self {
        Self::SSH2(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl Into<std::io::Error> for Error {
    fn into(self) -> std::io::Error {
        match self {
            Self::SSH2(e) => std::io::Error::from(e),
            Self::IO(e) => e
        }
    }
}




