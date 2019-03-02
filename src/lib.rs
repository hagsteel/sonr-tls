use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;

use sonr::reactor::{Reactive, Reaction};
use sonr::net::stream::Stream;
use sonr::errors::Result;

use native_tls::{Identity, TlsAcceptor, HandshakeError, MidHandshakeTlsStream};
use sonr::{Event, Evented, Token};

pub use native_tls::TlsStream;

pub struct ReactiveTlsAcceptor<S> 
    where S: Evented + Read + Write
{
    acceptor: Arc<TlsAcceptor>,
    handshakes: HashMap<Token, MidHandshakeTlsStream<Stream<S>>>,
    ready_streams: VecDeque<TlsStream<Stream<S>>>,
}

impl<S> ReactiveTlsAcceptor<S> 
    where S: Evented + Read + Write
{
    pub fn new(cert_path: &str, cert_pass: &str) -> Result<Self> {
        let acceptor = acceptor(cert_path, cert_pass)?;
        Ok(Self {
            acceptor,
            handshakes: HashMap::new(),
            ready_streams: VecDeque::new(),
        })
    }
}

impl<S> Reactive for ReactiveTlsAcceptor<S> 
    where S: Evented + Read + Write
{
    type Output = TlsStream<Stream<S>>;
    type Input = S;

    fn react_to(&mut self, stream: Self::Input) {
        if let Ok(stream) = Stream::new(stream) {
            match self.acceptor.accept(stream) {
                Ok(stream) => self.ready_streams.push_back(stream),
                Err(HandshakeError::WouldBlock(stream)) => {
                    self.handshakes.insert(stream.get_ref().token(), stream);
                }
                Err(_e) => { /* Let the connections drop on error for now */ },
            }
        }
    }

    fn reacting(&mut self, event: Event) -> bool {
        if let Some(stream) = self.handshakes.remove(&event.token()) {
            match stream.handshake() {
                Ok(stream) => self.ready_streams.push_back(stream),
                Err(HandshakeError::WouldBlock(stream)) => {
                    self.handshakes.insert(stream.get_ref().token(), stream);
                }
                Err(_e) => { /* Let the connections drop on error for now */ },
            }

            true
        } else { 
            false 
        }
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        match self.ready_streams.pop_front() {
            Some(stream) => Reaction::Value(stream),
            None => Reaction::NoReaction
        }
    }
}

fn acceptor(path: &str, pass: &str) -> Result<Arc<TlsAcceptor>> {
    let mut file = File::open(path)?;
    let mut identity = Vec::new();
    file.read_to_end(&mut identity)?;
    let identity = Identity::from_pkcs12(&identity, pass).unwrap();

    let acceptor = TlsAcceptor::new(identity).unwrap();
    Ok(Arc::new(acceptor))
}
