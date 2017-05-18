use std::env;

#[macro_use]
extern crate bencher;
use bencher::Bencher;

extern crate rbf;
use rbf::record::{AsciiMode};
use rbf::layout::{Layout, setup};
use rbf::reader::Reader;

fn load_layout(bench: &mut Bencher) {
    bench.iter(|| {
        ::rbf::layout::setup::layout_load_layout_ascii()
    })
}

fn read_file(bench: &mut Bencher) {
    bench.iter(|| {
        // load our layout
        let layout = Layout::<AsciiMode>::new("./tests/test.xml");

        // create reader
        fn mapper(x: &str) -> &str { &x[0..2] };
        let mut reader = Reader::<AsciiMode>::new("./tests/test_ascii.data", layout, mapper);          

        while let Some(rec) = reader.next() { 
            let fname = rec.name.len();
        }             
    })
}

// get file name from env variable to get the right file
fn read_big_file(bench: &mut Bencher) {
    bench.iter(|| {
        // get file name
        let key = match env::var_os("RBF_FILE") {
            Some(val) => val,
            None => panic!("RBF_FILE is not defined in the environment."),
        };
        let rbf_file = key.into_string().unwrap();

        // load our layout
        let layout = Layout::<AsciiMode>::new("./tests/test.xml");

        // create reader
        fn mapper(x: &str) -> &str { &x[0..2] };
        let mut reader = Reader::<AsciiMode>::new(&rbf_file, layout, mapper);

        while let Some(rec) = reader.next() { 
            let fname = rec.name.len();
        }
             
    })
}

benchmark_group!(benches, load_layout, read_file, read_big_file);
benchmark_main!(benches);