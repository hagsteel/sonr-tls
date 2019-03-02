use std::io::{Read, Write, ErrorKind::WouldBlock};
use std::env;
use std::thread;
use std::collections::HashMap;
use sonr::reactor::{Reactive, Reaction};
use sonr::system::System;
use sonr::net::tcp::ReactiveTcpListener;
use sonr::net::stream::Stream;
use sonr::sync::queue;
use sonr::{Token, Evented, Event};
use sonr::net::tcp::TcpListener;
use sonr_tls::{ReactiveTlsAcceptor, TlsStream};

static RESPONSE: &'static [u8] = b"HTTP/1.1 200 OK\nContent-Length: 13\n\nHello World\n\n";

struct TlsConnections<S: Evented + Read + Write> {
    connections: HashMap<Token, TlsStream<Stream<S>>>
}

impl<S> Reactive for TlsConnections<S> 
    where S: Evented + Read + Write
{
    type Input = TlsStream<Stream<S>>;
    type Output = ();

    fn reacting(&mut self, event: Event) -> bool {
        let mut removals = Vec::new();

        let reacting = self.connections.get_mut(&event.token()).map(|s| {
            let res = s.get_mut().reacting(event);

            if s.get_mut().readable() {
                let mut buf = [0; 1024];
                let res = s.read(&mut buf[..]);
            }

            while s.get_ref().writable() {
                let res =  s.write(&RESPONSE);
                match res {
                    Ok(_) => {}
                    Err(ref e) if e.kind() == WouldBlock => {}
                    Err(_e) => removals.push(s.get_ref().token())
                }
            }
        }).is_some();

        while let Some(token) = removals.pop() {
            self.connections.remove(&token);
        }

        reacting
    }

    fn react_to(&mut self, stream: Self::Input) {
        self.connections.insert(stream.get_ref().token(), stream);
    }

    fn react(&mut self) -> Reaction<()> {
        Reaction::NoReaction
    }
}

fn main() {
    System::init();
    let listener = ReactiveTcpListener::bind("0.0.0.0:5555").unwrap(); 
    let mut queue = queue::ReactiveQueue::unbounded();

    for _ in 0..7 {
        let stealer = queue.deque();
        thread::spawn(move || {
            System::init();
            let stealer = queue::ReactiveDeque::new(stealer).unwrap();
            let path = env::var("SONR_PFX_PATH").unwrap();
            let pass = env::var("SONR_PFX_PASS").unwrap();
            let acceptor = ReactiveTlsAcceptor::new(&path, &pass).unwrap();
            let connections = TlsConnections { connections: HashMap::new() }.map(|s| {
                let () = s;
                s
            });
            let server = stealer.chain(acceptor.chain(connections));
            System::start(server);
        });
    }
    
    let server = listener.map(|(s, _)| s).chain(queue);
    System::start(server);
}
