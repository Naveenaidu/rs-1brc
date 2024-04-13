use clap::Parser;

use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use std::{collections::HashMap, error, io::{BufRead, BufReader}};
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
    // We want all values rounded to 1 decimal place  using the semantics of IEEE 754 rounding-direction "roundTowardPositive"
    // Note: We use Decimal instaed of Floats, because it's easier to round up the fractional part of Decimals instead of floats
    // Read later: https://users.rust-lang.org/t/why-doesnt-round-have-an-argument-for-digits/100688/24
    min: Decimal,
    max: Decimal,
    mean: Decimal,
    count: Decimal,
}

fn read_line(data: String) -> (String, Decimal) {
    let parts: Vec<&str> = data.split(';').collect();
    let station_name = parts[0].to_string();
    let value = parts[1].parse::<Decimal>().expect("Failed to parse value");
    (station_name, value)
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let file = std::fs::File::open(&args.file).expect("Failed to open file");
    let reader = BufReader::new(file);

    // let mut result: HashMap<String, StationValues> = HashMap::new();
    let mut result: FxHashMap<String, StationValues> = FxHashMap::default();

    // TODO: Naveen: Instead of lines() use read_line(). This will not allocate a new string
    for line in reader.lines(){
        let line = line.expect("Failed to read line");
        let (station_name, value) = read_line(line);

        result
            .entry(station_name)
            .and_modify(|e| {
                if value < e.min {
                    e.min = value;
                }
                if value > e.max {
                    e.max = value;
                }
                e.mean = e.mean + value;
                e.count += dec!(1);
            })
            .or_insert(StationValues {
                min: value,
                max: value,
                mean: value,
                count: dec!(1),
            });

    }

    for (_, station_values) in result.iter_mut() {
        station_values.max = station_values
            .max
            .round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        station_values.min = station_values
            .min
            .round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        station_values.mean = (station_values.mean / station_values.count)
            .round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
    }
    // print!("{:?}", result);

    


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
