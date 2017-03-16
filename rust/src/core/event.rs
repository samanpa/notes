pub trait EventHandler {
    fn process(&mut self, ctx: &mut super::Context);
    fn fd(&self) -> ::plat::net::RawFd;
}
