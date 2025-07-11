use std::env;
use std::thread;
use clap::{ value_parser, ArgAction };
use colored::Colorize;
use lt_server::device_monitor::DeviceMonitor;
use lt_server::server::LunaTechServer;
use lt_server::app;

fn main() {
    let matches = clap::Command
        ::new("lunatech server")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Koris")
        .about("Realtime audio analysis server")
        .arg(
            clap::Arg
                ::new("sample_rate")
                .short('r')
                .long("sample_rate")
                .help("Sets the sample rate")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32))
        )
        .arg(
            clap::Arg
                ::new("buffer_size")
                .short('b')
                .long("buffer_size")
                .help("Sets the buffer size")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32))
        )
        .arg(
            clap::Arg
                ::new("port")
                .short('p')
                .long("port")
                .help("Set the port to broadcast on")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u16))
        )
        .arg(
            clap::Arg
                ::new("gain")
                .allow_hyphen_values(true)
                .short('g')
                .long("gain")
                .help("Sets the gain")
                .action(ArgAction::Set)
                .value_parser(value_parser!(f32))
        )
        .arg(
            clap::Arg
                ::new("headless")
                .short('H')
                .long("HEADLESS")
                .help("Enable headless mode; server starts by default")
                .action(ArgAction::SetTrue)
        )
        .arg(
            clap::Arg
                ::new("input_mode")
                .short('I')
                .long("I")
                .help("Monitor input device instead of output device")
                .action(ArgAction::SetTrue)
        )
        .get_matches();
    println!(
        "{}{} Server {}",
        "Luna".red().bold(),
        "Tech".purple().bold(),
        env!("CARGO_PKG_VERSION")
    );
    println!("Developed by {}", env!("CARGO_PKG_AUTHORS"));
    println!();

    let mut lt_server: Option<LunaTechServer> = None;
    let mut device_monitor: Option<DeviceMonitor> = None;

    let default_buffer_size = matches.get_one::<u32>("buffer_size").unwrap_or(&app::DEFAULT_BUFFER_SIZE);
    let default_port = matches.get_one::<u16>("port").unwrap_or(&app::DEFAULT_PORT);
    let default_gain = matches.get_one::<f32>("gain").unwrap_or(&app::DEFAULT_GAIN).clamp(-50.0, 50.0);

    let mut lt_server_opts = app::LTServerOpts {
        buffer_size: *default_buffer_size,
        port: *default_port,
        gain: default_gain,
        headless: matches.get_flag("headless"),
        input_mode: matches.get_flag("input_mode"),
        lt_server_state: app::LTServerState::Stopped,
    };

    if lt_server_opts.headless {
        app::start_lt_server(&mut lt_server_opts, &mut lt_server, &mut device_monitor);
    }

    if let Some(device_monitor) = &mut device_monitor {
        device_monitor.start_device_monitor();
    }

    if lt_server_opts.headless {
        thread::park();
    }

    let native_opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::Vec2::new(500.0, 500.0)),
        centered: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        env!("CARGO_PKG_NAME"),
        native_opts,
        Box::new(|_cc|
            Ok(
                Box::new(app::LTServerApp {
                    restart_queued: false,
                    lt_server: lt_server.take(),
                    lt_device_monitor: device_monitor.take(),
                    port_string: lt_server_opts.port.to_string(),
                    buffer_size_string: lt_server_opts.buffer_size.to_string(),
                    lt_server_opts,
                    timeout: None,
                })
            )
        )
    );
}