use super::{Context, Time, TimerTask, Timer};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

struct TimerEntry {
    time: Time,
    task: Box<TimerTask>,
}

impl TimerEntry {
    fn new(time : Time, task: Box<TimerTask>) -> Self {
        TimerEntry{ time: time, task: task}
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &TimerEntry) -> Ordering {
        self.time.cmp(&other.time).reverse()
    }
}
impl PartialEq for TimerEntry {
    fn eq(&self, other: &TimerEntry) -> bool {
        self.time == other.time
    }
}
impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &TimerEntry) -> Option<Ordering>{
        self.time.partial_cmp(&other.time).map( |order| order.reverse())
    }
}
impl Eq for TimerEntry {}

pub struct HeapTimer {
    entries : BinaryHeap<TimerEntry>,
}

impl HeapTimer{
    pub fn new() -> Self {
        HeapTimer{ entries : BinaryHeap::new() }
    }
}

impl Timer for HeapTimer {
    fn schedule(&mut self, cb: Box<TimerTask>, time: Time) {
        let entry = TimerEntry::new(time, cb);
        self.entries.push(entry);
    }

    fn process(&mut self, ctx: &Context, time: Time) {
        loop {
            let entry = self.entries.pop();
            match entry {
                None => break,
                Some(entry) => {
                    if entry.time > time.clone() {
                        self.entries.push(entry);
                        break;
                    } else {
                        entry.task.run(ctx, time.clone())
                    }
                }
            }
        }
    }
}
