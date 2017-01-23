extern crate notes;

use notes::core::time;
use notes::logger::{LogFile,Permission};

fn main()
{
    let args : Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        println!("No log file given");
        return;
    }
    let logfile = LogFile::new(&args[1], Permission::ReadWrite);
    let size = logfile.unwrap().write(time::now(), 0, "Some data.".as_bytes());
    println!("Wrote {:x}", size.unwrap());
}
