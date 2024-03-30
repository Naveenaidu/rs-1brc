use crate::StationValues;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn read_test_output_file(file_name: String) -> HashMap<String, StationValues> {
    let data = std::fs::read_to_string(file_name).expect("Failed to read file");
    // remove whitespace and braces
    // {Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9} => Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9
    let data_without_braces = data
        .trim_start()
        .trim_end()
        .trim_matches(['{', '}'].as_ref());

    // split the data by comma
    // Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9 => [Adelaide=15.0/15.0/15.0, Cabo San Lucas=14.9/14.9/14.9]
    let stations_data: Vec<&str> = data_without_braces.split(",").collect();
    let mut result: HashMap<String, StationValues> = HashMap::new();
    // split the data by "=" and "/" to get the station name and values
    for station_data in stations_data {
        let parts: Vec<&str> = station_data.split("=").collect();
        let station_name = parts[0].trim_start().trim_end().to_string();
        let values: Vec<&str> = parts[1].split("/").collect();
        let min = values[0].parse::<Decimal>().expect("Failed to parse min");
        let mean = values[1].parse::<Decimal>().expect("Failed to parse mean");
        let max = values[2].parse::<Decimal>().expect("Failed to parse max");
        result.insert(
            station_name,
            StationValues {
                min,
                max,
                mean,
                count: dec!(0),
            },
        );
    }
    result
}
