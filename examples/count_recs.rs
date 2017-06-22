// Simple exemple of rbf usage. Just counts the number of records and record IDs in the file.
// Usage:count_recs layout_file data_file
use std::env;
use std::collections::HashMap;

extern crate rbf;
use rbf::record::AsciiMode;
use rbf::layout::Layout;
use rbf::reader::Reader;

fn main () {
    let mut nb_lines: usize = 0;
    let mut nb_records: HashMap<String, usize> = HashMap::new();

    // get arguments
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: {} layout_file data_file", args[0]);
        std::process::exit(1);
    }


    // load layout (suppose only ascii data)
    let layout = Layout::<AsciiMode>::new(&args[1]);

    // create reader
    let mapper = Box::new(|x: &str| x[0..2].to_string());
    let mut reader = Reader::new(&args[2], layout, mapper);  

    // loop through records
    while let Some(rec) = reader.next() {
        nb_lines += 1;

        // if key doesn't exists, set to 1
        if nb_records.contains_key(&rec.name) {
            *nb_records.get_mut(&rec.name).unwrap() += 1;
        }
        else {
            nb_records.insert(rec.name.clone(), 1);
        }
    } 

    // print out results
    println!("Input file has {} lines", nb_lines);

    for (recname, i) in nb_records {
        println!("Number of {} records = {} ", recname, i);
    }
}