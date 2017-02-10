extern crate notes;

use notes::core::{Time, TimerTask, Context, Timer};
use notes::core::{simpletimer, reactor};
use notes::logger::{LogFile,Permission};

struct Printer {}

impl TimerTask for Printer {
    fn run(&self, ctx: &Context, time: Time) {
    }
}

fn print() -> Box<Printer> {
    Box::new(Printer{})
}

fn add(time: &Time, dur : u64) -> Time {
    time.clone() + std::time::Duration::from_secs(dur)
}

fn main()
{
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
    let end = 1000_0000;

    for x in 0..end {
        timer.schedule(&ctx, print(), add(&time, x));
    }
    let start_time = Time::now();
    for x in 0..end {
        (*timer).process(&ctx, add(&time, x));
    }

    let time_diff = Time::now() - start_time;
    println!("{:?}", time_diff);
    
}



fn run_client()
{
    let timer = simpletimer::SimpleTimer::new();

    //let start_time = Time::now();

    let mut reactor = reactor::Reactor::new(timer);
    //let tcp_client = Rc::new(TcpClient::new(reactor, ));
    /*
    tcp_client.on_connect( 
*/
    let _ = reactor.as_mut().map(|reactor| {
        let mut ctx = Context::new(0);
        reactor.run(&mut ctx, true);
    });
    
}
