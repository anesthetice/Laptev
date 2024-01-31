use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    HandshakeFailed(HandshakeFailedReason),
}
#[derive(Debug)]
pub enum HandshakeFailedReason {
    ServerOffline,
    UknownServer,
    KeyExchangeFailed,
    AuthenticationFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Self::HandshakeFailed(reason) => {
                use HandshakeFailedReason as HFR;
                match reason {
                    HFR::ServerOffline => "could not connect to server",
                    HFR::UknownServer => {
                        "could not retrieve the password to the server from configuration"
                    }
                    HFR::KeyExchangeFailed => {
                        "could not exchange cryptographic keys with the server"
                    }
                    HFR::AuthenticationFailed => {
                        "could not authenticate, password probably incorrect"
                    }
                }
            }
        }
    }
}
