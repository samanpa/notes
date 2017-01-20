extern crate notes;

use notes::core::time;
use notes::logger;

fn main()
{
    let start = time::now();
    let count = 10_000_000;
    for _ in 0..count {
        time::now();
    }
    let end = time::now();

    println!("{:?}", (end.ns - start.ns) / 1000_0000 );

    let logfile = logger::LogFile::new("logfile"
                                       , logger::Permission::ReadWrite);
    
}
