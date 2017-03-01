pub mod time;
pub mod simpletimer;
pub mod error;
pub mod reactor;
pub mod event;
pub mod net;

pub use self::time::Time;

#[allow(dead_code)]
pub struct Context {
    event_time : Time,
    exchange_time : Time,
    channel : u64,
}

impl Context {
    pub fn new(channel: u64) -> Self {
        Context{ event_time : Time::now(),
                 exchange_time: Time::now(),
                 channel: channel }
    }

}

pub trait TimerTask {
    fn run(&self, ctx: &Context, time: Time);
}

pub trait Timer {
    fn schedule(&mut self, ctx: &Context, cb: Box<TimerTask>, time: Time);
    fn process(&mut self, ctx: &Context, time: Time);
}



