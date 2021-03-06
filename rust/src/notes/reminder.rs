use std;
use core::time::Time;

pub struct Task {
    location: std::string::String,
    id: u32,
    repeat: super::Repeat,
    note: std::string::String,
    owner: std::string::String
}

pub enum TaskUpdateType {
    Start,
    Amend,
    Finish,
}

pub struct TaskUpdate {
    time: Time,
    entry_id: u32,
    update_type: TaskUpdateType
}

pub struct Reminder {
    due : Time,
    entry_id : u32
}


pub struct Timer {
    due : Time
}
