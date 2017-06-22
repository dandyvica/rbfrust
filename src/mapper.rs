use regex::Regex; 

/// Convenient conversion from a string ref.
pub type RecordHasher = Box<Fn(&str) -> String>;

pub struct RecordMapper {
    pub hasher: RecordHasher,
}

/// Default closure is the identity function
impl Default for RecordMapper {
    fn default() -> RecordMapper { 
        RecordMapper{ hasher: Box::new(|x: &str| x.to_string()) }
    }
}

/// Builds the closure used to map a line to a record ID.
/// # Example
/// ```rust
/// use rbf::mapper::RecordMapper;
///
/// // our test string
/// let s = "01XX02AAAAAAAAAAAAAAAAAAA";
/// 
/// // type 0
/// let m1 = RecordMapper::from("type:0 map:DUMMY_RECORD_ID");
/// assert_eq!((m1.hasher)(s), "DUMMY_RECORD_ID");
/// 
/// // type 1
/// let m2 = RecordMapper::from("type:1 map:0..2");
/// assert_eq!((m2.hasher)(s), "01");
/// 
/// // type 2
/// let m3 = RecordMapper::from("type:2 map:0..2,4..6");
/// assert_eq!((m3.hasher)(s), "0102");
/// ```
#[allow(unused_variables)]
impl<'a> From<&'a str> for RecordMapper {
    fn from(original: &'a str) -> RecordMapper {
        let mapper_reg = Regex::new(r"^type:(?P<h_type>\d)\s+map:\s*(?P<h_value>[\w\.,]+)\s*$").unwrap();
        let caps = mapper_reg.captures(original).unwrap();

        match caps["h_type"].parse::<usize>().unwrap() {
            0 => {
                // in this case, closure is just returning a constant string
                let constant = caps["h_value"].to_string();

                RecordMapper{ 
                    hasher: Box::new(move |x: &str| constant.clone()),
                }
            },
            1 => {
                let range_reg = Regex::new(r"(?P<r_inf>\d+)\.\.(?P<r_sup>\d+)").unwrap();
                let caps_range = range_reg.captures(&caps["h_value"]).unwrap();
                let range = (
                    caps_range["r_inf"].parse::<usize>().unwrap(),
                    caps_range["r_sup"].parse::<usize>().unwrap(),
                );

                RecordMapper{ 
                    hasher: Box::new(move |x: &str| x[range.0 .. range.1].to_string()),
                }                
            },
            2 => {
                let dual_range_reg = Regex::new(r"(?P<r1_inf>\d+)\.\.(?P<r1_sup>\d+)\s*,\s*(?P<r2_inf>\d+)\.\.(?P<r2_sup>\d+)").unwrap();
                let caps_dual_range = dual_range_reg.captures(&caps["h_value"]).unwrap();
                let dual_range = (
                    caps_dual_range["r1_inf"].parse::<usize>().unwrap(),
                    caps_dual_range["r1_sup"].parse::<usize>().unwrap(),
                    caps_dual_range["r2_inf"].parse::<usize>().unwrap(),
                    caps_dual_range["r2_sup"].parse::<usize>().unwrap(),
                );

                RecordMapper{ 
                    hasher: Box::new(move |x: &str|
                        { 
                            let mut s = String::with_capacity(20);
                            s.push_str(&x[dual_range.0 .. dual_range.1]);
                            s.push_str(&x[dual_range.2 .. dual_range.3]);
                            s
                        }
                    )
                }                
            }
            _ => panic!("Unknown mapper pattern {}", original),
        }
    }
}

#[cfg(test)]
mod tests {
    use mapper::RecordMapper;

    #[test]
    fn mapper_test() {
        // our tst string
        let s = "01XX02AAAAAAAAAAAAAAAAAAA";

        // type 0
        let m1 = RecordMapper::from("type:0 map:DUMMY_RECORD_ID");
        assert_eq!((m1.hasher)(s), "DUMMY_RECORD_ID");

        // type 1
        let m2 = RecordMapper::from("type:1 map:0..2");
        assert_eq!((m2.hasher)(s), "01");

        // type 2
        let m3 = RecordMapper::from("type:2 map:0..2,4..6");
        assert_eq!((m3.hasher)(s), "0102");
    }  

    #[test]
    #[should_panic]
    #[allow(unused_variables)]    
    fn field_badcons() {
        let m = RecordMapper::from("type:3 map:?");
    }       
      
}