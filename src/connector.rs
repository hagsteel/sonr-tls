use native_tls::TlsConnector as NativeTlsConnector;
use native_tls::{HandshakeError, MidHandshakeTlsStream};
use sonr::prelude::*;
use sonr::{Evented, Token};
use std::collections::HashMap;
use std::io::{Read, Write};

use crate::TlsStream;

pub struct TlsConnector<S: Evented + Read + Write> {
    handshakes: HashMap<Token, MidHandshakeTlsStream<Stream<S>>>,
}

impl<S> TlsConnector<S>
where
    S: Evented + Read + Write,
{
    pub fn new() -> Self {
        Self {
            handshakes: HashMap::new(),
        }
    }
}

impl<S: Read + Write> Reactor for TlsConnector<S>
where
    S: Evented + Read + Write,
{
    type Output = TlsStream<Stream<S>>;
    type Input = (String, Stream<S>);

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Value((domain, stream)) => {
                let connector = NativeTlsConnector::new().unwrap();
                let res = connector.connect(&domain, stream);
                match res {
                    Ok(tls_stream) => return Reaction::Value(TlsStream::new(tls_stream)),
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
