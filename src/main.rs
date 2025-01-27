extern crate cpal;

use clap::{value_parser, ArgAction};

pub mod prompts;
pub mod monitor;
pub mod analyzer;

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
    let device_monitor = monitor::DeviceMonitor::new(&device, *sample_rate, *buffer_size);
    device_monitor.monitor_device();
}