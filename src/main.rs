use std::collections::HashMap;

use clap::Parser;


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
struct StationValues{
    min: f32,
    max: f32,
    mean: f32,
    count: u32,
}

fn read_line(data: String) -> (String, f32){
    let parts: Vec<&str> = data.split(';').collect();
    let station_name = parts[0].to_string();
    let value = parts[1].parse::<f32>().expect("Failed to parse value");
    (station_name, value)
}

// Calculate the station values
// For new entry: The min and max and mean would be the same
// For existing entry: The min and max would be updated if the new value is less than min or greater than max
// TODO: mean has to be calculated at the END
fn calculate_station_values(data: String) -> HashMap<String, StationValues> {
    let mut result: HashMap<String, StationValues> = HashMap::new();
    for line in data.lines() {
        let (station_name, value) = read_line(line.to_string());
        result.entry(station_name)
            .and_modify(|e| {
                if value < e.min {
                    e.min = value;
                }
                if value > e.max {
                    e.max = value;
                }
                e.mean = e.mean + value;
                e.count += 1;
            })
            .or_insert(StationValues {
                min: value,
                max: value,
                mean: value,
                count: 1,
            });
    }

    // Calculate the mean for all entries
    for (_, station_values) in result.iter_mut() {
        station_values.mean = station_values.mean / station_values.count as f32;
    }

    result

}



fn main() {
    let args = Args::parse();
    let data = std::fs::read_to_string(args.file).expect("Failed to read file");
    let result = calculate_station_values(data);
    print!("{:?}", result);

    let test_output = read_test_output_file("tests/measurements-10.out".to_string());
    print!("{:?}", test_output);
}

fn read_test_output_file(file_name: String) -> HashMap<String, StationValues> {
    let data = std::fs::read_to_string(file_name).expect("Failed to read file");
    // remove whitespace and braces
    // {Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9} => Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9
    let data_without_braces = data.trim_start().trim_end().trim_matches(['{', '}'].as_ref());

    // split the data by comma
    // Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9 => [Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9]
    let stations_data: Vec<&str> = data_without_braces.split(",").collect();  
    let mut result: HashMap<String, StationValues> = HashMap::new();
    // split the data by "=" and "/" to get the station name and values
    for station_data in stations_data {
        let parts: Vec<&str> = station_data.split("=").collect();
        let station_name = parts[0].to_string();
        let values: Vec<&str> = parts[1].split("/").collect();
        let min = values[0].parse::<f32>().expect("Failed to parse min");
        let max = values[1].parse::<f32>().expect("Failed to parse max");
        let mean = values[2].parse::<f32>().expect("Failed to parse mean");
        result.insert(station_name, StationValues{min, max, mean, count:0});
    }
    result

    
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{calculate_station_values, read_test_output_file};


    #[test]
    fn test_measurement_data() {
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        let files = fs::read_dir(test_dir).unwrap();


        for file in files {
                let test_file_name = file.unwrap().path().to_str().unwrap().to_string();
                if test_file_name.ends_with(".out") {
                    continue;
                }
                let output_file_name = test_file_name.replace(".txt", ".out");
                print!("\nTest file: {}\n", test_file_name);
                let test_output = read_test_output_file(output_file_name);
                let data = fs::read_to_string(test_file_name.clone()).expect("Failed to read file");
                let mut result = calculate_station_values(data);
                let mut test_output_map_copy = test_output.clone();
                
                // compare two hashmaps
                for (station_name, station_values) in test_output.into_iter() {
                    let result_station_values = result.remove(&station_name).expect(("Station not found: ". to_string() + &station_name + " in result hashmap").as_str());
                    assert_eq!(station_values.min, result_station_values.min);
                    assert_eq!(station_values.max, result_station_values.max);
                    assert_eq!(station_values.mean, result_station_values.mean);
                    test_output_map_copy.remove(&station_name);
                }

                assert_eq!(result.len(), 0);
                assert_eq!(test_output_map_copy.len(), 0);

                print!("Test passed\n");
                print!("-----------------------------------\n");
        }

    }
}

