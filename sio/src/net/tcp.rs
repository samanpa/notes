extern crate llio;

use llio::{Buff,EventType,Token};
use ::reactor::Handle;
use ::{Context,EventSource,Service};
use std::net::SocketAddrV4;
use std::io::{Result,Read,Write};
use std::marker::PhantomData;

pub struct TcpListener {
    inner: llio::TcpListener,
}

impl TcpListener {
    pub fn new(addr: SocketAddrV4) -> Result<Self> {
        let listener = try!(llio::TcpListener::new(addr));
        Ok(TcpListener{inner: listener})
    }
}


//FIXME: should we have a transport trait
pub struct TcpStream {
    stream:  llio::TcpStream,
    addr: SocketAddrV4
}

pub struct Connect {
    stream:  Option<TcpStream>,
    token:   Token,
    handle:  Handle,
    fd:      llio::RawFd
}

pub struct TcpClient<S> {
    token:   Token,
    handle:  Handle,
    buff:    Buff,
    fd:      llio::RawFd,
    service: S
}

pub struct Chain<F,S> {
    connect: Connect,
    factory: F,
    fd:      llio::RawFd,
    _service: PhantomData<S>
}


impl TcpStream {
    pub fn connect(addr: SocketAddrV4, handle: Handle) -> Result<Connect> {
        let stream = try!(llio::TcpStream::new());
        let _      = try!(stream.nonblock());
        let _      = try!(stream.connect(&addr));
        let token  = try!(handle.new_token());
        let fd     = stream.fd();
        let stream  = TcpStream {
            stream: stream,
            addr: addr
        };
        let connect = Connect {
            stream: Some(stream),
            token:  token,
            handle: handle,
            fd: fd
        };
        Ok(connect)
    }

    fn has_sock_error(&self) -> Result<()> {
        //FIXME: Handle EWOULDBLOCK
        self.stream.has_sock_error()
    }
}

impl Read for TcpStream {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize> {
        self.stream.read(buff)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buff: &[u8]) -> Result<usize> {
        self.stream.write(buff)
    }

    fn flush(&mut self) -> Result<()> {
        self.stream.flush()
    }
}




impl Connect {
    fn process(&mut self, _: &mut Context) -> Result<TcpStream> {
        if let Some(ref stream) = self.stream {
            stream.has_sock_error()?
        };
        Ok(self.stream.take().unwrap())
    }

    pub fn with_service<F,S>(self, factory: F) -> Result<()>
        where F: 'static + Fn(TcpStream) -> S
        ,     S: 'static + Service
    {
        let handle = self.handle.clone();
        let token = self.token.clone();
        let fd = self.fd;
        //Fixme: Should we really be storing the fd in the chain.
        //  what if the fd has been closed for some reason.
        let chain = Chain{ connect: self
                           , factory: factory
                           , fd: fd
                           , _service: PhantomData
        };
        handle.register(token, EventType::Write, chain)?;
        Ok(())
    }
}


impl <F,S> EventSource for Chain<F,S>
    where F: 'static + Fn(TcpStream) -> S
    , S: 'static + Service {
    fn process(&mut self, ctx:&mut Context) -> Result<()> {
        //Fixme: Notify service if we failed to connect
        let stream = try!(self.connect.process(ctx));
        let service = (self.factory)(stream);
        //FIXME: handle EWOULDBLOCK
        let mut handle = self.connect.handle.clone();
        let mut client = TcpClient {
            token:   self.connect.token.clone(),
            handle:  self.connect.handle.clone(),
            buff:    Buff::with_capacity(64),
            service: service,
            fd:      self.fd
        };
        try!(client.service.on_connect());
        try!(handle.modify(client.token, EventType::Read, client));
        Ok(())
    }

    fn fd(&self) -> llio::RawFd {
        self.fd
    }
}


impl <S> EventSource for TcpClient<S>
    where S: Service
{
    fn process(&mut self, _: &mut Context) -> Result<()> {
        use std::io::Read;

        let nread = {
            let mut transport = self.service.get_transport();
            let nread = try!(transport.read(self.buff.as_mut_slice()));
            self.buff.advance_write(nread);
            nread
        };
        if nread == 0 {
            self.service.on_disconnect();
            try!(self.handle.unregister(self.token.clone()));
            return Ok(());
        }
        let len = try!(self.service.process(self.buff.as_slice()));
        self.buff.advance_read(len);
        Ok(())
    }

    fn fd(&self) -> llio::RawFd {
        self.fd
    }
}
