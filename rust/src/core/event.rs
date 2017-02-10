pub enum EventType
{
    Read,
    Write,
    ReadWrite
}

pub trait EventHandler {
    fn handle(&mut self);
    fn get_fd(&self) -> i32;
}
