use std::io::{self, Read, Write};
use sonr::{Evented, Poll, PollOpt, Ready, Token};
use sonr::net::stream::Stream;

impl<T: Evented + Read + Write> AsRef<Stream<T>> for TlsStream<Stream<T>> {
    fn as_ref(&self) -> &Stream<T> {
        self.0.get_ref()
    }
}

impl<T: Evented + Read + Write> AsMut<Stream<T>> for TlsStream<Stream<T>> {
    fn as_mut(&mut self) -> &mut Stream<T> {
        self.0.get_mut()
    }
}

pub struct TlsStream<T>(native_tls::TlsStream<T>);

impl<T> TlsStream<T> {
    pub fn new(inner: native_tls::TlsStream<T>) -> Self {
        Self(inner)
    } 
}

impl<T : Read + Write> TlsStream<T> {
    pub fn get_ref(&self) -> &T {
        self.0.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
}

impl<T : Read + Write + Evented> Evented for TlsStream<T> {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll.register(
            self.0.get_ref(),
            token,
            interest,
            opts
        )
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll.reregister(
            self.0.get_ref(),
            token,
            interest,
            opts
        )
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll.deregister(self.0.get_ref())
    }
}


impl<T: Read + Write> Read for TlsStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<T: Read + Write> Write for TlsStream<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
