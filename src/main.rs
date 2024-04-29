use clap::Parser;

use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use std::collections::HashMap;
use std::error;
use std::io::Read;
use memchr::memchr;
extern crate num_cpus;

use ahash::RandomState;
mod util;

const READ_BUF_SIZE: usize = 128 * 1024; // 128 KiB


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

fn process_chunk(data: &[u8], result: &mut HashMap::<Box<[u8]>, StationValues, RandomState>) -> () {
    let mut buffer = &data[..];

   loop {
        // Why did using find_seperator here degrade performance
        match memchr(b';', &buffer) {
            
            None => {
                break;
            }
            Some(comma_seperator) => {
                
                let end = memchr(b'\n', &buffer[comma_seperator..]).unwrap();
                let name = &buffer[..comma_seperator];
                let value = &buffer[comma_seperator+1..comma_seperator+end];
                let value : f32 = fast_float::parse(value).expect("Failed to parse value");

                result
                    .entry(name.into())
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
    // result
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    use std::time::Instant;
    let now = Instant::now();

    let mut file = std::fs::File::open(&args.file).expect("Failed to open file");
    
    let (sender, receiver) = crossbeam_channel::bounded::<Box<[u8]>>(1_000);
    let n_threads = std::thread::available_parallelism().unwrap().into();
    let mut handles = Vec::with_capacity(n_threads);
    for _ in 0..n_threads{
        let receiver = receiver.clone();
        let handle = std::thread::spawn(move || {
            let mut result = HashMap::<Box<[u8]>, StationValues, RandomState>::default();
            for buf in receiver {
                process_chunk(&buf, &mut result);
            }
            result
        });
        handles.push(handle);
    }

    let mut buf = vec![0; READ_BUF_SIZE];
    let mut unprocessed_buffer :Vec<u8> = Vec::new();
    loop {
        let bytes_read = file.read(&mut buf[..]).expect("Failed to read file");
        if bytes_read == 0 {
            break;
        }

      let actual_buf = &mut buf[..bytes_read];  

        let last_new_line_index = match find_new_line_pos(&actual_buf){
            Some(index) => {
                index
            },
            None => {
                // println!("No newline found in buffer");
                // No newline found in the buffer. Store all the bytes in unprocessed_buffer
                // and continue reading the file
                // TODO: handle this case
                unprocessed_buffer.append(&mut actual_buf.to_owned());
                continue;
            }
        };

        if bytes_read == last_new_line_index + 1 {
            // If the buffer is full, then we can safely assume that the last byte is a newline
            // and we can process the buffer

            if unprocessed_buffer.len() != 0 {
                
                unprocessed_buffer.append(&mut actual_buf[..(last_new_line_index+1)].to_owned());
                let buf_boxed = Box::<[u8]>::from(&unprocessed_buffer[..]);
                sender.send(buf_boxed).expect("Failed to send buffer");
                unprocessed_buffer.clear();
            } else {
                let buf_boxed = Box::<[u8]>::from(&actual_buf[..(last_new_line_index+1)]);
                sender.send(buf_boxed).expect("Failed to send buffer");

            }
        } else {
            // If the buffer is not full, then we can't assume that the last byte is a newline
            // We need to store the bytes that are not processed in unprocessed_buffer
            // and continue reading the file

            // Send chunk till last new line
            if unprocessed_buffer.len() != 0 {
                unprocessed_buffer.append(&mut actual_buf[..(last_new_line_index+1)].to_owned());
                let buf_boxed = Box::<[u8]>::from(&unprocessed_buffer[..]);
                sender.send(buf_boxed).expect("Failed to send buffer");
                unprocessed_buffer.clear();
            } else {
                let buf_boxed = Box::<[u8]>::from(&actual_buf[..(last_new_line_index+1)]);
                sender.send(buf_boxed).expect("Failed to send buffer");
                unprocessed_buffer.append(&mut actual_buf[(last_new_line_index+1)..].to_vec());
            }       
            
        }

    }    
    drop(sender);

    let mut result = HashMap::<Box<[u8]>, StationValues, RandomState>::default();
    for  handle in handles{
        let map = handle.join().unwrap();
        for (station_name, station_values) in map.into_iter() {
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
        // println!("{:?};{:?};{:?};{:?}", from_utf8(_name), _min, _max, _mean);
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);


    Ok(())
}


fn find_new_line_pos(bytes: &[u8]) -> Option<usize> {
    // In this case (position is not far enough),
    // naive version is faster than bstr (memchr)
    bytes.iter().rposition(|&b| b == b'\n')
}