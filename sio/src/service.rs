use llio;
use std::io::Result;

pub trait Service {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn process(&mut self, data: &[u8]) -> Result<usize>;
    fn get_transport(&mut self) -> &mut llio::TcpStream;
}

pub trait ServiceFactory<S: Service> {
    fn create(&mut self, stream: llio::TcpStream) -> Result<S>;
}

