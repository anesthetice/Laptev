use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Forbidden,
    HandshakeFailed(HandshakeFailedReason),
    InvalidSocketAddr,
    ServerNotResponding,
}
#[derive(Debug, Clone)]
pub enum HandshakeFailedReason {
    ServerNotResponding,
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
            Self::Forbidden => "not authenticated to server",
            Self::HandshakeFailed(reason) => {
                use HandshakeFailedReason as HFR;
                match reason {
                    HFR::ServerNotResponding => {
                        "could not connect to server"
                    },
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
            },
            Self::InvalidSocketAddr => "not a valid socket addr",
            Self::ServerNotResponding => "could not connect to server",
        }
    }
}
