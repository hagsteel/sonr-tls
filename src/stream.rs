use sonr::net::stream::{Stream, StreamRef};
use sonr::{Evented, Poll, PollOpt, Ready, Token};
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read, Write};

pub struct TlsStream<T>(native_tls::TlsStream<T>);

impl<T> TlsStream<T> {
    pub fn new(inner: native_tls::TlsStream<T>) -> Self {
        Self(inner)
    }
}

impl<T> Debug for TlsStream<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Stream")
            .field("0", &self.0)
            .finish()
    }
}

impl<T: Read + Write> TlsStream<T> {
    pub fn get_ref(&self) -> &T {
        self.0.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
}

impl<T: Read + Write + Evented> Evented for TlsStream<T> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        poll.register(self.0.get_ref(), token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        poll.reregister(self.0.get_ref(), token, interest, opts)
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

impl<T: Evented + Read + Write> StreamRef for TlsStream<Stream<T>> {
    type Evented = T;

    fn stream_ref(&self) -> &Stream<T> {
        self.0.get_ref()
    }

    fn stream_mut(&mut self) -> &mut Stream<T> {
        self.0.get_mut()
    }
}
