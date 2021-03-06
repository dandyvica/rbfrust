// Simple exemple of rbf usage. Just read the whole file
use std::env;
use std::time::Instant;

extern crate rbf;
use rbf::record::AsciiMode;
use rbf::layout::Layout;
use rbf::reader::Reader;

#[allow(unused_variables)]
fn main () {

    // get arguments
    let args: Vec<String> = env::args().collect();    

    // get arguments
    if args.len() == 1 {
        println!("Usage: {} layout_file data_file", args[0]);
        std::process::exit(1);
    }

    // tick time
    let now = Instant::now();

    // load layout (suppose only ascii data)
    let layout = Layout::<AsciiMode>::new(&args[1]);

    // create reader
    let mapper = Box::new(|x: &str| x[0..2].to_string());
    let mut reader = Reader::new(&args[2], layout, mapper);  

    // loop through records
    while let Some(rec) = reader.next() {

    } 

    // print out results
    let secs = now.elapsed().as_secs();
    println!("Elapsed time: {}.{} secs for {} lines read", 
            now.elapsed().as_secs(), now.elapsed().subsec_nanos(), reader.nblines_read);
    println!("Average: {} per lines/s", reader.nblines_read/secs);
}