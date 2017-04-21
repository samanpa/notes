extern crate llio;

use llio::{Buff,EventType,Token};
use ::reactor::Handle;
use ::{Context,EventHandler,Service,ServiceFactory};
use std::net::SocketAddrV4;
use std::io::{Result};
use std::marker::PhantomData;

//FIXME: should we have a transport trait

pub struct Connect<NS: ServiceFactory<S>, S: Service + 'static> {
    stream:  Option<llio::TcpStream>,
    token:   Token,
    handle:  Handle,
    factory: NS,
    fd:      llio::RawFd,
    _phantom: PhantomData<S>
}

pub struct TcpClient<S> where S: Service{
    token: Token,
    handle: Handle,
    buff: Buff,
    service: S,
    fd:      llio::RawFd,
}

impl <NS,S> Connect<NS,S>
    where NS: ServiceFactory<S> + 'static
    , S: Service 
{
    pub fn get_token(&self) -> Token {
        self.token.clone()
    }
}

impl <NS,S> EventHandler for Connect<NS,S>
    where NS: ServiceFactory<S> + 'static
    , S: Service 
{
    fn process(&mut self, _: &mut Context) -> Result<()> {
        let stream = self.stream.take().unwrap();
        let fd = stream.fd();
        try!(stream.has_sock_error());
        let mut client = TcpClient {
            token: self.token.clone(),
            handle: self.handle.clone(),
            buff: Buff::with_capacity(64),
            service: try!(self.factory.create(stream)),
            fd: fd
        };
        client.service.on_connect();
        try!(self.handle.modify(self.token, EventType::Read, client));
        Ok(())
    }

    fn fd(&self) -> llio::RawFd {
        self.fd
    }
}

impl <S> TcpClient<S> where S: Service {
    pub fn connect<NS>(addr: SocketAddrV4, handle: Handle, factory: NS) -> Result<Connect<NS,S>>
        where NS: ServiceFactory<S>  {
        let stream = try!(llio::TcpStream::new());
        let _      = try!(stream.nonblock());
        let _      = try!(stream.connect(&addr));
        let token  = try!(handle.new_token());
        let fd     = stream.fd();
        let connect = Connect {
            stream: Some(stream),
            token:  token,
            handle: handle,
            fd: fd,
            factory: factory,
            _phantom: PhantomData
        };
        Ok(connect)
    }
}


impl <S> EventHandler for TcpClient<S>
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
