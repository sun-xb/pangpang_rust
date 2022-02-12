




#[derive(Debug)]
pub enum Error {
    StdIoError(std::io::Error),
    SshError(thrussh::Error),
    PpStreamError(String),
    WritePtyError(String),
    ReadPtyError(String),
    ProfileNotFound(String),
    UrlParseError(String),
    HttpProxyConnectionLost,
    HttpProxyServerError(String),
    HttpProxyRequestError(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl std::error::Error for Error {}

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

