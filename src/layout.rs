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
use std::rc::Rc;
use std::collections::HashMap;

use xml::reader::{EventReader, XmlEvent};
use regex::Regex;

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
    pub ignore_line: Regex,
    /// List of field names to always skip when reading
    pub skip_field: String,
    /// Hash map of all read records from file
    pub rec_map: HashMap<String, Record<T>>,
    /// Hash map of all field types found when reading
    pub ftypes: HashMap<String, Rc<FieldDataType>>,
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
        let mut ignore_line = Regex::new("").unwrap();
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
                                Some(v) => Regex::new(&v.to_string()).unwrap(),
                                None => Regex::new("").unwrap(),
                            };
                            skip_field = match attr.get("skipField") {
                                Some(v) => v.to_string(),
                                None => String::from(""),
                            };                            
                        }
                        "fieldtype" => {
                            // mandatory XML attributes
                            let ft_name = attr.get("name").unwrap();
                            let ft_type = attr.get("type").unwrap();   

                            let mut ft =  FieldDataType::new(ft_name, ft_type);                       

                            // optional XML attributes
                            match attr.get("pattern") {
                                Some(v) => ft.set_pattern(&v),
                                None => (),
                            }; 

                            // finally insert field type
                            ftypes.insert(
                                ft_name.to_string(), 
                                Rc::new(ft)
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

                            // try to get already insert field type                                                      
                            let ft = match ftypes.get(&f_type) {
                                Some(ft) => ft,
                                None => panic!("No field type {} found!", f_type),
                            };

                            // length could be present or Not
                            let f_length = match attr.get("length") {
                                Some(length) => length.parse::<usize>().unwrap(),
                                None => 0,
                            };

                            // if length is not present, then lower and upper bounds for this field should
                            // be present
                            if f_length == 0 {
                                // get lower offset
                                let f_lower_offset = match attr.get("start") {
                                    Some(n) => n.parse::<usize>().unwrap(),
                                    None => 0,
                                };  
                                    
                                // get upper offset
                                let f_upper_offset = match attr.get("end") {
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
            rec_map: rec_map,
            ftypes: ftypes,
        }
    }

    pub fn from_xml(xml_file: &str) -> Layout<T> {
        let mut layout = Layout::new(xml_file);
        let skip_field = layout.skip_field.clone();

        if layout.skip_field != "" {
            layout.set_skip_field(&skip_field);
        }  

        layout       
    }

    /// Returns the number of records in the layout.
    pub fn len(&self) -> usize {
        self.rec_map.len()
    }

    /// Tests if Layout contains a record by giving its name.
    pub fn contains_record(&self, recname: &str) -> bool {
        self.rec_map.contains_key(recname)
    } 

    /// Tests if Layout contains a field record-wise.
    pub fn contains_field(&self, fname: &str) -> bool {
        self.rec_map.iter().any(|(_,v)| v.contains_field(fname))
    }    

    /// Gets a record reference from its name.
    pub fn get(&self, rec_name: &str) -> Option<&Record<T>> {
        self.rec_map.get(rec_name)
    }
    
    /// Gets a mutable reference on record from its name.   
    pub fn get_mut(&mut self, rec_name: &str) -> Option<&mut Record<T>> {
        self.rec_map.get_mut(rec_name)
    }

    /// Gets a field type Rc.
    pub fn get_type(&self, ftype_name: &str) -> Option<&Rc<FieldDataType>> {
        self.ftypes.get(ftype_name)
    }  

    /// Removes each field from the list from the whole layout, i.e. from all records.
    /// If a field name doesn't exist, no error is returned and the deletion is ignored.
    pub fn remove(&mut self, flist: Vec<&str>) {
        for (_, rec) in &mut self.rec_map {
            rec.remove(|f| flist.contains(&&*f.name));
        }
    }

/*    /// Retains only the records and list specified. All other records or fields are removed.
    pub fn retain(&mut self, rec_list: HashMap<&str, Vec<&str>>) {
        // create vector of record names to retain only those ones.
        let rec_names: Vec<_> = rec_list.keys().collect();
        //self.rec_map.retain(|&k, _| rec_names.contains(k));

        // now for each remainig record, delete fields
        for (rec_name, rec) in &mut self.rec_map {
            if rec_names.contains(&&&**rec_name) {
                rec.retain(|f| rec_list.get(&**rec_name).unwrap().contains(&&*f.name));
            }
            else {
                self.rec_map.remove(&**rec_name);
            }
        }

    }*/    

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

    /// Sets skil field.
    pub fn set_skip_field(&mut self, skip_field: &str) {
        // save value and delete all fields in the list from layout
        self.skip_field = String::from(skip_field);

        // remove field names
        self.remove(::layout::utility::into_field_list(skip_field));
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

mod utility {
    /// Converts a comma-separated string into a vector of trimmed string refs.
    pub fn into_field_list(s: &str) -> Vec<&str> {
        let flist: Vec<_> = s.split(',').map(|f| f.trim()).collect();
        flist
    }

    /// Converts a comma-separated string into a vector of trimmed string refs.
    use std::collections::HashMap;    

    pub fn into_rec_map(s: &str) -> HashMap<&str, Vec<&str>> {
        let mut rec_map: HashMap<&str, Vec<&str>> = HashMap::new();

        for list in s.split(";") {
            let v: Vec<_> = list.split(":").map(|f| f.trim()).collect();
            rec_map.insert(v[0], into_field_list(v[1]));
        }

        rec_map
    } 

    #[test]
    fn layout_utility() {
        let mut s = into_field_list("AA, BB, CC, DD  ");
        assert_eq!(s, vec!("AA","BB","CC","DD"));

        s = into_field_list("AA ");
        assert_eq!(s, vec!("AA"));
        
        let v = into_rec_map("F1:AA,  BB, CC ; F2: DD, EE, FF   ; F3: GG, HH  ");
        assert_eq!(v.get("F1").unwrap(), &vec!("AA","BB","CC"));
        assert_eq!(v.get("F2").unwrap(), &vec!("DD","EE","FF"));
        assert_eq!(v.get("F3").unwrap(), &vec!("GG","HH"));
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

        // gain access to deep field
        let f = layout.get("LL").unwrap().get("ID").unwrap();
        assert_eq!(f[0].ftype.pattern, "\\w+");

        // same here
        assert_eq!(layout.get_type("A").unwrap().pattern, "\\w+");

        // field F1 is present in layout, but no FOO
        assert!(layout.contains_field("W1"));
        assert!(!layout.contains_field("FOO")); 

        // ignore line regex
        assert_eq!(layout.ignore_line.as_str(), "^A");

        // loop
        for (recname, rec) in &layout {
            assert!(recname.len() >= 2);
            assert!(rec.name.len() <= 3);        
        }        
    }

    #[test]
    fn layout_remove() {
        // load our layout
        let mut layout = ::layout::setup::layout_load_layout_ascii();
        assert_eq!(layout.contains_field("ID"), true);   

        // remove all "ID" fields from all records
        layout.remove(vec!("ID"));
        assert_eq!(layout.contains_field("ID"), false);

         // remove a list
        layout.remove(vec!("W26","N9","G24"));
        assert_eq!(layout.contains_field("ID"), false);

        assert_eq!(layout.get("LL").unwrap().count(), 25);
        assert_eq!(layout.get("NB").unwrap().count(), 8);
        assert_eq!(layout.get("GL").unwrap().count(), 23);           
    }

    #[test]
    fn layout_skip_field() {
        // load our layout
        let mut layout = ::layout::setup::layout_load_layout_ascii();

        layout.set_skip_field("ID , W26,    N9 ,   G24 ");

        assert_eq!(layout.contains_field("ID"), false);
        assert_eq!(layout.get("LL").unwrap().count(), 25);
        assert_eq!(layout.get("NB").unwrap().count(), 8);
        assert_eq!(layout.get("GL").unwrap().count(), 23);           
    } 
/*
    #[test]
    fn layout_retain() {
        // load our layout
        use std::collections::HashMap;
        let mut layout = ::layout::setup::layout_load_layout_ascii();

        let mut rec_list: HashMap<&str, Vec<&str>> = HashMap::new();
        rec_list.insert("LL", vec!("ID","W26"));
        rec_list.insert("NB", vec!("N7","N8"));

        layout.retain(rec_list);
           
    }  */      
    

/*    #[bench]
    fn bench_load_layout(b: &mut Bencher) {
        b.iter(|| ::layout::setup::layout_load_layout_ascii());
    }  */     
}