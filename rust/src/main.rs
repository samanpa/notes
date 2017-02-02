extern crate notes;

use notes::core;
use notes::core::{Time, TimerTask, simpletimer, Context, Timer};
use notes::logger::{LogFile,Permission};

struct Printer {
    x : u64
}

impl TimerTask for Printer {
    fn run(&self, ctx: &Context, time: Time) {
        println!("\t{}", self.x)
    }
}

fn print(x: u64) -> Box<Printer> {
    Box::new(Printer{x: x})
}

fn add(time: &Time, dur : u64) -> Time {
    time.clone() + std::time::Duration::from_secs(dur)
}

fn main()
{
    use notes::core::Timer;
    
    let args : Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        println!("No log file given");
        return;
    }
    let logfile = LogFile::new(&args[1], Permission::ReadWrite);
    let size = logfile.unwrap().write(Time::now(), 0, "Some data.".as_bytes());
    println!("Wrote {:x}", size.unwrap());


    let mut timer = Box::new(simpletimer::SimpleTimer::new());
    let mut ctx = Context::new(0);

    let time = Time::now();

    for x in 0..32 {
        timer.schedule(&ctx, print(31-x), add(&time, x));
    }

    for x in 16..32 {
        println!("{}", x);
        (*timer).process(&ctx, add(&time, x));
    }
            
}
