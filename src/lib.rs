use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::Arc;

use sonr::errors::Error as SonrError;
use sonr::net::stream::Stream;
use sonr::reactor::{Reaction, Reactor};

use native_tls::{HandshakeError, Identity, MidHandshakeTlsStream, TlsAcceptor};
use sonr::{Evented, Token};

mod stream;

pub use crate::stream::TlsStream;

pub struct ReactiveTlsAcceptor<S>
where
    S: Evented + Read + Write,
{
    acceptor: Arc<TlsAcceptor>,
    handshakes: HashMap<Token, MidHandshakeTlsStream<Stream<S>>>,
}

impl<S> ReactiveTlsAcceptor<S>
where
    S: Evented + Read + Write,
{
    pub fn new(cert_path: &str, cert_pass: &str) -> Result<Self, Error> {
        let acceptor = acceptor(cert_path, cert_pass)?;
        Ok(Self {
            acceptor,
            handshakes: HashMap::new(),
        })
    }
}

impl<S> Reactor for ReactiveTlsAcceptor<S>
where
    S: Evented + Read + Write,
{
    type Output = TlsStream<Stream<S>>;
    type Input = Stream<S>;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Value(stream) => {
                match self.acceptor.accept(stream) {
                    Ok(stream) => return Reaction::Value(TlsStream::new(stream)),
                    Err(HandshakeError::WouldBlock(stream)) => {
                        self.handshakes.insert(stream.get_ref().token(), stream);
                        return Reaction::Continue;
                    }
                    Err(_e) => {
                        return Reaction::Continue; /* Let the connections drop on error for now */
                    }
                }
            }
            Reaction::Event(event) => {
                if let Some(stream) = self.handshakes.remove(&event.token()) {
                    match stream.handshake() {
                        Ok(stream) => return Reaction::Value(TlsStream::new(stream)),
                        Err(HandshakeError::WouldBlock(stream)) => {
                            self.handshakes.insert(stream.get_ref().token(), stream);
                            Reaction::Continue
                        }
                        Err(_e) => {
                            Reaction::Continue /* Let the connections drop on error for now */
                        }
                    }
                } else {
                    Reaction::Event(event)
                }
            }
            Reaction::Continue => Reaction::Continue,
        }
    }
}


fn acceptor(path: &str, pass: &str) -> Result<Arc<TlsAcceptor>, Error> {
    let mut file = File::open(path)?;
    let mut identity = Vec::new();
    file.read_to_end(&mut identity)?;
    let identity = Identity::from_pkcs12(&identity, pass)?;

    let acceptor = TlsAcceptor::new(identity)?;
    Ok(Arc::new(acceptor))
}


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
