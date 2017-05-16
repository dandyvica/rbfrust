//! Represents a basic type used by fields. Each field can be associated with a standard type,
//! which defines the type data it holds.
//!
//! 5 different types can be used, but it can be easily extended if desired:
//!
//!  * `string`
//!  * `integer`
//!  * `decimal`
//!  * `date`
//!  * `time`
//!
//! # Examples
//! ```rust
//! use rbf::fieldtype::{BaseType, FieldType};
//!
//! let ft = FieldType::new("I", "integer");
//!
//! assert_eq!(&ft.id, "I");
//! assert_eq!(ft.base_type, BaseType::Integer); 
//! ```

use std::fmt;

/// List all possible field types when built from a string
const POSSIBLE_TYPES: [&str; 5] = ["string", "decimal", "integer", "date", "time"];

#[derive(Debug)]
#[derive(PartialEq)]
/// This is the list of all possible core field types.
pub enum BaseType {
    String,
    Decimal,
    Integer,
    Date,
    Time
}

/// Convenient conversion from a string ref.
impl<'a> From<&'a str> for BaseType {
    fn from(original: &'a str) -> BaseType {
        match original {
            "string" => BaseType::String,
            "decimal" =>  BaseType::Decimal,
            "integer" => BaseType::Integer,
            "date" => BaseType::Date,
            "time" => BaseType::Time,
            unknown_type @ _ => panic!("<{}> is not allowed as a field type", unknown_type)
        }
    }
}


// implement display trait
impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            BaseType::String => "String",
            BaseType::Decimal => "Decimal",
            BaseType::Integer => "Integer",
            BaseType::Date => "Date",
            BaseType::Time => "Time",                                    
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug)]
pub struct FieldType {
    /// Nickname for the field type
    pub id: String,
    /// Base type (which is only limited to a set a values)
    pub base_type: BaseType,
}

impl FieldType {
    /// Creates a new `FieldType` with an ID (a kind of nickname to refer to) and
    /// a type which should in the list: string, decimal, integer, date or time.
    ///
    /// # Arguments
    ///
    /// * `id` - nickname for the field type
    /// * `string_type`: base underlying type
    ///    
    pub fn new(id: &str, string_type: &str) -> FieldType {
        // first test arguments: non-sense to deal with empty data
        if id.is_empty() {
            panic!("Cannot create FieldType with empty id!");
        }
        if string_type.is_empty() {
            panic!("Cannot create FieldType with an empty string type!");
        }
        if !POSSIBLE_TYPES.contains(&string_type) {
            panic!("<{}> is not allowed as a field type", string_type);
        }

        // according to string type, create corresponding type
        FieldType {
            id: id.to_string(), 
            base_type: BaseType::from(string_type)
        }
    }

}


// implement display trait
impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: <{}>, base type: <{}>", self.id, self.base_type)
    }
}


#[cfg(test)]
mod tests {

    use fieldtype::{BaseType, FieldType};

    #[test]
    #[should_panic]
    #[allow(unused_variables)]
    fn unknown_fieldtype() {
        let ft = FieldType::new("C", "complex");
    }

    #[test]
    fn fieldtype_1() {
        let ft = FieldType::new("I", "integer");
        assert_eq!(&ft.id, "I");
        assert_eq!(ft.base_type, BaseType::Integer);    
    }
}  