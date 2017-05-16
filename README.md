# A record-based file library
 In some industries (e.g.: airline, banking), a lot of Ascii files are exchanged using a record-based organization. 
 Usually, this kind of file is a plain vanilla file where each line is mapped to a record, and each record to a field 
 within this record. It is based on a positional organization.
 
 Those files are generally coming from a mainframe legacy system, created by some Cobol
 programs, using copybooks (akin to C structures).
 
 The way to recognize a record is generally by defining a record identifier 
 (example: first 2 characters of each line). Each record identifier defines the type of 
 the record and how it is organized.
 
 Each record is a set of contiguous Ascii fields, each field having a length (in chars), a type
 (either representing an alphanumerical or numeric field) and a length. An offset from the beginning of the record
 is also part of a field.
 
 This library allows to loop through all the records, and loop through all fields of a record.
 File data could be Ascii or UTF-8.
 
 The definition of the file structure is provided through an XML definition file.

## Layout definition file

Such a file could be easily defined by an XML layout file. 
Suppose you've got an Ascii file for some statistical data on continents, countries such as capital, population, etc. 
Such a file could look like this:

```text
CONTAsia           43820000            16920000            29.5     Shanghai            
COUNChina                         1338100000          Beijing             
COUNChina Hong Kong SAR           7000000             Hong KongR
COUNChina Macau SAR               500000              Macau City          
COUNChina Tibet                   2620000             Lhasa               
COUNJapan                         127400000           Tokyo               
COUNKorea (North)                 22800000            P'yongyang          
COUNKorea (South)                 48900000            Seoul               
COUNMongolia                      2800000             Ulaanbaatar         
COUNTaiwan                        23200000            Taipei              
COUNRussian Federation            141900000           Moscow              
COUNAfghanistan                   29100000            Kabul               
COUNBangladesh                    164400000           Dhaka               
COUNBhutan                        700000              Thimphu             
COUNIndia                         1188800000          New Delhi           
COUNIran                          75100000            Tehran              
```

The corresponding XML definition file describing this format could be:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!-- inspired from https://en.wikipedia.org/wiki/List_of_continents_by_GDP_%28nominal%29 -->
<!-- and http://www.nationsonline.org/oneworld/asia.htm -->
<rbfile
    xmlns="http://www.w3schools.com"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.w3schools.com rbf.xsd"
>

    <meta version="1.0" description="Continents, countries, cities" ignoreLine="^#" skipField="ID" mapper="type:1 map:0..4"/>

	<fieldtype name="CHAR" type="string" pattern="\w+" format=""/>
	<fieldtype name="NUM" type="decimal"/>
	<fieldtype name="INT" type="integer"/>

	<record name="CONT" description="Continent data">
		<field name="ID" description="Record ID" length="4" type="CHAR"/>
		<field name="NAME" description="Name of the continent" length="15" type="CHAR"/>
		<field name="AREA" description="Area of the continent" length="20" type="NUM"/>
		<field name="POPULATION" description="Population of the continent" length="20" type="NUM"/>
		<field name="DENSITY" description="Density per km2" length="9" type="NUM"/>
		<field name="CITY" description="Most populus city" length="20" type="CHAR"/>
	</record>

	<record name="COUN" description="Country data">
		<field name="ID" description="Record ID" length="4" type="CHAR"/>
		<field name="NAME" description="Name of the country" length="30" type="CHAR"/>
		<field name="POPULATION" description="Number of inhabitants" length="20" type="INT"/>
		<field name="CAPITAL" description="Capital of the country" length="20" type="CHAR"/>
	</record>

</rbfile>
```

## How to use it

This is an example, which just counts record occurence:

```rust
use std::env;
use std::collections::HashMap;

extern crate rbf;
use rbf::layout::Layout;
use rbf::reader::Reader;

fn main () {
    let mut nb_lines: usize = 0;
    let mut nb_records: HashMap<String, usize> = HashMap::new();

    // get arguments
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: {} layout_file data_file", args[0]);
        std::process::exit(1);
    }


    // load layout
    let layout = Layout::new(&args[1]);

    // create reader
    fn mapper(x: &str) -> &str { &x[0..2] };
    let mut reader = Reader::new(&args[2], layout, mapper);  

    // loop through records
    while let Some(rec) = reader.next() {
        nb_lines += 1;

        // if key doesn't exists, set to 1
        if nb_records.contains_key(&rec.name) {
            *nb_records.get_mut(&rec.name).unwrap() += 1;
        }
        else {
            nb_records.insert(rec.name.clone(), 1);
        }
    } 

    // print out results
    println!("Input file has {} lines", nb_lines);

    for (recname, i) in nb_records {
        println!("Number of {} records = {} ", recname, i);
    }
}
```
