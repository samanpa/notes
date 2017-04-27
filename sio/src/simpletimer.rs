use super::{Context, Time, TimerTask, Timer};
use std::ptr;


type NodePtr = *mut TimerEntry;

struct TimerEntry {
    next: NodePtr,
    time: Time,
    task: Box<TimerTask>,
    prev: NodePtr,
}

impl TimerEntry {
    fn new(task: Box<TimerTask>, time: Time) -> Self {
        TimerEntry{next: ptr::null_mut()
                   , time: time
                   , task: task
                   , prev: ptr::null_mut()}
    }
}

pub struct SimpleTimer {
    head: NodePtr
}

impl SimpleTimer {
    pub fn new() -> Self {
        SimpleTimer{ head: ptr::null_mut() }
    }
}

impl Timer for SimpleTimer {
    fn schedule(&mut self, task: Box<TimerTask>, time: Time) {
        let new_entry = Box::new(TimerEntry::new(task, time));
        let mut new_entry = Box::into_raw(new_entry);

        let mut next = self.head;
        let mut prev = ptr::null_mut();

        
        unsafe {
            while !next.is_null() && (*next).time <= time {
                prev = next;
                next = (*next).next;
            }
            
            (*new_entry).next = next;
            (*new_entry).prev = prev;

            if !next.is_null() {
                (*next).prev = new_entry;
            }
            if !prev.is_null() {
                (*prev).next = new_entry;
            }
        }

        if self.head == next {
            self.head = new_entry;
        }
    }

    fn peek_time(&self) -> Option<Time> {
        if !self.head.is_null() {
            unsafe{ Some((*self.head).time) }
        }
        else {
            None
        }
    }

    
    fn process(&mut self, ctx: &Context, time: Time) {
        unsafe {
            while !self.head.is_null() && (*self.head).time <= time {
                let mut entry = &mut *self.head;
                entry.task.run(ctx, time);
                let prev = self.head;
                self.head = entry.next;
                if !self.head.is_null() {
                    (*self.head).prev = ptr::null_mut();
                }

                drop(Box::from_raw(prev)); //drop for clarity
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::SimpleTimer;
    use super::{Time,TimerTask,Context};
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::vec::Vec;
    struct Task {
        num: u64,
        vec: Rc<RefCell<Vec<u64>>>,
    }
    impl Task {
        fn new(num: u64, vec: Rc<RefCell<Vec<u64>>>) -> Self {
            Task{ num: num, vec: vec }
        }
    }

    impl TimerTask for Task {
        fn run(&mut self, ctx: &Context, _time: Time) {
            self.vec.borrow_mut().push(self.num);
        }
    }
    
    #[test]
    fn basics() {
        use Timer;
        let mut timer = SimpleTimer::new();
        let time = Time::min();
        let vec = Rc::new(RefCell::new(Vec::new()));
        let max = 2_000_000;

        let start = Time::now();
        for i in 0..max {
            let i = max-i;
            let task1 = Task::new(i, vec.clone());
            timer.schedule(Box::new(task1), time + Duration::new(i, 0));
        }
        let end = Time::now();
        println!("{:?}", end -start);

        let start = Time::now();
        let ctx = Context::new(1);
        for i in 0..max {
            timer.process(&ctx, time + Duration::new(max, 0) );
            vec.borrow_mut().clear();
        }
        let end = Time::now();
        println!("{:?}", end -start);
    }

}
