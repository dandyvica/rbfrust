#[macro_use]
extern crate bencher;
use bencher::Bencher;

extern crate rbf;
use rbf::record::{AsciiMode, UTF8Mode, ReadMode};
use rbf::layout::{Layout};
use rbf::reader::Reader;

// how long is it to read a Layout
fn load_layout(bench: &mut Bencher) {
    bench.iter(|| {
        rbf::layout::setup::layout_load_layout_ascii()
    })
}

// try to bench record set_value
fn set_value(bench: &mut Bencher) {
    let mut rec = rbf::record::setup::set_up_by_offset::<AsciiMode>();
    bench.iter(|| {
        rec.set_value("AAAAAAAAAABBBBBBBBBBCCCCCCCCCCCCCCCCCCCCDDDDDDDDDD")
    })
}

// try to bench record set_value
fn set_value_huge_100(bench: &mut Bencher) {
    let mut rec = rbf::record::setup::set_up_by_length_huge::<AsciiMode>(100);
    let s = "A".to_string().repeat(1000);

    bench.iter(|| {
        rec.set_value(&s)
    })
}

// try to bench record set_value
fn set_value_huge_1000(bench: &mut Bencher) {
    let mut rec = rbf::record::setup::set_up_by_length_huge::<AsciiMode>(1000);
    let s = "A".to_string().repeat(10000);

    bench.iter(|| {
        rec.set_value(&s)
    })
}

// try to bench record set_value
fn set_value_huge_utf8_1000(bench: &mut Bencher) {
    let mut rec = rbf::record::setup::set_up_by_length_huge::<UTF8Mode>(1000);
    let s = "α".to_string().repeat(10000);

    bench.iter(|| {
        rec.set_value(&s)
    })
}

fn next_record_id(bench: &mut Bencher) {
    // load our layout
    let layout = Layout::<AsciiMode>::new("./tests/test.xml");

    // create reader
    fn mapper(x: &str) -> &str { &x[0..2] };
    let mut reader = Reader::<AsciiMode>::new("./tests/test_ascii.data", layout, mapper); 


    bench.iter(|| {
        reader.next_record_id()             
    })
}


benchmark_group!(benches, load_layout, set_value, set_value_huge_100, 
    set_value_huge_1000, set_value_huge_utf8_1000, next_record_id);
benchmark_main!(benches);