// by Michael J. Simms
// Copyright (c) 2021 Michael J. Simms

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
 #![allow(dead_code)]

pub mod fit_file;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    extern crate csv;

    /// Called for each record message as it is processed.
    fn callback(timestamp: u32, global_message_num: u16, local_msg_type: u8, _message_index: u16, fields: Vec<crate::fit_file::FitFieldValue>, data: &mut Context) {
        if global_message_num == crate::fit_file::GLOBAL_MSG_NUM_SESSION {
            let msg = crate::fit_file::FitSessionMsg::new(fields);
            let sport_names = crate::fit_file::init_sport_name_map();
            let sport_id = msg.sport.unwrap();

            println!("[Sport Message] {}", sport_names.get(&sport_id).unwrap());
        }
        else if global_message_num == crate::fit_file::GLOBAL_MSG_NUM_RECORD {
            let msg = crate::fit_file::FitRecordMsg::new(fields);
            let mut latitude = 0.0;
            let mut longitude = 0.0;
            let mut altitude = 0.0;
            let mut power = 0;
            let mut valid_location = true;

            match msg.position_lat {
                Some(res) => {

                    // Make sure we have a valid reading.
                    if res != 0x7FFFFFFF {
                        latitude = crate::fit_file::semicircles_to_degrees(res);
                    }
                    else {
                        valid_location = false;
                    }
                }
                None => {
                    valid_location = false;
                }
            }
            match msg.position_long {
                Some(res) => {

                    // Make sure we have a valid reading.
                    if res != 0x7FFFFFFF {
                        longitude = crate::fit_file::semicircles_to_degrees(res);
                    }
                    else {
                        valid_location = false;
                    }
                }
                None => {
                    valid_location = false;
                }
            }
            match msg.altitude {
                Some(res) => {

                    // Make sure we have a valid reading.
                    if res != 0xFFFF {
                        altitude = (res as f64 / 5.0) - 500.0;
                    }
                }
                None => {
                }
            }
            match msg.power {
                Some(res) => {
                    if res != 0xFFFF {
                        power = res;
                    }
                }
                None => {
                }
            }

            // Increment the number of records processed.
            data.num_records_processed = data.num_records_processed + 1;
            data.accumulated_power = data.accumulated_power + power as u64;

            if valid_location {
                println!("[Record Message] Timestamp: {} Latitude: {} Longitude: {} Altitude: {}", timestamp, latitude, longitude, altitude);
            }
            else {
                println!("[Record Message] Invalid location data");
            }
        }
        else if global_message_num == crate::fit_file::GLOBAL_MSG_NUM_LENGTH {
            // Increment the number of records processed.
            data.num_length_msgs_processed = data.num_length_msgs_processed + 1;
        }
        else {
            let global_message_names = crate::fit_file::init_global_msg_name_map();
            let mut field_num = 1;

            match global_message_names.get(&global_message_num) {
                Some(name) => println!("[{} Message] Timestamp {}, Values: ", name, timestamp),
                None => println!("[Global Message Num {} Local Message Type {}] Timestamp {}, Values: ", global_message_num, local_msg_type, timestamp)
            }

            for field in fields {
                print!("   ({}) Base Type: {}, Value: ", field_num, field.base_type);

                match field.type_enum {
                    crate::fit_file::FieldType::FieldTypeNotSet => { print!("[not set] "); },
                    crate::fit_file::FieldType::FieldTypeUInt => { print!("{} ", field.value_uint); },
                    crate::fit_file::FieldType::FieldTypeSInt => { print!("{} ", field.value_sint); },
                    crate::fit_file::FieldType::FieldTypeFloat => { print!("{} ", field.value_float); },
                    crate::fit_file::FieldType::FieldTypeByteArray => {
                        for byte in field.value_byte_array.iter() {
                            print!("{:#04x} ", byte);
                        }
                    },
                    crate::fit_file::FieldType::FieldTypeStr => { print!("\"{}\" ", field.value_string); },
                }

                field_num = field_num + 1;
                println!("");
            }
            println!("");
        }
    }

    /// Context structure. An instance of this will be passed to the parser and ultimately to the callback function so we can use it for whatever.
    struct Context {
        num_records_processed: u16,
        num_length_msgs_processed: u16,
        accumulated_power: u64
    }

    impl Context {
        pub fn new() -> Self {
            let context = Context{ num_records_processed: 0, num_length_msgs_processed: 0, accumulated_power: 0 };
            context
        }
    }

    #[test]
    fn file1_zwift() {
        let file = std::fs::File::open("tests/20210218_zwift.fit").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut context = Context::new();
        let fit = crate::fit_file::read(&mut reader, callback, &mut context);

        match fit {
            Ok(fit) => {
                print!("FIT File Header: ");
                fit.header.print();
                println!("");
                println!("Num records processed: {}", context.num_records_processed);
                assert!(context.num_records_processed == 1163);
            }
            _ => { println!("Error"); },
        }
    }

    #[test]
    fn file2_bike() {
        let file = std::fs::File::open("tests/20191117_bike_wahoo_elemnt.fit").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut context = Context::new();
        let fit = crate::fit_file::read(&mut reader, callback, &mut context);

        match fit {
            Ok(fit) => {
                print!("FIT File Header: ");
                fit.header.print();
                println!("");
                println!("Num records processed: {}", context.num_records_processed);
                assert!(context.num_records_processed == 4876);
            }
            _ => { println!("Error"); },
        }
    }

    #[test]
    fn file3_swim() {
        let file = std::fs::File::open("tests/20200529_short_ocean_swim.fit").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut context = Context::new();
        let fit = crate::fit_file::read(&mut reader, callback, &mut context);

        match fit {
            Ok(fit) => {
                print!("FIT File Header: ");
                fit.header.print();
                println!("");
                println!("Num records processed: {}", context.num_records_processed);
                assert!(context.num_records_processed == 179);
            }
            _ => (),
        }
    }

    #[test]
    fn file4_run_with_power() {
        let file = std::fs::File::open("tests/20210507_run_coros_pace_2.fit").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut context = Context::new();
        let fit = crate::fit_file::read(&mut reader, callback, &mut context);

        match fit {
            Ok(fit) => {
                print!("FIT File Header: ");
                fit.header.print();
                println!("");
                println!("Num records processed: {}", context.num_records_processed);
                println!("Accumulated power: {}", context.accumulated_power);
                assert!(context.num_records_processed == 2364);
                assert!(context.accumulated_power == 634203);
            }
            _ => (),
        }
    }

    #[test]
    fn file5_track_run() {
        let file = std::fs::File::open("tests/20210610_track_garmin_fenix_6.fit").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut context = Context::new();
        let fit = crate::fit_file::read(&mut reader, callback, &mut context);

        match fit {
            Ok(fit) => {
                print!("FIT File Header: ");
                fit.header.print();
                println!("");
                println!("Num records processed: {}", context.num_records_processed);
                assert!(context.num_records_processed == 1672);
            }
            _ => (),
        }
    }

    #[test]
    fn file5_pool_swim() {
        let file = std::fs::File::open("tests/20210709_pool_swim.fit").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut context = Context::new();
        let fit = crate::fit_file::read(&mut reader, callback, &mut context);

        match fit {
            Ok(fit) => {
                print!("FIT File Header: ");
                fit.header.print();
                println!("");
                println!("Num records processed: {}", context.num_length_msgs_processed);
                assert!(context.num_length_msgs_processed == 55);
            }
            _ => (),
        }
    }

    fn convert_to_camel_case(name: &String) -> String {
        let mut new_name = String::new();
        let mut need_upper_case = true;

        for c in name.chars() { 
            if need_upper_case {
                new_name.push(c.to_ascii_uppercase());
                need_upper_case = false;
            }
            else if c == '_' {
                need_upper_case = true;
            }
            else {
                new_name.push(c);
            }
        }
        new_name
    }

    fn print_message_struct(name: String, field_map: &HashMap::<String, (u8, String)>) {
        let mut struct_name: String = "Fit".to_string();
        struct_name.push_str(&convert_to_camel_case(&name));
        struct_name.push_str("Msg");

        println!("pub struct {} {{", struct_name);
        for (field_name, (_field_id, field_type)) in field_map {
            println!("    pub {}: Option<{}>,", field_name, *field_type);
        }
        println!("}}");
        println!("");
        println!("impl {} {{", struct_name);
        println!("");
        println!("    /// Constructor: Takes the fields that were read by the file parser and puts them into a structure.");
        println!("    pub fn new(fields: Vec<FitFieldValue>) -> Self {{");
        print!("        let mut msg = {} {{ ", struct_name);
        let mut split_count = 0;
        for (field_name, _field_details) in field_map {
            print!("{}: None, ", field_name);
            if split_count % 3 == 0 {
                println!("");
                print!("            ");
            }
            split_count = split_count + 1;
        }
        println!("");
        println!("        }};");
        println!("");
        println!("        for field in fields {{");
        println!("            if !field.is_dev_field {{");
        println!("                match field.field_def {{");
        for (field_name, (field_id, field_type)) in field_map.iter() {
            println!("                    {} => {{ msg.{} = Some(field.get_{}()); }},", field_id, field_name, *field_type);
        }
        println!("");
        println!("                }}");
        println!("            }}");
        println!("        }}");
        println!("        msg");
        println!("    }}");
        println!("}}");
        println!("");
    }

    #[test]
    fn create_message_structs() {
        let file_path = "tests/Messages-Table.csv";
        let file = match std::fs::File::open(&file_path) {
            Err(why) => panic!("Couldn't open {} {}", file_path, why),
            Ok(file) => file,
        };

        let mut reader = csv::Reader::from_reader(file);
        let mut current_msg_name = String::new();
        let mut field_map = HashMap::<String, (u8, String)>::new();

        for record in reader.records() {
            let record = record.unwrap();

            // First column is the message name.
            let msg_name: String = record[0].parse().unwrap();
            if msg_name.len() > 0 {

                // Print the previous definition, if there is one.
                if current_msg_name.len() > 0 {
                    print_message_struct(current_msg_name, &field_map);
                }

                current_msg_name = String::from(msg_name);
                field_map.clear();
            }
            else {
                let field_id = &record[1];

                if field_id.len() > 0 {
                    let field_id_num: u8 = field_id.parse::<u8>().unwrap();
                    let field_name: String = record[2].parse().unwrap();
                    let mut field_type_str: String = record[3].parse().unwrap();

                    // Normalize the field type string.
                    if field_type_str == "byte" {
                        field_type_str = "u8".to_string();
                    }
                    else if field_type_str == "uint8" {
                        field_type_str = "u8".to_string();
                    }
                    else if field_type_str == "uint8z" {
                        field_type_str = "u8".to_string();
                    }
                    else if field_type_str == "uint16" {
                        field_type_str = "u16".to_string();
                    }
                    else if field_type_str == "uint16z" {
                        field_type_str = "u16".to_string();
                    }
                    else if field_type_str == "uint32" {
                        field_type_str = "u32".to_string();
                    }
                    else if field_type_str == "uint32z" {
                        field_type_str = "u32".to_string();
                    }
                    else if field_type_str == "sint8" {
                        field_type_str = "i8".to_string();
                    }
                    else if field_type_str == "sint16" {
                        field_type_str = "i16".to_string();
                    }
                    else if field_type_str == "sint32" {
                        field_type_str = "i32".to_string();
                    }
                    else if field_type_str == "float32" {
                        field_type_str = "f32".to_string();
                    }
                    else if field_type_str == "float64" {
                        field_type_str = "f64".to_string();
                    }

                    field_map.insert(field_name, (field_id_num, field_type_str));
                }
            }
        }
    }
}
