use std::thread;
use clap::{value_parser, ArgAction};
use colored::Colorize;

use lt_server::prompts;
use lt_server::device_monitor;
use lt_server::server;

const DEFAULT_SAMPLE_RATE: u32 = 44100;
const DEFAULT_BUFFER_SIZE: u32 = 256 * 4;
const DEFAULT_PORT: u16 = 3000;

fn main() {
    let matches = clap::Command::new("lunatech server")
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
        .arg(
            clap::Arg::new("port")
                .short('p')
                .long("port")
                .help("Set the port to broadcast on")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u16))
        )
        .get_matches();
    println!("{}{} Server\n", "Luna".red().bold(), "Tech".purple().bold());
    
    // TODO: INTERACTIVE MODE (allow args for I/O mode and device #, or default if not specified)
    // TODO: Replace crossbeam channel with std

    let sample_rate = matches.get_one::<u32>("sample_rate").unwrap_or(&DEFAULT_SAMPLE_RATE);
    let buffer_size = matches.get_one::<u32>("buffer_size").unwrap_or(&DEFAULT_BUFFER_SIZE);
    let port = matches.get_one::<u16>("port").unwrap_or(&DEFAULT_PORT);
    
    let host = cpal::default_host();
    let device = prompts::select_device_by_prompt(&host);

    let mut server = server::LunaTechServer::new(*port);
    let mut device_monitor = device_monitor::DeviceMonitor::new(*sample_rate, *buffer_size);

    // Todo: Check if bounded is faster
    let (tx, rx) = crossbeam::channel::unbounded();

    device_monitor.set_thread_sender(tx);
    server.set_thread_receiver(rx);

    device_monitor.build_stream_from_device(&device).unwrap_or_else(|_| { 
        panic!("{}", "Failed to set device".bold().red().to_string()) 
    });

    println!("\nStarting device monitor");
    device_monitor.start_device_monitor(); 
    println!("{}", "Monitor started successfully\n".cyan().bold());
        
    println!("Starting server");
    server.start_server();
    println!("{}", "Server started successfully\n".cyan().bold());
    
    println!("{}{} {}", "Luna".red().bold(), "Tech".purple().bold(), "server is now running".bold()); 

    loop {
        thread::park();
    }
}