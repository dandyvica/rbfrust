//! Represents a structure containing all fields and records, read from an external XML file.
//! This definition file, called **layout** file, defines all field and record features.
//!
//! # Examples
//! ```rust
//!  use rbf::record::AsciiMode;
//!  use rbf::layout::Layout;
//!
//!  // load my layout
//!  let layout = Layout::<AsciiMode>::new("./tests/test.xml");
//!
//!  // check layout methods
//!  assert_eq!(layout.contains_record("LL"), true);
//!  assert_eq!(layout.contains_record("100"), false);    
//!
//!  // layout has 36 records
//!  assert_eq!(layout.len(), 4);
//!
//!  // first record has 27 fields
//!  assert_eq!(layout.get("LL").unwrap().count(), 27);
//!
//!  // fiel W1 is present in layout, but no FOO
//!  assert!(layout.contains_field("W1"));
//!  assert!(!layout.contains_field("FOO")); 
//!
//!  // loop
//!  for (recname, rec) in &layout {
//!      assert!(recname.len() >= 2);
//!  }
//! ```
use std::env;
use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::collections::HashMap;
use std::rc::Rc;

use xml::reader::{EventReader, XmlEvent};

use fieldtype::FieldDataType;
use field::Field;
use record::Record;

// useful macro to get value from attribute name
#[doc(hidden)]
#[macro_export]
macro_rules! get_value {
    ($attr:ident, $name: expr) => {{
        $attr.iter().find(|e| e.name.local_name == $name).unwrap().value.clone()
    }};
}


#[derive(Debug)]
pub struct Layout<T> {
    /// XML layout file name
    pub xml_file: String,
    /// If all records have the same length, this stores the record length for all records
    pub rec_length: usize,    
    /// Layout file version
    pub version: String,
     /// Layout file description   
    pub description: String,
    /// SQL schema name (future use)
    pub schema: String,
    /// Regex for lines to exclude from reading
    pub ignore_line: String,
    /// List of field names to always skip when reading
    pub skip_field: String,
    /// Hash map of all read records from file
    pub rec_map: HashMap<String, Record<T>>,
}

use xml::attribute::OwnedAttribute;
fn as_hash(attributes: &Vec<OwnedAttribute>) -> HashMap<&str, &str>
{
    // loop through attributes to create a hash. Not present in xml_rs (?!)
    let mut h: HashMap<&str, &str> = HashMap::new();

    for own_attr in attributes {
        h.insert(&own_attr.name.local_name, &own_attr.value);
    }
    h
}

impl<T> Layout<T> {
    /// Reads the XML layout file to create record and field structs.
    ///
    /// # Arguments
    ///
    /// * `xml_file` - full file name and path of the XML layout file
    /// 
    ///
    /// # Panics
    /// If `xml_file` could not be read   
    pub fn new(xml_file: &str) -> Layout<T> {
        // try to open xml_file
        let file = match File::open(&xml_file) {
            // The `description` method of `io::Error` returns a string that
            // describes the error
            Err(why) => panic!("couldn't open {}: {}, current directory is: {}", 
                            xml_file, why.description(), env::current_dir().unwrap().display()),
            Ok(file) => BufReader::new(file),
        };

        // define hash to hold fieldtypes
        let mut ftypes: HashMap<String, Rc<FieldDataType>> = HashMap::new();

        //let mut rec_list: Vec<Record> = Vec::new();
        let mut rec_map: HashMap<String, Record<T>> = HashMap::new();

        let mut last_rec_name = String::new();

        // temp variable to get meta values
        let mut rec_length: usize = 0;
        let mut version = String::new();
        let mut description = String::new();
        let mut schema = String::new();
        let mut ignore_line = String::new();
        let mut skip_field = String::new();       

        // loop through elements
        let parser = EventReader::new(file);
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                    // fetch attributes as a hash
                    let attr = as_hash(&attributes);

                    // now depending on XML tag
                    match name.local_name.as_ref() {
                        "meta" => {
                            rec_length = match attr.get("reclength") {
                                Some(v) => v.parse::<usize>().unwrap(),
                                None => 0,
                            };  
                            version = match attr.get("version") {
                                Some(v) => v.to_string(),
                                None => String::from(""),
                            }; 
                            description = match attr.get("description") {
                                Some(v) => v.to_string(),
                                None => String::from(""),
                            }; 
                            schema = match attr.get("schema") {
                                Some(v) => v.to_string(),
                                None => String::from(""),
                            };                                                                                   
                            ignore_line = match attr.get("ignoreLine") {
                                Some(v) => v.to_string(),
                                None => String::from(""),
                            };
                            skip_field = match attr.get("skipField") {
                                Some(v) => v.to_string(),
                                None => String::from(""),
                            };                            
                        }
                        "fieldtype" => {
                            let ft_name = attr.get("name").unwrap();
                            let ft_type = attr.get("type").unwrap();                           
                            ftypes.insert(
                                ft_name.to_string(), 
                                Rc::new(FieldDataType::new(ft_name, ft_type))
                            );
                        }                     
                        "record" => {
                            let rec_name = attr.get("name").unwrap();                            
                            let rec_desc = attr.get("description").unwrap();
                            let rec_length: usize;

                            // save last met Record name to be able to add fields whenever we meet
                            // a <field> tag
                            last_rec_name = rec_name.to_string();

                            // length could be present or Not
                            rec_length = match attr.get("length") {
                                Some(length) => length.parse::<usize>().unwrap(),
                                None => 0,
                            };

                            // add new record
                            //rec_list.push(Record::new(last_rec_name, rec_type, rec_length))
                            rec_map.insert(
                                rec_name.to_string(),
                                Record::<T>::new(rec_name, rec_desc, rec_length)
                            );                            
                        }  
                        "field" => {
                            // name and description are mandatory
                            let f_name = attr.get("name").unwrap();
                            let f_desc = attr.get("description").unwrap();

                            // so is the field type
                            let f_type = attr.get("type").unwrap().to_string();                                                       
                            let ft = ftypes.get(&f_type).unwrap();

                            // length could be present or Not
                            let f_length = match attr.get("length") {
                                Some(length) => length.parse::<usize>().unwrap(),
                                None => 0,
                            };

                            // if length is not present, then lower and upper bounds for this field should
                            // be present
                            if f_length == 0 {
                                // get lower offset
                                let f_lower_offset = match attr.get("lower_offset") {
                                    Some(n) => n.parse::<usize>().unwrap(),
                                    None => 0,
                                };  
                                    
                                // get upper offset
                                let f_upper_offset = match attr.get("upper_offset") {
                                    Some(n) => n.parse::<usize>().unwrap(),
                                    None => 0,
                                };

                                // add Field into the last created record
                                rec_map.get_mut(&last_rec_name).unwrap().push(
                                    Field::from_offset(
                                        f_name, f_desc, &ft, f_lower_offset, f_upper_offset
                                    )                               
                                );                                                  
                            }
                            // here, length is not null
                            else {
                                // add Field into the last created record
                                rec_map.get_mut(&last_rec_name).unwrap().push(
                                    Field::from_length(
                                        f_name, f_desc, &ft, f_length
                                    )                               
                                );
                            }
                        }                      
                        _ => ()
                    }
                    //println!("{} {:?}", name, attributes);
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }        

        Layout {
            xml_file: xml_file.to_string(),
            rec_length: rec_length,    
            version: version,
            description: description,
            schema: schema,
            ignore_line: ignore_line,
            skip_field: skip_field,
            rec_map: rec_map
        }
    }

    /// Returns the number of records in the layout
    pub fn len(&self) -> usize {
        self.rec_map.len()
    }

    /// Tests if Layout contains a record by giving its name
    pub fn contains_record(&self, recname: &str) -> bool {
        self.rec_map.contains_key(recname)
    } 

    /// Tests if Layout contains a field record-wise
    pub fn contains_field(&self, fname: &str) -> bool {
        self.rec_map.iter().any(|(_,v)| v.contains_field(fname))
    }    

    /// Gets a record reference from its name
    pub fn get(&self, rec_name: &str) -> Option<&Record<T>> {
        self.rec_map.get(rec_name)
    }
    /// Gets a mutable reference on record from its name    
    pub fn get_mut(&mut self, rec_name: &str) -> Option<&mut Record<T>> {
        self.rec_map.get_mut(rec_name)
    }

    /// Checks whether layout is valid: if `rec_length` is not 0, all records have the same length
    /// the sum of length all fields (i.e. record length) should match the `rec_length` value.
    /// If not, each record length should match the declared length
    pub fn is_valid(&self) -> (bool,&str,usize,usize) {
        if self.rec_length != 0 {
            for (_, rec) in &self.rec_map {
                if self.rec_length != rec.calculated_length {
                    return (false, "", self.rec_length, rec.calculated_length)
                }
            }
        }
        else {
            for (_, rec) in &self.rec_map {
                if rec.declared_length != rec.calculated_length {
                    return (false, &rec.name, rec.declared_length, rec.calculated_length)
                }
            }            
        }
        (true, "", 0, 0)
    }
           

}

// non-consuming iterator (access items by ref)
impl<'a, T> IntoIterator for &'a Layout<T> {
    type Item = (&'a String, &'a Record<T>);    
    type IntoIter = ::std::collections::hash_map::Iter<'a, String, Record<T>>;
    
    // a Record contains a vector, just return the vector iterator
    fn into_iter(self) -> Self::IntoIter {
        self.rec_map.iter()
    }
}

// module to setup test data for layout
pub mod setup {

    use record::AsciiMode;    
    use layout::Layout;
    
    pub fn layout_load_layout_ascii() -> Layout<AsciiMode> {
        // load our layout
        Layout::<AsciiMode>::new("./tests/test.xml")
    }  

}

#[cfg(test)]
mod tests {

    #[test]
    fn layout_ascii() {
        // load our layout
        let layout = ::layout::setup::layout_load_layout_ascii();

        // is it a valid XML layout ?
        assert!(layout.is_valid().0);

        // check layout methods
        assert_eq!(layout.contains_record("LL"), true);
        assert_eq!(layout.contains_record("100"), false);    

        // Layout has 3 records
        assert_eq!(layout.len(), 4);

        // LL record has 27 fields
        assert_eq!(layout.get("LL").unwrap().count(), 27);

        // field F1 is present in layout, but no FOO
        assert!(layout.contains_field("W1"));
        assert!(!layout.contains_field("FOO")); 

        // loop
        for (recname, rec) in &layout {
            assert!(recname.len() >= 2);
            assert!(rec.name.len() <= 3);        
        }        
    }

/*    #[bench]
    fn bench_load_layout(b: &mut Bencher) {
        b.iter(|| ::layout::setup::layout_load_layout_ascii());
    }  */     
}