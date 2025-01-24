extern crate cpal;

use std::thread;
use clap::{value_parser, ArgAction};
use colored::*;
use cpal::{traits::{DeviceTrait, StreamTrait}, SampleRate, StreamConfig};

pub mod prompts;

const DEFAULT_SAMPLE_RATE: u32 = 44100;
const DEFAULT_BUFFER_SIZE: u32 = 256; 

fn main() {
    let matches = clap::Command::new("lunatech")
        .version("1.0")
        .author("Koris")
        .about("lorem ipsum dolor")
        .arg(
            clap::Arg::new("sample_rate")
                .short('r')
                .long("sample_rate")
                .help("Sets the sample rate")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32))
        )
        .arg(
            clap::Arg::new("buffer_size")
                .short('b')
                .long("buffer_size")
                .help("Sets the buffer size")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32))
        )
        .get_matches();
    let sample_rate = matches.get_one::<u32>("sample_rate").unwrap_or(&DEFAULT_SAMPLE_RATE);
    let buffer_size = matches.get_one::<u32>("buffer_size").unwrap_or(&DEFAULT_BUFFER_SIZE);
    

    let host = cpal::default_host();
    let device = prompts::select_device_by_prompt(&host);
    monitor_device(&device, *sample_rate, *buffer_size);
}

fn monitor_device(device: &cpal::Device, sample_rate: u32, buffer_size: u32) {
    print!("Monitoring device: ");
    println!("{}", device.name().unwrap().green().bold());

    let config = StreamConfig {
        channels: 1_u16,// device.default_output_config().unwrap().channels(), TODO: ???
        sample_rate: SampleRate(sample_rate), 
        buffer_size: cpal::BufferSize::Fixed(buffer_size),
    };

    let mut last_time = std::time::Instant::now();
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {   
            let latency = last_time.elapsed().as_secs_f32();
            last_time = std::time::Instant::now();
            let framerate = 1_f32 / latency;
            println!("{:?} frames/sec", framerate.round());
        },
        move |_| {
            // TODO: Error handling
        },
        None
    ).unwrap();

    stream.play().unwrap();
    thread::park();
}
