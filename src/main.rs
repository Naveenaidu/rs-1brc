use clap::Parser;
use opentelemetry::{
    global::{self},
    trace::{TraceContextExt, Tracer},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

use opentelemetry_semantic_conventions as semcov;
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use std::{collections::HashMap, error};

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
    print!("{:?} | parts: {:?} \n", data, parts);
    let station_name = parts[0].to_string();
    let value = parts[1].parse::<Decimal>().expect("Failed to parse value");
    (station_name, value)
}

pub fn init_tracer() {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![
                KeyValue::new(semcov::resource::SERVICE_NAME, "rs-1brc"),
                KeyValue::new(
                    semcov::resource::SERVICE_VERSION,
                    env!("CARGO_PKG_VERSION").to_string(),
                ),
            ]),
        ))
        .install_simple()
        .expect("Failed to install tracer");
    global::set_tracer_provider(tracer.provider().unwrap());
}

// Calculate the station values
// For new entry: The min and max and mean would be the same
// For existing entry: The min and max would be updated if the new value is less than min or greater than max. And mean will be calculated after getting all count
fn calculate_station_values(data: String) -> HashMap<String, StationValues> {
    let tracer = global::tracer("rs-1brc");
    let mut result: HashMap<String, StationValues> = HashMap::new();
    tracer.in_span("calculate_station_values", |_ctx| {
        for line in data.lines() {
            let (station_name, value) = read_line(line.to_string());
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

        tracer.in_span("calculate mean", |_ctx| {
            // Calculate the mean for all entries
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
        })
    });

    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();
    init_tracer();
    let tracer = global::tracer("rs-1brc");
    tracer.in_span("main", |cx| {
        cx.span().set_attribute(opentelemetry::KeyValue {
            key: "hello".into(),
            value: "world".into(),
        });

        let mut file_content = Vec::new();
        let mut file = std::fs::File::open(&args.file).expect("Failed to open file");
        std::io::Read::read_to_end(&mut file, &mut file_content)
            .expect("Failed to read file content");

        // let file_content = unsafe {String::from_utf8_unchecked(file_content)};
        // let _result = calculate_station_values(file_content);
        // print!("{:?}\n", result);

        let mut result: HashMap<String, StationValues> = HashMap::new();
        for chunk in file_content.split(|&x| x == b'\n') {
            // For last line
            if chunk.is_empty() {
                continue;
            }
            let mut line_bytes = chunk.split(|&x| x == b';');
            let station_name_bytes = line_bytes.next().expect("Failed to get station name");
            let station_name = String::from_utf8(station_name_bytes.to_vec())
                .expect("Failed to convert to string");

            let value_bytes = line_bytes.next().expect("Failed to get value");
            let value = String::from_utf8(value_bytes.to_vec())
                .expect("Failed to convert to string")
                .parse::<Decimal>()
                .expect("Failed to parse value");


            let line = String::from_utf8(chunk.to_vec()).expect("Failed to convert to string");
            print!("{:?}\n", line);
            // let (station_name, value) = read_line(line);
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





    });

    global::shutdown_tracer_provider();

    // print!("\n-----------------------------------\n");
    // let test_output = crate::util::read_test_output_file("tests/measurements-20.out".to_string());
    // print!("{:?}", test_output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::calculate_station_values;
    use crate::util::read_test_output_file;

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
                let result_station_values = result.remove(&station_name).expect(
                    ("Station not found: ".to_string() + &station_name + " in result hashmap")
                        .as_str(),
                );
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
