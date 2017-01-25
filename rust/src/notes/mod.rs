mod note;
mod reminder;

pub enum RepeatFrequency
{
    None,
    Hourly,
    Daily,
    Weekend,
    WeekDay,        
    Weekly,
    Monthly,
    Yearly
}
        
pub struct Repeat {
    interval: i64,
    frequency: RepeatFrequency
}


