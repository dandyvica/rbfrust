use std::collections::HashMap;
use regex::Regex;  


/// Converts a comma-separated string into a vector of trimmed string refs.
/// # Example
/// ```rust
/// use rbf::util::into_field_list;
///
/// let mut s = into_field_list("AA, BB, CC, DD  ");
/// assert_eq!(s, vec!("AA","BB","CC","DD"));
/// ```
pub fn into_field_list(s: &str) -> Vec<&str> {
    let flist: Vec<_> = s.split(',').map(|f| f.trim()).collect();
    flist
}

/// Converts a pattern to a map of trimmed string refs. Key is the record name,
/// value is the vector of field names.
/// # Example
/// ```rust
/// use rbf::util::{into_field_list, into_rec_map};
///
/// let v = into_rec_map("F1:AA,  BB, CC ; F2: DD, EE, FF   ; F3: GG, HH  ");
/// assert_eq!(v.get("F1").unwrap(), &vec!("AA","BB","CC"));
/// assert_eq!(v.get("F2").unwrap(), &vec!("DD","EE","FF"));
/// assert_eq!(v.get("F3").unwrap(), &vec!("GG","HH"));
/// ```
pub fn into_rec_map(s: &str) -> HashMap<&str, Vec<&str>> {
    let mut rec_map: HashMap<&str, Vec<&str>> = HashMap::new();

    for list in s.split(";") {
        let v: Vec<_> = list.split(":").map(|f| f.trim()).collect();
        rec_map.insert(v[0], into_field_list(v[1]));
    }

    rec_map
} 

/// Convenient conversion from a string ref.
type RecordHasher = Fn(&str) -> String;

struct ClosureEnv {
    constant: String,
    range: (usize,usize),
    dual_range: (usize,usize,usize,usize),
}

struct RecordMapper {
    hasher: Box<RecordHasher>,
    env: ClosureEnv,
    orig: String,
}

/*impl<'a> From<&'a str> for RecordMapper {
    fn from(original: &'a str) -> RecordMapper {
        let mapperReg = Regex::new(r"^type:(?P<h_type>\d)\s+map:\s*(?P<h_value>[\w\.,]+)\s*$").unwrap();
        let caps = mapperReg.captures(original).unwrap();

        match caps["h_type"].parse::<usize>().unwrap() {
            0 => {
                let mut r = RecordMapper{ 
                    hasher: Box::new(|x: &str| x.to_string()),
                    env: ClosureEnv{ constant: caps["h_value"].to_string(), range:(0,0), dual_range: (0,0,0,0) },
                    orig: original.to_owned(),
                };

                r.hasher = Box::new(move |x: &str| r.env.constant);
                r
            },
            _ => panic!("Unknown mapper pattern {}", original),
        }
    }
}*/

/*pub fn into_hasher(s: &str) -> Box<RecordHash> {
    Box::new(|x: &str| String::from(&x[0..2]))
}*/


#[cfg(test)]
mod tests {
      
}