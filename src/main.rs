extern crate notes;

use notes::core::time;

fn main()
{
    let start = time::now();
    let count = 1000_0000;
    for i in 0..count {
        time::now();
    }
    let end = time::now();

    println!("{:?}", (end.ns - start.ns) / 1000_0000 );
}
