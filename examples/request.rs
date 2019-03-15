use std::collections::HashMap;
use std::io::{Read, Write};
use sonr::{Evented, Token};
use sonr::prelude::*;
use sonr::reactor::producers::Mono;
use sonr::net::tcp::TcpStream;
use sonr_tls::TlsStream;
use sonr_tls::TlsConnector;

struct TlsConnections<S: Evented + Read + Write> {
    connections: HashMap<Token, TlsStream<Stream<S>>>,
}

impl<S> Reactor for TlsConnections<S>
where
    S: Evented + Read + Write,
{
    type Input = TlsStream<Stream<S>>;
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Value(stream) => {
                self.connections.insert(stream.get_ref().token(), stream);
                Reaction::Continue
            }
            Reaction::Event(event) => {
                if self.connections.get(&event.token()).is_none() {
                    return Reaction::Event(event);
                }

                if let Some(s) = self.connections.get_mut(&event.token()) {
                    let _ = s.get_mut().react(Reaction::Event(event));

                    if s.get_mut().writable() {
                        &mut s.write(b"GET / HTTP/1.0\r\n\r\n");
                    }

                    if s.get_mut().readable() {
                        let mut buf = [0; 1024];
                        if let Ok(n) =s.read(&mut buf[..]) {
                            eprintln!("{}", String::from_utf8_lossy(&buf[..n]).to_string());
                        }
                    }
                }

                Reaction::Continue
            }
            Reaction::Continue => Reaction::Continue,
        }
    }
}

fn main() {
    System::init();
    let stream = TcpStream::connect(&"172.217.20.46:443".parse().unwrap()).unwrap();
    let mono = Mono::new(stream).unwrap().map(|s| ("google.com".to_owned(), Stream::new(s).unwrap()));
    let connections = TlsConnections { connections: HashMap::new() };
    let connector = TlsConnector::new().chain(connections);

    let run = mono.chain(connector);

    System::start(run);
}
