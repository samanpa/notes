use super::{Context, Time, TimerElapsed, Timer};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

struct TimerEntry {
    time: Time,
    data: Box<TimerElapsed>,
}

impl TimerEntry {
    fn new(time : Time, data: Box<TimerElapsed>) -> Self {
        TimerEntry{ time: time, data: data}
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &TimerEntry) -> Ordering {
        self.time.cmp(&other.time)
    }
}
impl PartialEq for TimerEntry {
    fn eq(&self, other: &TimerEntry) -> bool {
        self.time == other.time
    }
}
impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &TimerEntry) -> Option<Ordering>{
        self.time.partial_cmp(&other.time)
    }
}
impl Eq for TimerEntry {}

struct SimpleTimer {
    entries : BinaryHeap<TimerEntry>,
}

impl SimpleTimer{
    fn new() -> Self {
        SimpleTimer{ entries : BinaryHeap::new() }
    }
}

impl Timer for SimpleTimer {
    fn schedule(&mut self, ctx: &Context, cb: Box<TimerElapsed>, time: Time) {
        let entry = TimerEntry::new(time, cb);
        self.entries.push(entry);
    }

    fn process(&mut self, time: Time, ctx: &Context) {
        loop {
            let entry = self.entries.pop();
            match entry {
                None => break,
                Some(entry) => {
                    if entry.time > time {
                        self.entries.push(entry);
                        break;
                    } else {
                        (*entry.data).on_elapsed(&ctx, time)
                    }
                }
            }
        }
    }
}
