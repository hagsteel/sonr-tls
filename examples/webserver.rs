use sonr::net::stream::Stream;
use sonr::net::tcp::ReactiveTcpListener;
use sonr::net::tcp::TcpListener;
use sonr::reactor::{Reaction, Reactive};
use sonr::sync::queue;
use sonr::system::System;
use sonr::{Event, Evented, Token};
use sonr_tls::{ReactiveTlsAcceptor, TlsStream};
use std::collections::HashMap;
use std::env;
use std::io::{ErrorKind::WouldBlock, Read, Write};
use std::thread;

static RESPONSE: &'static [u8] = b"HTTP/1.1 200 OK\nContent-Length: 13\n\nHello World\n\n";

struct TlsConnections<S: Evented + Read + Write> {
    connections: HashMap<Token, TlsStream<Stream<S>>>,
}

impl<S> Reactive for TlsConnections<S>
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

                    if s.get_mut().readable() {
                        let mut buf = [0; 1024];
                        let res = s.read(&mut buf[..]);
                    }

                    while s.get_ref().writable() {
                        let res = s.write(&RESPONSE);
                        match res {
                            Ok(_) => {}
                            Err(ref e) if e.kind() == WouldBlock => { break }
                            Err(_e) => {
                                self.connections.remove(&event.token());
                                break
                            }
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
    let listener = ReactiveTcpListener::bind("0.0.0.0:5555").unwrap();
    let mut queue = queue::ReactiveQueue::unbounded();

    for _ in 0..8 {
        let stealer = queue.deque();
        thread::spawn(move || {
            System::init();
            let stealer = queue::ReactiveDeque::new(stealer).unwrap();
            let path = env::var("SONR_PFX_PATH").unwrap();
            let pass = env::var("SONR_PFX_PASS").unwrap();
            let acceptor = ReactiveTlsAcceptor::new(&path, &pass).unwrap();
            let connections = TlsConnections {
                connections: HashMap::new(),
            };
            let server = stealer.chain(acceptor.chain(connections));
            System::start(server);
        });
    }

    let server = listener.map(|(s, _)| s).chain(queue);
    System::start(server);
}
