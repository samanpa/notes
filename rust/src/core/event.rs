pub enum EventType
{
    Read,
    Write,
    ReadWrite
}

pub trait EventHandler {
    fn process(&mut self, ctx: &mut super::Context);
    fn fd(&self) -> i32;
}
