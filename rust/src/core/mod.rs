pub mod time;
pub mod simpletimer;

pub use self::time::Time;

pub struct Context {
    event_time : Time,
    exchange_time : Time,
    channel : u64,
}

trait TimerElapsed {
    //Should we consume the callback object?
    fn on_elapsed(self, ctx: &Context, time: Time);
}

trait Timer {
    fn schedule(&mut self, ctx: &Context, cb: Box<TimerElapsed>, time: Time);
}



