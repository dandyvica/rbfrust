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
//! use rbf::fieldtype::{BaseDataType, FieldDataType};
//!
//! let ft = FieldDataType::new("I", "integer");
//!
//! assert_eq!(&ft.id, "I");
//! assert_eq!(ft.base_data_type, BaseDataType::Integer); 
//! ```

use std::fmt;

/// List all possible field types when built from a string
const POSSIBLE_TYPES: [&str; 5] = ["string", "decimal", "integer", "date", "time"];

#[derive(Debug)]
#[derive(PartialEq)]
/// This is the list of all possible core field types.
pub enum BaseDataType {
    String,
    Decimal,
    Integer,
    Date{ date_format: String },
    Time{ time_format: String },
}

/// Convenient conversion from a string ref.
impl<'a> From<&'a str> for BaseDataType {
    fn from(original: &'a str) -> BaseDataType {
        match original {
            "string" => BaseDataType::String,
            "decimal" =>  BaseDataType::Decimal,
            "integer" => BaseDataType::Integer,
            "date" => BaseDataType::Date{ date_format: "%D%m%s".to_string() },
            "time" => BaseDataType::Time{ time_format: "%H%M%S".to_string() },
            unknown_type @ _ => panic!("<{}> is not allowed as a field type", unknown_type)
        }
    }
}


// implement display trait
impl fmt::Display for BaseDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            BaseDataType::String => "String".to_string(),
            BaseDataType::Decimal => "Decimal".to_string(),
            BaseDataType::Integer => "Integer".to_string(),
            BaseDataType::Date{ ref date_format } => format!("Date {{ {} }}", *date_format),
            BaseDataType::Time{ ref time_format } => format!("Time {{ {} }}", *time_format),
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug)]
pub struct FieldDataType {
    /// Nickname for the field type
    pub id: String,
    /// Base type (which is only limited to a set a values)
    pub base_data_type: BaseDataType,
    /// Optional pattern which describes field format
    pub pattern: String,
}

impl FieldDataType {
    /// Creates a new `FieldDataType` with an ID (a kind of nickname to refer to) and
    /// a type which should in the list: string, decimal, integer, date or time.
    ///
    /// # Arguments
    ///
    /// * `id` - nickname for the field type
    /// * `string_type`: base underlying type
    ///    
    pub fn new(id: &str, string_type: &str) -> FieldDataType {
        // first test arguments: non-sense to deal with empty data
        if id.is_empty() {
            panic!("Cannot create FieldDataType with empty id!");
        }
        if string_type.is_empty() {
            panic!("Cannot create FieldDataType with an empty string type!");
        }
        if !POSSIBLE_TYPES.contains(&string_type) {
            panic!("<{}> is not allowed as a field type", string_type);
        }

        // according to string type, create corresponding type
        FieldDataType {
            id: id.to_string(), 
            base_data_type: BaseDataType::from(string_type),
            pattern: String::new(),
        }
    }

    /// Sets the date format for conversion to time structs.
    ///
    /// # Arguments
    ///
    /// * `date_format` - format according to strftime() call
    /// 
    pub fn set_date_format(&mut self, date_format: &str) {
        self.base_data_type = BaseDataType::Date { date_format : date_format.to_string() };
    }  

    /// Sets the time format for conversion to time structs.
    ///
    /// # Arguments
    ///
    /// * `time_format` - format according to strftime() call
    /// 
    pub fn set_time_format(&mut self, time_format: &str) {
        self.base_data_type = BaseDataType::Time { time_format : time_format.to_string() };
    } 

    pub fn set_pattern(&mut self, pattern: &str) {
        self.pattern = String::from(pattern);
    }
}


// implement display trait
impl fmt::Display for FieldDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: <{}>, base type: <{}>", self.id, self.base_data_type)
    }
}


#[cfg(test)]
mod tests {

    use fieldtype::{BaseDataType, FieldDataType};

    #[test]
    #[should_panic]
    #[allow(unused_variables)]
    fn unknown_fieldtype() {
        let ft = FieldDataType::new("C", "complex");
    }

    #[test]
    fn fieldtype_simple() {
        let ft = FieldDataType::new("I", "integer");
        assert_eq!(&ft.id, "I");
        assert_eq!(ft.base_data_type, BaseDataType::Integer);    
    }

    #[test]
    fn fieldtype_all() {
        let ft = FieldDataType::new("S", "string");
        assert_eq!(&ft.id, "S");
        assert_eq!(ft.base_data_type, BaseDataType::String);

        let ft = FieldDataType::new("N", "decimal");
        assert_eq!(&ft.id, "N");
        assert_eq!(ft.base_data_type, BaseDataType::Decimal);  

        let ft = FieldDataType::new("I", "integer");
        assert_eq!(&ft.id, "I");
        assert_eq!(ft.base_data_type, BaseDataType::Integer);  

        let ft = FieldDataType::new("D", "date");
        assert_eq!(&ft.id, "D");
        assert_eq!(ft.base_data_type, BaseDataType::Date{ date_format: "%D%m%s".to_string() });  

        let ft = FieldDataType::new("T", "time");
        assert_eq!(&ft.id, "T");
        assert_eq!(ft.base_data_type, BaseDataType::Time{ time_format: "%H%M%S".to_string() });                                      
    }    
}  