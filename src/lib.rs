use std::io;

use sonr::errors::Error as SonrError;

mod stream;
mod acceptor;
mod connector;

pub use crate::stream::TlsStream;
pub use crate::acceptor::TlsAcceptor;
pub use crate::connector::TlsConnector;


// -----------------------------------------------------------------------------
// 		- Error -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Sonr(SonrError),
    Tls(native_tls::Error),
}

impl From<SonrError> for Error {
    fn from(err: SonrError) -> Error {
        Error::Sonr(err)
    }
}

impl From<native_tls::Error> for Error {
    fn from(err: native_tls::Error) -> Error {
        Error::Tls(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
