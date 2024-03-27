use std::{collections::HashMap, hash::Hash};

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

#[derive(Debug)]
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
}