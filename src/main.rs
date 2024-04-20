use clap::Parser;

use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use std::{ error, io::{BufRead, BufReader}, str::from_utf8};
use rustc_hash::FxHashMap;

mod util;

#[derive(Parser, Debug)]
#[command(
    name = "rs-1brc",
    version = "1.0",
    about = "confusedHooman's version of 1BRC challenge"
)]
struct Args {
    #[arg(short = 'f', long, help = "Path to the measurement file")]
    file: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StationValues {
    min: f32,
    max: f32,
    mean: f32,
    count: f32,
}


fn read_line(data: &Vec<u8>) -> (Vec<u8>, f32) {
    let mut parts = data.rsplit(|&c| c == b';');
    let value_str = parts.next().expect("Failed to parse value string");
    let value = fast_float::parse(value_str).expect("Failed to parse value");
    let station_name = parts.next().expect("Failed to parse station name");
    
    (station_name.to_vec(), value)
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let file = std::fs::File::open(&args.file).expect("Failed to open file");
    let mut reader = BufReader::new(file);

    let mut result2: FxHashMap<Vec<u8>, StationValues> = FxHashMap::default();

    let mut buf = Vec::new();
    while let Ok(bytes_read) = reader.read_until(b'\n', &mut buf){
        // Reached EOF
        if bytes_read == 0 {
            break;
        }
        buf.truncate(bytes_read - 1);


        // let line = String::from_utf8_lossy(&buf).trim();
        let (station_name, value) = read_line(&buf);
        // println!("{:?} {:?}", station_name, value);
        result2
            .entry(station_name)
            .and_modify(|e| {
                if value < e.min {
                    e.min = value;
                }
                if value > e.max {
                    e.max = value;
                }
                e.mean = e.mean + value;
                e.count += 1.0;
            })
            .or_insert(StationValues {
                min: value,
                max: value,
                mean: value,
                count: 1.0,
            });
        
        buf.clear();
    }

    for (_, station_values) in result2.iter_mut() {
        // We want all values rounded to 1 decimal place  using the semantics of IEEE 754 rounding-direction "roundTowardPositive"
        // Note: We use Decimal instaed of Floats, because it's easier to round up the fractional part of Decimals instead of floats
        // Read later: https://users.rust-lang.org/t/why-doesnt-round-have-an-argument-for-digits/100688/24
        let _max =  Decimal::from_f32(station_values.max).unwrap().round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        let _min =  Decimal::from_f32(station_values.min).unwrap().round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        let _mean =  Decimal::from_f32(station_values.mean/station_values.count).unwrap().round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
    }




    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use std::{fs, path::PathBuf};

//     use crate::calculate_station_values;
//     use crate::util::read_test_output_file;

//     #[test]
//     fn test_measurement_data() {
//         let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
//         let files = fs::read_dir(test_dir).unwrap();

//         for file in files {
//             let test_file_name = file.unwrap().path().to_str().unwrap().to_string();
//             if test_file_name.ends_with(".out") {
//                 continue;
//             }
//             let output_file_name = test_file_name.replace(".txt", ".out");
//             print!("\nTest file: {}\n", test_file_name);
//             let test_output = read_test_output_file(output_file_name);
//             let data = fs::read_to_string(test_file_name.clone()).expect("Failed to read file");
//             let mut result = calculate_station_values(data);
//             let mut test_output_map_copy = test_output.clone();

//             // compare two hashmaps
//             for (station_name, station_values) in test_output.into_iter() {
//                 let result_station_values = result.remove(&station_name).expect(
//                     ("Station not found: ".to_string() + &station_name + " in result hashmap")
//                         .as_str(),
//                 );
//                 assert_eq!(station_values.min, result_station_values.min);
//                 assert_eq!(station_values.max, result_station_values.max);
//                 assert_eq!(station_values.mean, result_station_values.mean);
//                 test_output_map_copy.remove(&station_name);
//             }

//             assert_eq!(result.len(), 0);
//             assert_eq!(test_output_map_copy.len(), 0);

//             print!("Test passed\n");
//             print!("-----------------------------------\n");
//         }
//     }
// }
