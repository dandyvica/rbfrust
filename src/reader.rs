//! Represents a way to read a record-based file by mapping each read line to a record. The mapping between
//! the data from the file and the record name is made by the `mapper` function.
//!
//! # Examples
//! ```rust
//!    use rbf::record::{AsciiMode, UTF8Mode};
//!    use rbf::layout::Layout;
//!    use rbf::reader::Reader;
//!
//!    // load our layout
//!    let layout = Layout::<UTF8Mode>::new("./tests/test.xml");
//!
//!    // create reader
//!    fn mapper(x: &str) -> &str { &x[0..2] };
//!    let mut reader = Reader::<UTF8Mode>::new("./tests/test_utf8.data", layout, mapper);
//!
//!    // useful vars
//!    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
//!    let digits = "123456789";
//!    let greek = "αβγδεζηθικλμνξοπρστυφχψω";
//!    
//!    // read file and loop through records
//!    while let Some(rec) = reader.next() {  
//!        match rec.name.as_ref() {
//!            "LL" => {
//!                assert_eq!(rec.get_value("ID"), "LL");        
//!
//!                // test every field
//!                for (i, l) in letters.chars().enumerate() {
//!                    let fname = format!("W{}", i+1);
//!                    assert_eq!(rec.get_value(&fname), l.to_string().repeat(i+1));
//!                }                                          
//!            }
//!            "NB" => {
//!                assert_eq!(rec.get_value("ID"), "NB"); 
//!                // test every field
//!                for (i, n) in digits.chars().enumerate() {
//!                    let fname = format!("N{}", i+1);
//!                    assert_eq!(rec.get_value(&fname), n.to_string().repeat(i+1));
//!                }                                             
//!            },
//!            "GL" => {
//!                assert_eq!(rec.get_value("ID"), "GL");   
//!                for (i, l) in greek.chars().enumerate() {
//!                    let fname = format!("G{}", i+1);
//!                    assert_eq!(rec.get_value(&fname), l.to_string().repeat(i+1));
//!                }                        
//!            },   
//!            "DP" => {
//!                assert_eq!(rec.get_value("ID"), "DP");    
//!                assert_eq!(rec.get("F5").unwrap()[0].value(), "AAAAA");
//!                assert_eq!(rec.get("F5").unwrap()[1].value(), "BBBBB"); 
//!                assert_eq!(rec.get("F5").unwrap()[2].value(), "CCCCC"); 
//!                assert_eq!(rec.get("F5").unwrap()[3].value(), "DDDDD");                                                                       
//!            },                      
//!            _ => panic!("record name <{}> not found in file <{}>", rec.name, "./tests/test_utf8.data")
//!        }
//!    }
//!
//! ```

use std::error::Error;
use std::io::{BufReader,BufRead};
use std::fs::File;

use record::{ReadMode, Record};
use layout::Layout;

/// This enum defines whether we should stop reading when an unknown record ID is found
#[derive(PartialEq)]
pub enum ReaderLazyness {
    /// When set, this panics the reader
    Stringent,
    /// When set, ignore unknown reader
    Lazy,
}


// function type to get the record ID from the whole line read from the target file
type RecordMapper = fn(&str) -> &str;

pub struct Reader<T> {
    /// record-based file to read
    pub rbf_file: String,
    /// layout struct describing the file to read
    pub layout: Layout<T>,
    /// function to map each line to a record name
    pub mapper: RecordMapper,
    /// buffer use when reading the file line by line
    bufreader: BufReader<File>,
    /// the line read from file
    pub line: String,
    /// lazyness when reading
    pub lazyness: ReaderLazyness,
    /// input file size
    pub file_size: u64,
    /// number of chars read when reading a line
    pub chars_read: usize,
    /// number of lines read so far
    pub nblines_read: u64,
}

impl<T> Reader<T> {
    /// Creates a new reader.
    ///
    /// # Arguments
    ///
    /// * `rbf_file` - name and path of the record-based file to read
    /// * `layout`: Layout struct previously created from the XML layout file describing the data file
    /// * `mapper` function to map each line to a record name
    ///
    /// # Panics
    /// If `rbf_file` could not be read
    pub fn new(rbf_file: &str, layout: Layout<T>, mapper: RecordMapper) -> Reader<T>
    {
        // open file for reading
        let bufreader = match File::open(&rbf_file) {
            // if ok, create a new BufReader to read the file line by line
            Ok(f) => match layout.rec_length {
                0 => BufReader::new(f),
                _ => BufReader::with_capacity(layout.rec_length+1, f),
            },
            // The `description` method of `io::Error` returns a string that
            // describes the error            
            Err(why) => panic!("couldn't open {}: {}", rbf_file, why.description()),            
        };

        // get file size
        let metadata = ::std::fs::metadata(&rbf_file).unwrap();

        Reader {
            rbf_file: rbf_file.to_string(),
            layout: layout,
            mapper: mapper,
            bufreader: bufreader,
            line: String::new(),
            lazyness: ReaderLazyness::Lazy,
            file_size: metadata.len(),
            chars_read: 0,
            nblines_read: 0,
        }
    }

    pub fn next_record_id(&mut self) -> Option<String> {
        // record ID from line
        let mut rec_id: String;

        // try to get a record ID
        loop {
            // clear buffer, otherwise buffer is growing  
            self.line.clear();            

            // read one line of text
            match self.bufreader.read_line(&mut self.line) {
                // No bytes read? This is EOF and we must end the iteration
                Ok(chars_read) => if chars_read == 0 { 
                        return None; 
                    } else { 
                        self.chars_read = chars_read;
                        self.nblines_read += 1; 
                    },
                // error reading bytes
                Err(why) => panic!("error {} when reading file {}", why.description(), self.rbf_file),
            }; 

            // get the record ID using mapper
            rec_id = (self.mapper)(&self.line).to_owned();

            // record ID could not exist
            match self.layout.contains_record(&rec_id) {
                true => break,
                false => if self.lazyness == ReaderLazyness::Stringent {
                                panic!("couldn't find record ID {} in file {}", rec_id, self.rbf_file);
                        } 
                        else {
                                continue;
                        }
            };
        } 

        Some(rec_id)
    }

    /// Returns a mutable reference on the record corresponding to the line read. **next()** returns **None**
    /// if EOF. 
    /// It allows to read the whole file using the following idiom:
    ///
    /// ```rust,ignore
    ///  // loop through records
    ///  while let Some(rec) = reader.next() {
    ///      // do something with rec
    ///  } 
    /// ```    
    /// # Panics
    /// If an error is met when reading the file.
    pub fn next(&mut self) -> Option<&mut Record<T>>
        where Record<T>: ReadMode 
    {
        // record ID from line
        let mut rec_id: String;

        // try to get a record ID
        loop {
            // clear buffer, otherwise buffer is growing  
            self.line.clear();            

            // read one line of text
            match self.bufreader.read_line(&mut self.line) {
                // No bytes read? This is EOF and we must end the iteration
                Ok(chars_read) => if chars_read == 0 { 
                        return None; 
                    } else { 
                        self.chars_read = chars_read;
                        self.nblines_read += 1; 
                    },
                // error reading bytes
                Err(why) => panic!("error {} when reading file {}", why.description(), self.rbf_file),
            }; 

            // get the record ID using mapper
            rec_id = (self.mapper)(&self.line).to_owned();

            // record ID could not exist
            match self.layout.contains_record(&rec_id) {
                true => break,
                false => if self.lazyness == ReaderLazyness::Stringent {
                                panic!("couldn't find record ID {} in file {}", rec_id, self.rbf_file);
                        } 
                        else {
                                continue;
                        }
            };
        }

        // set value for this record
        let rec = self.layout.get_mut(&rec_id).unwrap();

        // set all field values
        rec.set_value(&self.line);

        // return our record
        return Some(rec);
    }

    /// Sets reader lazyness
    pub fn set_lazyness(&mut self, lazyness: ReaderLazyness) {
        self.lazyness = lazyness;
    }
 
}


