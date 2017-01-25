use super::{Context, Time, TimerElapsed, Timer};

struct TimerEntry {
    time: Time,
    data: Box<TimerElapsed>,
    prev: Option<Box<TimerEntry>>,
    next: Option<Box<TimerEntry>>,
}

struct SimpleTimer {
    next : Option<Box<TimerEntry>>
}

impl SimpleTimer {
    fn new() -> SimpleTimer {
        SimpleTimer{ next: None }
    }
}

impl Timer for SimpleTimer {
    fn schedule(&mut self, ctx: &Context, cb: Box<TimerElapsed>, time: Time) {
        let mut entry = TimerEntry{ time: time, data: cb, prev: None, next: None };
        let mut prev = None;
        let curr = &mut self.next;
        loop {
            match curr {
                Some(ref next) if next.time <= time
                    => {curr = &mut next }
                _   => break
            }
        }
        entry.next = curr;
        entry.prev = prev;

        if let Some(_) = prev {
            prev.next = entry;
        }
        if let Some(_) = curr {
            curr.prev = entry;
        }
        if curr == self.next {
            self.next = entry;
        }
    }
}
