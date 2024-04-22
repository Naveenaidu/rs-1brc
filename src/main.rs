use clap::Parser;

use memmap::Mmap;
use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use std::error;
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

fn process_chunk(data: &[u8]) -> FxHashMap<&[u8], StationValues> {
    let mut result: FxHashMap<&[u8], StationValues> = FxHashMap::default();
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
                    .entry(name)
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

    use std::time::Instant;
    let now = Instant::now();

    let file = std::fs::File::open(&args.file).expect("Failed to open file");
    

    let mut result: FxHashMap<&[u8], StationValues> = FxHashMap::default();

    let mmap: Mmap;
    let data: &[u8];

    mmap = unsafe { Mmap::map(&file).unwrap() };
    data = &*mmap;
    let buffer = &data[..];

    let (tx, rx) = mpsc::channel();


    // count logical cores this process could try to use
    let mut chunks_to_create = num_cpus::get();
    let mut chunks_created = 0;
    let mut chunks: Vec<(usize, usize)> = Vec::new();
    let mut start: usize = 0;

    while chunks_to_create > 0 && buffer.len() > 0 {
        let chunk_size = (buffer.len() / chunks_to_create) - 1;
        if buffer[chunk_size] == b'\n' {
            chunks.push((start, chunk_size+1));
            start = chunk_size + 1;
            chunks_to_create -= 1;
            chunks_created += 1;
        } else {
            let newline = memchr(b'\n', &buffer[chunk_size..]).unwrap();
            chunks.push((start, chunk_size+newline+1));
            start = chunk_size+newline+1;
            chunks_to_create -= 1;
            chunks_created += 1;
        }
    }

    thread::scope(|scope| {
        for (start, end) in chunks {
            let tx = tx.clone();
            scope.spawn(move || {
                let val = process_chunk(&buffer[start..end]);
                tx.send(val).unwrap();
            }); 
        }
    });


// RAYON CODE
//     let values: Vec<FxHashMap<&[u8], StationValues>> = chunks.into_par_iter().map(|(start, end)| process_chunk(&buffer[start..end])).collect();

//     for val in values{
//         for (station_name, station_values) in val.into_iter() {
//             result
//                 .entry(station_name)
//                 .and_modify(|e| {
//                     if station_values.min < e.min {
//                         e.min = station_values.min;
//                     }
//                     if station_values.max > e.max {
//                         e.max = station_values.max;
//                     }
//                     e.mean = e.mean + station_values.mean;
//                     e.count += station_values.count;
//                 })
//                 .or_insert(station_values);
        
//     }
// }


   for _ in 0..chunks_created{
    let val = rx.recv().unwrap();
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
}



    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    Ok(())
}
