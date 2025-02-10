use std::thread;

use clap::{value_parser, ArgAction};

use colored::Colorize;
use lt_server::prompts;
use lt_server::device_monitor;
use lt_server::server;
use tokio::sync::mpsc;

const DEFAULT_SAMPLE_RATE: u32 = 44100;
const DEFAULT_BUFFER_SIZE: u32 = 256 * 4;
const CHANNEL_SIZE: usize = 100; //? Select a meaningful channel size

#[tokio::main]
async fn main() {
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
        .get_matches();
    println!("{}{} Server\n", "Luna".red().bold(), "Tech".cyan().bold());

    let sample_rate = matches.get_one::<u32>("sample_rate").unwrap_or(&DEFAULT_SAMPLE_RATE);
    let buffer_size = matches.get_one::<u32>("buffer_size").unwrap_or(&DEFAULT_BUFFER_SIZE);
    
    let host = cpal::default_host();
    let device = prompts::select_device_by_prompt(&host);

    let mut server = server::LunaTechServer::new(3000);
    let mut device_monitor = device_monitor::DeviceMonitor::new(*sample_rate, *buffer_size);
    let (thread_sender, thread_receiver) = mpsc::channel(CHANNEL_SIZE);

    device_monitor.set_thread_sender(thread_sender);
    server.set_thread_receiver(thread_receiver);

    device_monitor.build_stream_from_device(&device).unwrap_or_else(|_| { 
        panic!("{}", "Failed to set device".bold().red().to_string()) 
    });

    println!("\nStarting device monitor");
    device_monitor.start_device_monitor();
    println!("{}", "Monitor started successfully\n".cyan().bold());
        
    println!("Starting server");
    server.start_server().await;
    println!("{}", "Server started successfully\n".cyan().bold());
    
    println!("{}{} {}", "Luna".red().bold(), "Tech".cyan().bold(), "server is now running".bold()); 
    thread::park();
}