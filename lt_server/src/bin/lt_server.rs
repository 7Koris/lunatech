use std::env;
use std::thread;
use clap::{value_parser, ArgAction};
use colored::Colorize;
use cpal::traits::HostTrait;
use egui::Stroke;
use egui::TextFormat;
use egui::{Button, Grid, Label, RichText, Vec2};

use lt_server::device_monitor::DeviceMonitor;
use lt_server::device_monitor;
use lt_server::server;
use lt_server::server::LunaTechServer;

const DEFAULT_SAMPLE_RATE: u32 = 44100;
const DEFAULT_BUFFER_SIZE: u32 = 1024;
const DEFAULT_PORT: u16 = 3000;

#[derive(PartialEq)]
enum LTServerState {
    Running,
    Stopped,
}

struct LTServerOpts {
    sample_rate: u32,
    buffer_size: u32,
    port: u16,
    headless: bool,
    input_mode: bool,
    lt_server_state: LTServerState,
}

fn start_lt_server(lt_server_opts: &mut LTServerOpts, lt_server: &mut Option<LunaTechServer>, lt_device_monitor: &mut Option<DeviceMonitor>) { 
    if lt_server_opts.lt_server_state == LTServerState::Running {
        println!("{}", "Server is already running, please stop it first".bold().red());
        return;
    }

    let host = cpal::default_host();
    let device = if lt_server_opts.input_mode {
        host.default_input_device()
    } else {
        host.default_output_device()
    }.expect("Failed to get default device");

    *lt_server = Some(LunaTechServer::new(lt_server_opts.port));
    *lt_device_monitor = Some(DeviceMonitor::new(lt_server_opts.sample_rate, lt_server_opts.buffer_size));
    // Todo: Check if bounded is faster
    // Todo: Replace crossbeam channel with std
    let (tx, rx) = crossbeam::channel::unbounded();

    if let Some(lt_server) = lt_server {
        lt_server.set_thread_receiver(rx);
    } else {
        panic!("{}", "Failed to initialize and start server".bold().red());
    }

    if let Some(device_monitor)  = lt_device_monitor { 
        device_monitor.set_thread_sender(tx);
        device_monitor.build_stream_from_device(&device).unwrap_or_else(|_| { 
            panic!("{}", "Failed to set device".bold().red().to_string()) 
        });
    } else {
        panic!("{}", "Failed to initialize and start device monitor".bold().red());
    }

    lt_server_opts.lt_server_state = LTServerState::Running;
    println!("{}{} {}", "Luna".red().bold(), "Tech".purple().bold(), "server is now running".bold());
    if let Some(lt_server) = lt_server {
        lt_server.start_server();
    }
}

fn stop_lt_server(lt_server_opts: &mut LTServerOpts, lt_server: &mut Option<server::LunaTechServer>, device_monitor: &mut Option<device_monitor::DeviceMonitor>) {
    if lt_server_opts.lt_server_state == LTServerState::Stopped {
        println!("{}", "Server is already stopped, please start it first".bold().red());
        return;
    }

    if let Some(device_monitor) = device_monitor {
        device_monitor.stop_device_monitor();
    }

    lt_server_opts.lt_server_state = LTServerState::Stopped;
    println!("{}{} {}", "Luna".red().bold(), "Tech".purple().bold(), "server is now stopped".bold()); 
}

fn main() {
    let matches = clap::Command::new("lunatech server")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Koris")
        .about("Realtime audio analysis server")
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
        .arg(
            clap::Arg::new("headless")
                .short('H')
                .long("HEADLESS")
                .help("Enable headless mode; server starts by default")
                .action(ArgAction::SetTrue)
        )
        .arg(
            clap::Arg::new("input_mode")
                .short('I')
                .long("I")
                .help("Monitor input device instead of output device")
                .action(ArgAction::SetTrue)
        )
        .get_matches();
    println!("{}{} Server {}", "Luna".red().bold(), "Tech".purple().bold(), env!("CARGO_PKG_VERSION"));
    println!("Developed by {}", env!("CARGO_PKG_AUTHORS"));    
    println!();
    
    let mut lt_server: Option<LunaTechServer> = None;
    let mut device_monitor: Option<DeviceMonitor> = None;

    let default_sample_rate = matches.get_one::<u32>("sample_rate").unwrap_or(&DEFAULT_SAMPLE_RATE);
    let default_buffer_size = matches.get_one::<u32>("buffer_size").unwrap_or(&DEFAULT_BUFFER_SIZE);
    let default_port = matches.get_one::<u16>("port").unwrap_or(&DEFAULT_PORT);

    let mut lt_server_opts = LTServerOpts {
        sample_rate: *default_sample_rate,
        buffer_size: *default_buffer_size,
        port: *default_port,
        headless: matches.get_flag("headless"),
        input_mode: matches.get_flag("input_mode"),
        lt_server_state: LTServerState::Stopped,
    };

    if lt_server_opts.headless {
        start_lt_server(&mut lt_server_opts, &mut lt_server, &mut device_monitor);
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
        Box::new(|_cc| Ok(Box::new(LTServerApp {
            restart_queued: false,
            lt_server: lt_server.take(),  
            lt_device_monitor: device_monitor.take(),  
            port_string: lt_server_opts.port.to_string(),
            buffer_size_string: lt_server_opts.buffer_size.to_string(),
            sample_rate_string: lt_server_opts.sample_rate.to_string(),
            lt_server_opts,
            timeout: None,
    }))));
}

struct LTServerApp {
    restart_queued: bool, 
    lt_server: Option<LunaTechServer>,
    lt_device_monitor: Option<DeviceMonitor>,
    port_string: String,
    buffer_size_string: String,
    sample_rate_string: String,
    lt_server_opts: LTServerOpts,
    timeout: Option<std::time::Instant>,
}

impl eframe::App for LTServerApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut no_errors = true; 
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("LunaTech Server {}", env!("CARGO_PKG_VERSION")));

            let button_label = if self.lt_server_opts.lt_server_state == LTServerState::Running {
                egui::RichText::new("ON").color(egui::Color32::GREEN) // Green text
            } else {
                egui::RichText::new("OFF").color(egui::Color32::RED) // Red text
            };
            
            let button = egui::Button::new(button_label)
                .corner_radius(60.)
                .min_size(Vec2::new(100.0, 100.0));

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.add(button).clicked() {
                    if self.lt_server_opts.lt_server_state == LTServerState::Running {
                        stop_lt_server(&mut self.lt_server_opts, &mut self.lt_server, &mut self.lt_device_monitor);
                        self.lt_server_opts.lt_server_state = LTServerState::Stopped;
                    } else {
                        start_lt_server(&mut self.lt_server_opts, &mut self.lt_server, &mut self.lt_device_monitor);
                        self.lt_server_opts.lt_server_state = LTServerState::Running;
                    }
                };
            });

            let format = TextFormat {
                underline: Stroke {
                    width: 1.0,
                    color: egui::Color32::BLACK,
                },
                ..Default::default()
            };

            ui.add(Label::new(
                RichText::new("Settings").color(egui::Color32::WHITE).strong(),
            ));
             
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
               
                Grid::new("settings_grid")
                    .num_columns(3)
                    .spacing([30.0, 0.0])
                    .show(ui, |ui| {

                        ui.label("Audio Input Mode");  

                       // ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        if ui.add(egui::Checkbox::new(&mut self.lt_server_opts.input_mode, "")).changed() {
                            self.restart_queued = true;
                        }
                        //});

                        ui.end_row();
                        
                        ui.label("port");    
                        
                        ui.add(egui::TextEdit::singleline(&mut self.port_string)
                            .hint_text(DEFAULT_PORT.to_string())
                        );
        
                        if self.port_string.parse::<u16>().is_err() {
                            ui.add(
                                Label::new(
                                    RichText::new("Invalid port").color(egui::Color32::RED)
                                )
                            );
                            no_errors = false;
                        }

                        ui.end_row();

                        // ui.label("buffer size");    
                        // ui.add_sized([100. ,0.], egui::TextEdit::singleline(&mut self.buffer_size_string)
                        //     .hint_text(DEFAULT_BUFFER_SIZE.to_string())
                        //     //.desired_width(100.0)
                        // );

                        // if self.buffer_size_string.parse::<u32>().is_err() {
                        //     ui.add(
                        //         Label::new(
                        //             RichText::new("Invalid buffer size").color(egui::Color32::RED)
                        //         )
                        //     );
                        //     no_errors = false;
                        // }

                        // ui.end_row();
            
                        // ui.label("sample rate");    

                        // ui.add_sized([100. ,0.], egui::TextEdit::singleline(&mut self.sample_rate_string)
                        //     .hint_text(DEFAULT_SAMPLE_RATE.to_string())
                        // );

                        // if self.sample_rate_string.parse::<u32>().is_err() {
                        //     ui.add(
                        //         Label::new(
                        //             RichText::new("Invalid sample rate").color(egui::Color32::RED)
                        //         )
                        //     );
                        //     no_errors = false;
                        // }
            
                        ui.end_row();
                });
            });

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                let button = Button::new("Update Settings"); 
                if ui.add_enabled(no_errors, button).clicked() {
                    self.restart_queued = true;
                    // TODO: Visual Feedback
                }
            });

            if self.restart_queued {
                self.restart_queued = false;

                if no_errors {
                    // update settings
                    if let Ok(port) = self.port_string.parse::<u16>() {
                        self.lt_server_opts.port = port;
                    }

                    if let Ok(buffer_size) = self.buffer_size_string.parse::<u32>() {
                        self.lt_server_opts.buffer_size = buffer_size;
                    }

                    if let Ok(sample_rate) = self.sample_rate_string.parse::<u32>() {
                        self.lt_server_opts.sample_rate = sample_rate;
                    }
                }

                if self.lt_server_opts.lt_server_state == LTServerState::Running {
                    println!("Settings changed, server will restart");
                    stop_lt_server(&mut self.lt_server_opts, &mut self.lt_server, &mut self.lt_device_monitor);
                    start_lt_server(&mut self.lt_server_opts, &mut self.lt_server, &mut self.lt_device_monitor);
                }

                self.timeout = Some(std::time::Instant::now());
            }

            if let Some(timeout) = self.timeout {
                if timeout.elapsed().as_secs() < 1 {
                    ui.label(RichText::new("Success!").color(egui::Color32::GREEN));
                }
            }
            
        });
    }
}