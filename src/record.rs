//! Represents a record by its name, description and length. Each record contains a list of contiguous fields
//! which hold values when reading a record-based file.
//!
//! # Examples
//! ```rust
//! use std::rc::Rc;
//!
//! use rbf::fieldtype::FieldType;
//! use rbf::field::Field;
//! use rbf::record::{ReadMode, UTF8Mode, Record};
//!
//! let ft1 = Rc::new(FieldType::new("I", "integer"));                  
//! 
//! let f1 = Field::new("FIELD1", "Description for field 1", &ft1, 10);
//! let f2 = Field::new("FIELD2", "Description for field 2", &ft1, 10);
//! let f3 = Field::new("FIELD3", "Description for field 3", &ft1, 20);
//! let f4 = Field::new("FIELD2", "Description for field 2", &ft1, 10);        
//!
//! let mut rec = Record::<UTF8Mode>::new("RECORD1", "Description for record 1", 20);
//!
//! rec.push(f1);
//! rec.push(f2); 
//! rec.push(f3);
//! rec.push(f4); 
//!
//! assert_eq!(rec.calculated_length, 50);
//! assert_eq!(rec.count(), 4);
//!
//! let s = "FIELD1".to_string();
//!
//! assert!(rec.contains_field(&s)); 
//! assert_eq!(rec.contains_field("FOO"), false);     
//!
//! assert!(rec.get("FIELD1").is_some());
//! assert!(rec.get("FOO").is_none());
//!
//! let s2 = "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCCCCCCCCCCCDDDDDDDDDD";
//! rec.set_value(&s2);
//! assert_eq!(rec[0].value(), "AAAAAAAAAA");
//! assert_eq!(rec[1].value(), "BBBBBBBBBB"); 
//! assert_eq!(rec[2].value(), "CCCCCCCCCCCCCCCCCCCC");    
//! assert_eq!(rec[3].value(), "DDDDDDDDDD");
//!
//!
//! let s3 = "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCCCCCCCCCCCDDDDDDDDDDEEEEEEEEEEEEEEEE";
//! rec.set_value(&s3);
//! assert_eq!(rec[0].value(), "AAAAAAAAAA");
//! assert_eq!(rec[1].value(), "BBBBBBBBBB"); 
//! assert_eq!(rec[2].value(), "CCCCCCCCCCCCCCCCCCCC");    
//! assert_eq!(rec[3].value(), "DDDDDDDDDD");  
//! 
//!
//! let s4 = "AAAAAAAAAA";
//! rec.set_value(&s4);
//! assert_eq!(rec[0].value(), "AAAAAAAAAA");
//! assert_eq!(rec[1].raw_value, "          "); 
//! assert_eq!(rec[2].raw_value, "                    ");    
//! assert_eq!(rec[3].raw_value, "          "); 
//! assert_eq!(rec[1].value(), ""); 
//! assert_eq!(rec[2].value(), "");    
//! assert_eq!(rec[3].value(), ""); 
//!
//!
//! let s5 = "ααααααααααββββββββββγγγγγγγγγγγγγγγγγγγγδδδδδδδδδδ";
//! rec.set_value(&s5);  
//! assert_eq!(rec[0].value(), "αααααααααα");
//! assert_eq!(rec[1].value(), "ββββββββββ"); 
//! assert_eq!(rec[2].value(), "γγγγγγγγγγγγγγγγγγγγ");    
//! assert_eq!(rec[3].value(), "δδδδδδδδδδ");  
//! ```

use std::fmt;
use std::ops::{Index, IndexMut};
use std::slice::{Iter, IterMut};
use std::marker::PhantomData;

use field::Field;

/// This allows to define a way to read either pure Ascii data or UTF-8 data. Because the way
/// of slicing is not the same, it's much more efficient using Ascii.
pub struct AsciiMode;
pub struct UTF8Mode;

/// This trait will be implemented by readers
pub trait ReadMode {
    fn set_value(&mut self, value: &str); 
}

/// Implement Ascii read mode
impl ReadMode for Record<AsciiMode> {
    /// Sets the record value (which is equivalent to setting all fields).
    fn set_value(&mut self, value: &str) {
        let s = self.adjust_value(value);

        // setting record value is setting value for all fields/records composing the record
        for f in &mut self.flist {
            let r = f.lower_bound..f.upper_bound;
            f.set_value(&s[r]);
        } 
    }
}

/// Implement UTF-8 read mode
impl ReadMode for Record<UTF8Mode> {
    /// Sets the record value (which is equivalent to setting all fields).
    fn set_value(&mut self, value: &str) {
        let s = self.adjust_value(value);

        // this is made for UTF-8 strings
        for f in &mut self.flist {
            let fvalue: String = s.chars().skip(f.lower_bound).take(f.length).collect();
            f.set_value(&fvalue);
        }         
    }
}



/// Macro which builds a vector of Record data fields.
///
/// # Example
/// ```rust,ignore
/// // this builds a vector of record length if rec is a Record
/// # #[macro_use] use rbf::record;
/// # let rec = ::rbf::record::setup::set_up::<::rbf::record::AsciiMode>();
/// let v = vector_of!(rec, length);
/// ```
#[macro_export]
macro_rules! vector_of {
    ($rec:ident, $field: ident) => {{
        let v: Vec<_> = $rec.flist.iter().map(|f| f.$field.clone()).collect();
        v
    }};
}

pub struct Record<T> {
    /// Record name
    pub name: String,
    /// Record description
    pub description: String,
    /// Wihtin a layout, records might have different lengths. In that case, this holds individual record length
    pub declared_length: usize,
    /// List of fields composing the record
    pub flist: Vec<Field>,
    /// Sum of all field lengths
    pub calculated_length: usize,
    /// Reader mode struct, just a place holder
    pub reader_mode: PhantomData<T>,
}

impl<T> Record<T> {
    /// Creates a new Record.
    ///
    /// # Arguments
    ///
    /// * `name` - name of the record
    /// * `description`: description of the record
    /// * `length`: number of chars of the record
    ///
    /// # Panics
    /// If `name` is empty
    pub fn new(name: &str, description: &str, length: usize) -> Record<T> {
        // first test arguments: non-sense to deal with empty data
        if name.is_empty() {
            panic!("Cannot create Record with an empty name!");
        }

        // initialize all relevant members
        Record {
            name: name.to_string(),
            description: description.to_string(),
            declared_length: length,
            flist: Vec::new(),
            calculated_length: 0,
            reader_mode: PhantomData,
        }        
    }

    /// Adds a Field structure to the end of the record.
    pub fn push(&mut self, mut field: Field) {
        // copy current length
        let length = self.flist.len();
        
        // set field index
        field.index = length;

        // offset at this moment is merely the length of record (starts at 0)
        field.offset = self.calculated_length;

        // record is becoming longer
        self.calculated_length += field.length;

        // and adjust field bounds
        field.lower_bound = field.offset;
        field.upper_bound = field.offset + field.length;  

        // get last field having the same name (if any)
        match self.get(&field.name) {
            Some(ref mut v) => { field.multiplicity = v.pop().unwrap().multiplicity + 1; }
            None => ()
        }

        // finally, save Field struct
        self.flist.push(field);

    }

    /// Tests whether a Record contains a Field by giving its name.
    pub fn contains_field(&self, fname: &str) -> bool {
        self.flist.iter().any(|f| f.name == fname)
    }

    /// Returns the number of fields in the record.
    pub fn count(&self) -> usize {
        self.flist.len()
    }

    /// Returns a vector of fields matching the predicate.
    /// # Exemple
    /// ```rust
    /// // only keep fields having length over 20 chars
    /// # let rec = ::rbf::record::setup::set_up::<::rbf::record::AsciiMode>();    
    /// let fields = rec.filter(|f| f.length >20);
    /// ```      
    pub fn filter<F>(&self, pred: F) -> Option<Vec<&Field>>
        where F: Fn(&Field) -> bool
    {
        // search for indices matching predicate. Return is a vector of field refs
        let result: Vec<_> = self.flist.iter().filter(|e| pred(e)).collect();

        match result.is_empty() {
            true => None,
            false => Some(result)
        }
    }     

    /// Returns a vector of fields matching the field name (this returns a vector 
    /// because a Field could appear more than once in a Record).
    pub fn get(&self, fname: &str) -> Option<Vec<&Field>> {
        self.filter(|f| f.name == fname)
    }

    /// Only keeps fields matching the predicate.
    pub fn retain<F>(&mut self, pred: F)
        where F: Fn(&Field) -> bool
    {
        self.flist.retain(|e| pred(e))
    }

    /// Removes fields matching the predicate.
    pub fn remove<F>(&mut self, pred: F)
        where F: Fn(&Field) -> bool
    {
        self.flist.retain(|e| !pred(e))
    }    

/*    /// Sets the record value (which is equivalent to setting all fields).
    pub fn set_value(&mut self, value: &str) {
        let s: String;

        // if shorter, left-pad with blanks
        if value.len() < self.calculated_length {
            s = format!("{:length$} ", value, length=self.calculated_length);
        }
        else {
            s = value.to_string();
        }

        // setting record value is setting value for all fields/records composing the record
/*        for f in &mut self.flist {
            let r = f.lower_bound..f.upper_bound;
            f.set_value(&s[r]);
        } */

        // this is made for UTF-8 strings
        for f in &mut self.flist {
            let fvalue: String = s.chars().skip(f.lower_bound).take(f.length).collect();
            f.set_value(&fvalue);
        }         

    }*/ 

    /// Returns the record value (concatenation of all field values).
    pub fn value(&self) -> String {
        let v: Vec<_> = self.flist.iter().map(|f| f.raw_value.clone()).collect();   
        v.join("")
    }

    /// Returns the value from a field when it's sure there's only one field (no duplication) matching the
    /// field name. 
    ///
    /// #panics
    /// If `fname` is not found.
    pub fn get_value(&self, fname: &str) -> &str {
        // check for key existence
        let fields = match self.get(fname) {
            Some(f) => f,
            None => panic!("Key {} not found in record {}", fname, self.name),
        };

        // safely returns valuechar_at
        fields[0].value()
    } 

    /// Returns the value from a field when there're duplicated fields matching the
    /// field name. Returns the i-th field value (starting from 0).
    ///
    /// #panics
    /// If `fname` is not found.    
    pub fn get_value_with_index(&self, fname: &str, i: usize) -> &str {
        // check for key existence
        let fields = match self.get(fname) {
            Some(f) => f,
            None => panic!("Key {} not found in record {}", fname, self.name),
        };

        // check also index
        let f = match fields.get(i) {
            Some(f) => f,
            None => panic!("Index {} is out of bound for field {} in record {}", i, fname, self.name),
        };

        f.value()
    } 

    /// Adjust value according to record length   
    fn adjust_value(&self, value: &str) -> String {
        // if shorter, left-pad with blanks
        if value.len() < self.calculated_length {
            format!("{:length$} ", value, length=self.calculated_length)
        }
        else {
            value.to_string()
        }
    }

}

// implement debug trait
impl<T> fmt::Debug for Record<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = format!("name: <{}>, description: <{}>\n", self.name, self.description);
        for field in &self.flist {
            s += format!("\tfield:<{}>\n", field).as_str();
        }     
        write!(f, "{}", s)
    }
}

/// Returns a field reference in record by giving its index within the record.
impl<T> Index<usize> for Record<T> {
    type Output = Field;

    fn index(&self, i: usize) -> &Self::Output {
        if i >= self.flist.len() {
            panic!("record {}: index {} out of bounds, max index = {}", self.name, i, self.flist.len()-1)
        }
        self.flist.index(i)
    }
}

/// Returns a mutable field reference in record by giving its index within the record.
impl<T> IndexMut<usize> for Record<T> {
    fn index_mut(&mut self, i: usize) -> &mut Field {
        if i >= self.flist.len() {
            panic!("record {}: index {} out of bounds, max index = {}", self.name, i, self.flist.len()-1)
        }        
        self.flist.index_mut(i)
    }
}

// iterators for looping through fields of a record

// consuming iterator
impl<T> IntoIterator for Record<T> {
    type Item = Field;
    type IntoIter = ::std::vec::IntoIter<Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.flist.into_iter()
    }
}

// non-consuming iterator (access items by ref)
impl<'a, T> IntoIterator for &'a Record<T> {
    type Item = &'a Field;
    type IntoIter = Iter<'a, Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.flist.iter()
    }
}

// non-consuming mutable iterator (access items by mut ref)
impl<'a, T> IntoIterator for &'a mut Record<T> {
    type Item = &'a mut Field;
    type IntoIter = IterMut<'a, Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.flist.iter_mut()
    }
}

// cloning a record is deep copy
impl<T> Clone for Record<T> {
    fn clone(&self) -> Record<T> {
        let mut cloned = Record::new(&self.name, &self.description, self.declared_length);

        // copy other fields which can be potentially already set
        for f in self {
            cloned.push(f.clone());
        }                                        

        cloned
    }
}

// module to setup test data for record
pub mod setup {
    use std::rc::Rc;

    use fieldtype::FieldType;
    use field::Field;
    use record::Record;

    // this fn sets up the relevant data for testing a record
    pub fn set_up<T>() -> Record<T> {
        let ft1 = Rc::new(FieldType::new("I", "integer"));                  
        
        let f1 = Field::new("FIELD1", "Description for field 1", &ft1, 10);
        let f2 = Field::new("FIELD2", "Description for field 2", &ft1, 10);
        let f3 = Field::new("FIELD3", "Description for field 3", &ft1, 20);
        let f4 = Field::new("FIELD2", "Description for field 2", &ft1, 10);        

        let mut rec = Record::<T>::new("RECORD1", "Description for record 1", 20);

        rec.push(f1);
        rec.push(f2); 
        rec.push(f3);
        rec.push(f4);

        rec        
    }    
 
}

#[cfg(test)]
mod tests {

    use record::{AsciiMode, UTF8Mode, ReadMode};

    #[test]
    fn record_ascii() {

        // setup data
        let mut rec = ::record::setup::set_up::<AsciiMode>();

        assert_eq!(rec.calculated_length, 50);
        assert_eq!(rec.count(), 4);

        assert_eq!(vector_of!(rec, name), vec!["FIELD1", "FIELD2", "FIELD3", "FIELD2"]);  
        assert_eq!(vector_of!(rec, description), vec!["Description for field 1", "Description for field 2", "Description for field 3", "Description for field 2"]);  
        assert_eq!(vector_of!(rec, length), vec![10, 10, 20, 10]);  
        

        let s = "FIELD1".to_string();

        assert!(rec.contains_field(&s)); 
        assert_eq!(rec.contains_field("FOO"), false);     

        assert!(rec.get("FIELD1").is_some());
        assert!(rec.get("FOO").is_none());

        // line has exactly the right length in chars
        let s2 = "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCCCCCCCCCCCDDDDDDDDDD";
        rec.set_value(&s2);
        assert_eq!(rec[0].value(), "AAAAAAAAAA");
        assert_eq!(rec[1].value(), "BBBBBBBBBB"); 
        assert_eq!(rec[2].value(), "CCCCCCCCCCCCCCCCCCCC");    
        assert_eq!(rec[3].value(), "DDDDDDDDDD");
        assert_eq!(vector_of!(rec, raw_value), vec!["AAAAAAAAAA", "BBBBBBBBBB", "CCCCCCCCCCCCCCCCCCCC", "DDDDDDDDDD"]);

        // line is over right length in chars
        let s3 = "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCCCCCCCCCCCDDDDDDDDDDEEEEEEEEEEEEEEEE";
        rec.set_value(&s3);
        assert_eq!(rec[0].value(), "AAAAAAAAAA");
        assert_eq!(rec[1].value(), "BBBBBBBBBB"); 
        assert_eq!(rec[2].value(), "CCCCCCCCCCCCCCCCCCCC");    
        assert_eq!(rec[3].value(), "DDDDDDDDDD");  
        assert_eq!(vector_of!(rec, raw_value), vec!["AAAAAAAAAA", "BBBBBBBBBB", "CCCCCCCCCCCCCCCCCCCC", "DDDDDDDDDD"]);
        
        // line is shorter than the length in chars
        let s4 = "AAAAAAAAAA";
        rec.set_value(&s4);
        assert_eq!(rec[0].value(), "AAAAAAAAAA");
        assert_eq!(rec[1].raw_value, "          "); 
        assert_eq!(rec[2].raw_value, "                    ");    
        assert_eq!(rec[3].raw_value, "          "); 
        assert_eq!(rec[1].value(), ""); 
        assert_eq!(rec[2].value(), "");    
        assert_eq!(rec[3].value(), "");  
        assert_eq!(vector_of!(rec, str_value), vec!["AAAAAAAAAA", "", "", ""]);

    }

    #[test]
    fn record_utf8 () {

        // setup data
        let mut rec = ::record::setup::set_up::<UTF8Mode>();     

        let s5 = "ααααααααααββββββββββγγγγγγγγγγγγγγγγγγγγδδδδδδδδδδ";
        rec.set_value(&s5);  
        assert_eq!(rec[0].value(), "αααααααααα");
        assert_eq!(rec[1].value(), "ββββββββββ"); 
        assert_eq!(rec[2].value(), "γγγγγγγγγγγγγγγγγγγγ");    
        assert_eq!(rec[3].value(), "δδδδδδδδδδ");  
        assert_eq!(vector_of!(rec, raw_value), vec!["αααααααααα", "ββββββββββ", "γγγγγγγγγγγγγγγγγγγγ", "δδδδδδδδδδ"]);

    }

    #[test]
    #[should_panic]
    #[allow(unused_variables)]
    fn record_panic() {
        // setup data
        let rec = ::record::setup::set_up::<AsciiMode>();

        // this should panic
        let v1 = rec.get_value("FOO");
        let v2 = rec.get_value_with_index("FIELD2", 2);
    }        

}