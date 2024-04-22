use clap::Parser;

use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use std::{ error, io::{BufRead, BufReader, Read}, str::from_utf8};
use rustc_hash::FxHashMap;
use memchr::memchr;
extern crate num_cpus;
use std::sync::mpsc;
use std::thread;




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


fn read_line(data: &[u8]) -> (&[u8], f32) {
    let mut parts = data.rsplit(|&c| c == b';');
    let value_str = parts.next().expect("Failed to parse value string");
    // let value = fast_float::parse(value_str).expect("Failed to parse value");
    let value = match fast_float::parse(value_str) {
        Ok(v) => v,
        Err(e) => {
            println!("{:?} {:?}", from_utf8(data).unwrap(), from_utf8(value_str));
            0.0
        }
    };
    let station_name = parts.next().expect("Failed to parse station name");
    
    (station_name, value)
}

fn process_chunk(data: Vec<u8>) -> FxHashMap<Vec<u8>, StationValues> {
    let mut result: FxHashMap<Vec<u8>, StationValues> = FxHashMap::default();
    let mut buffer = &data[..];
    // println!("processing chunk");
   loop {
        match memchr(b';', &buffer) {
            None => {
                break;
            }
            Some(comma_seperator) => {
                let end = memchr(b'\n', &buffer[comma_seperator..]).unwrap();
                let name = &buffer[..comma_seperator];
                let value = &buffer[comma_seperator+1..comma_seperator+end];
                let value = fast_float::parse(value).expect("Failed to parse value");

                result
                    .entry(name.to_vec())
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
                buffer = &buffer[comma_seperator+end+1..];
            }
            
        }
    }
    // println!("done processing");
    result
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let mut file = std::fs::File::open(&args.file).expect("Failed to open file");
    let mut buffer = Vec::new();
    // println!("starting processing 2");
    file.read_to_end(&mut buffer).expect("Failed to read file");
    // println!("starting processing 1");

    let mut result: FxHashMap<Vec<u8>, StationValues> = FxHashMap::default();
    let mut buffer = &buffer[..];

    let (tx, rx) = mpsc::channel();
    // println!("starting processing");


    // count logical cores this process could try to use
    let mut chunks_to_create = num_cpus::get();
    let mut chunks_created = 0;
    // println!("chunks to create: {:?}", chunks_to_create);
    // let chunk_size = buffer.len() / num;
    while chunks_to_create > 0 && buffer.len() > 0 {
        let chunk_size = (buffer.len() / chunks_to_create) - 1;
        if buffer[chunk_size] == b'\n' {
            let chunk = buffer[..chunk_size+1].to_vec();
            // spwan a thread to process chunk
            // process_chunk(chunk);
            // println!("spawning thread");
            let tx = tx.clone();
            thread::spawn(move || {
                let val = process_chunk(chunk);
                tx.send(val).unwrap();
            });

            buffer = &buffer[chunk_size+1..];
            chunks_to_create -= 1;
            chunks_created += 1;

        } else {
            let newline = memchr(b'\n', &buffer[chunk_size..]).unwrap();
            let chunk = buffer[..chunk_size+newline+1].to_vec();
            // spwan a thread to process chunk
            // process_chunk(chunk);
            // println!("spawning thread");
            let tx = tx.clone();
            thread::spawn(move || {
                let val = process_chunk(chunk);
                tx.send(val).unwrap();
            });


            buffer = &buffer[chunk_size+newline+1..];
            chunks_to_create -= 1;
            chunks_created += 1;
        }
    }

    // println!("chunks created: {:?}", chunks_created);
    for i in 0..chunks_created{
        // println!("---------------- {:?}", i);
        let val = rx.recv().unwrap();
        
        // println!("received chunk");
        for (station_name, station_values) in val.into_iter() {
            result
                .entry(station_name)
                .and_modify(|e| {
                    if station_values.min < e.min {
                        e.min = station_values.min;
                    }
                    if station_values.max > e.max {
                        e.max = station_values.max;
                    }
                    e.mean = e.mean + station_values.mean;
                    e.count += station_values.count;
                })
                .or_insert(station_values);
        }
    }


    for (_name, station_values) in result.iter_mut() {
        // We want all values rounded to 1 decimal place  using the semantics of IEEE 754 rounding-direction "roundTowardPositive"
        // Note: We use Decimal instaed of Floats, because it's easier to round up the fractional part of Decimals instead of floats
        // Read later: https://users.rust-lang.org/t/why-doesnt-round-have-an-argument-for-digits/100688/24
        let _max =  Decimal::from_f32(station_values.max).unwrap().round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        let _min =  Decimal::from_f32(station_values.min).unwrap().round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        let _mean =  Decimal::from_f32(station_values.mean/station_values.count).unwrap().round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        // println!("{:?};{:?};{:?};{:?}", from_utf8(_name).unwrap(), _min, _mean, _max);
    }




    Ok(())
}
