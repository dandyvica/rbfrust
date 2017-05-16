//! Represents a data field by its name, description, type and length.
//!
//! This struct should be used with its companion struct [Record](../record/struct.Record.html). If a record can
//! be mapped to a line of text within a file, then a field is a substring from
//! that line, with a fixed length.
//!
//! Each field is holding the substring in the **value()** and **raw_value()** properties.
//!
//! # Examples
//! ```rust
//! use std::rc::Rc;
//! use rbf::fieldtype::FieldType;
//! use rbf::field::Field;
//!
//! let ft = Rc::new(FieldType::new("I", "integer"));
//! let mut f1 = Field::new("F1", "Description for field 1", &ft, 10);
//! let mut f2 = Field::new("F2", "Description for field 2", &ft, 10);        
//! 
//! assert_eq!(&f1.name, "F1");
//! assert_eq!(&f1.description, "Description for field 1");    
//! assert_eq!(f1.length, 10);
//!
//! f1.set_value("  XX  ");    
//! assert_eq!(f1.value(), "XX");
//!
//! let other_f1 = f1.clone();
//! assert_eq!(other_f1.value(), "XX"); 
//! ```

use std::fmt;
use std::rc::Rc;
use std::cmp::max;

use fieldtype::FieldType;

// useful macro print out data enclosed by HTML tag
#[doc(hidden)]
#[macro_export]
macro_rules! html_tag {
    ($tag:expr, $slf:ident, $( $data:ident ),*) => {{
        let mut v = Vec::new();
        $(
            v.push(format!("<{}>{}</{}>", $tag, $slf.$data, $tag));
        )*
        v.join("\n")
    }}
}

#[derive(Debug)]
pub struct Field {
    /// field name
    pub name: String,
    /// field description
    pub description: String,
    /// field length in chars
    pub length: usize,    
    /// field type of this field, in chars (but not in bytes, because of UTF-8 strings)
    pub ftype: Rc<FieldType>,
    /// field value, copied as-is
    pub raw_value: String,
    /// blank-stripped field value
    pub str_value: String,
    /// offset in chars of this field within its parent record
    pub offset: usize,
    /// index of this field within its record
    pub index: usize,
    /// first position (in chars) from the beginning of the record
    pub lower_bound: usize,
    /// last position (in chars) of the field within its record
    pub upper_bound: usize,
    /// in case of a record having the same field name several times, this tracks down the field number
    pub multiplicity: usize,
    /// for display purpose (= maximum of field description and length)
    pub cell_size: usize,
}

impl Field {
    /// Creates a new field.
    ///
    /// # Arguments
    ///
    /// * `name` - name of the field
    /// * `description`: description of the field
    /// * `FieldType` fieldtype: format of the field (type of data found in the field)
    /// * `length`: number of chars of the field
    ///
    /// # Panics
    /// If `name` is empty or `length` is 0
    pub fn new(name: &str, description: &str, ftype: &Rc<FieldType>, length: usize) -> Field {
        // test arguments: non-sense to deal with empty data
        if name.is_empty() {
            panic!("Cannot create Field with an empty name!");
        }
        if length == 0 {
            panic!("Cannot create Field with a null length!");
        }

        // initialize all relevant members
        Field {
            name: name.to_string(),
            description: description.to_string(),
            length: length,
            ftype: ftype.clone(), 
            raw_value: String::new(),
            str_value: String::new(),
            offset: 0,
            index: 0,
            lower_bound: 0,
            upper_bound: 0,
            multiplicity: 0,
            cell_size: max(length, name.len()),
        }

    }

    pub fn new_with_offset(name: &str, description: &str, ftype: &Rc<FieldType>, 
        lower_offset: usize, upper_offset: usize) -> Field {
        // test arguments: non-sense to deal with empty data
        if name.is_empty() {
            panic!("Cannot create Field with an empty name!");
        }
        // sanity check
        if lower_offset > upper_offset {
            panic!("Error creating field {}: lower offset {} > upper offset {}", name, lower_offset, upper_offset);
        }  

        // calculate length & initialize all relevant members
        let length = upper_offset-lower_offset+1;

        Field {
            name: name.to_string(),
            description: description.to_string(),
            length: length,
            ftype: ftype.clone(), 
            raw_value: String::new(),
            str_value: String::new(),
            offset: 0,
            index: 0,
            lower_bound: lower_offset,
            upper_bound: upper_offset,
            multiplicity: 0,
            cell_size: max(length, name.len()),
        }

    }

    /// Sets the value which is blank-stripped and also kept asis in the **raw_value** struct field.
    pub fn set_value(&mut self, val: &str) {
        self.str_value = String::from(val.trim());
        self.raw_value = String::from(val);        
    }

    /// Returns the field value.
    pub fn value(&self) -> &String {
        &self.str_value
    } 

    /// Returns the total number of chars in the fields.
    pub fn len(&self) -> usize {
        self.length
    }  

    /// Prints out field data as an HTML table row (useful for debugging).
    pub fn as_html(&self) {
        println!("<tr>");
        println!("{}", html_tag!("td", self, 
            name, description, length, ftype, raw_value, str_value, offset, index, lower_bound, upper_bound));
        println!("</tr>");       
    }
}

// implement display trait
impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "name: <{}>, description: <{}>, length: <{}>, field type: {}, raw_value=<{}>, str_value=<{}>, offset=<{}>, index=<{}>, lower_bound=<{}>, upper_bound=<{}>", 
            self.name, self.description, self.length, self.ftype, self.raw_value, self.str_value, 
            self.offset, self.index, self.lower_bound, self.upper_bound
        )
    }
}

// implement clone
impl Clone for Field {
    fn clone(&self) -> Field {
        let mut cloned = Field::new(&self.name, &self.description, &self.ftype, self.length);

        // copy other fields which can be potentially already set
        cloned.raw_value = self.raw_value.clone();
        cloned.str_value = self.str_value.clone();      
        cloned.offset = self.offset;  
        cloned.index = self.index; 
        cloned.lower_bound = self.lower_bound;
        cloned.upper_bound = self.upper_bound;  
        cloned.multiplicity = self.multiplicity;                                          

        cloned
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use fieldtype::FieldType;
    use field::Field;

    #[test]
    fn field_cons_offset() {
        let ft = Rc::new(FieldType::new("I", "integer"));
        let mut f1 = Field::new_with_offset("F1", "Description for field 1", &ft, 5, 10);     
        
        assert_eq!(&f1.name, "F1");
        assert_eq!(&f1.description, "Description for field 1");    
        assert_eq!(f1.length, 6);
    }

    #[test]
    fn field_cons_with_length() {
        let ft = Rc::new(FieldType::new("I", "integer"));
        let mut f1 = Field::new("F1", "Description for field 1", &ft, 10);     
        
        assert_eq!(&f1.name, "F1");
        assert_eq!(&f1.description, "Description for field 1");    
        assert_eq!(f1.length, 10);

        // utf-8
        f1.set_value("  αβ  ");    
        assert_eq!(f1.value(), "αβ");

        // ascii
        f1.set_value("  XX  ");    
        assert_eq!(f1.value(), "XX");        

        let other_f1 = f1.clone();
        assert_eq!(other_f1.value(), "XX");  
    }

    #[test]
    #[should_panic]
    #[allow(unused_variables)]    
    fn field_badcons() {
        let ft = Rc::new(FieldType::new("I", "integer"));

        let f1 = Field::new("F1", "Description for field 1", &ft, 0); 
        let f2 = Field::new("", "Description for field 1", &ft, 10);            
    }      
}