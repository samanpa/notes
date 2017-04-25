use std::io::Result;

pub trait Service {
    fn on_connect(&mut self) -> Result<()>;
    fn on_disconnect(&mut self);
    fn process(&mut self, data: &[u8]) -> Result<usize>;
    fn get_transport(&mut self) -> &mut super::net::TcpStream;
}
