

use thrussh_keys::key;


pub(crate) struct PpSshHandler;

impl thrussh::client::Handler for PpSshHandler {
    type Error = crate::errors::Error;

    type FutureBool = std::future::Ready<Result<(Self, bool), Self::Error>>;

    type FutureUnit = std::future::Ready<Result<(Self, thrussh::client::Session), Self::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        std::future::ready(Ok((self, b)))
    }

    fn finished(self, session: thrussh::client::Session) -> Self::FutureUnit {
        std::future::ready(Ok((self, session)))
    }

    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        println!("server public key {:?}", server_public_key);
        self.finished_bool(true)
    }

    fn channel_open_forwarded_tcpip(self, _channel: thrussh::ChannelId, connected_address: &str, connected_port: u32, originator_address: &str, originator_port: u32, session: thrussh::client::Session) -> Self::FutureUnit {
        println!("new channel: {} {} {} {}", connected_address, connected_port, originator_address, originator_port);
        self.finished(session)
    }

}

