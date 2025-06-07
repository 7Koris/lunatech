use colored::Colorize;
use cpal::traits::HostTrait;
use egui::{ Grid, Label, RichText, Separator, Slider, Vec2 };

use crate::{ device_monitor::{ self, DeviceMonitor }, server::{ self, LunaTechServer } };

pub const DEFAULT_SAMPLE_RATE: u32 = 44100;
pub const DEFAULT_BUFFER_SIZE: u32 = 2048;
pub const DEFAULT_PORT: u16 = 3000;
pub const DEFAULT_GAIN: f32 = 0.0;

#[derive(PartialEq)]
pub enum LTServerState {
    Running,
    Stopped,
}

pub struct LTServerOpts {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub port: u16,
    pub gain: f32,
    pub headless: bool,
    pub input_mode: bool,
    pub lt_server_state: LTServerState,
}

pub struct LTServerApp {
    pub restart_queued: bool,
    pub lt_server: Option<LunaTechServer>,
    pub lt_device_monitor: Option<DeviceMonitor>,
    pub port_string: String,
    pub buffer_size_string: String,
    pub sample_rate_string: String,
    pub lt_server_opts: LTServerOpts,
    pub timeout: Option<std::time::Instant>,
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

            let button = egui::Button
                ::new(button_label)
                .corner_radius(60.0)
                .min_size(Vec2::new(100.0, 100.0));

            ui.add(Separator::default());

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.add(button).clicked() {
                    if self.lt_server_opts.lt_server_state == LTServerState::Running {
                        stop_lt_server(
                            &mut self.lt_server_opts,
                            &mut self.lt_server,
                            &mut self.lt_device_monitor
                        );
                        self.lt_server_opts.lt_server_state = LTServerState::Stopped;
                    } else {
                        start_lt_server(
                            &mut self.lt_server_opts,
                            &mut self.lt_server,
                            &mut self.lt_device_monitor
                        );
                        self.lt_server_opts.lt_server_state = LTServerState::Running;
                    }
                }
            });

            ui.add(Separator::default());

            ui.add(Label::new(RichText::new("Settings").color(egui::Color32::WHITE).strong()));

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                Grid::new("settings_grid")
                    .num_columns(3)
                    .spacing([30.0, 0.0])
                    //.max_col_width(ui.available_width() * 0.5)
                    .show(ui, |ui| {
                        ui.label("Audio Input Mode");

                        // ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        if
                            ui
                                .add(egui::Checkbox::new(&mut self.lt_server_opts.input_mode, ""))
                                .changed()
                        {
                            self.restart_queued = true;
                        }
                        //});

                        ui.end_row();

                        ui.label("port");

                        let resp = ui.add(
                            egui::TextEdit
                                ::singleline(&mut self.port_string)
                                .hint_text(DEFAULT_PORT.to_string()) // do something on edited
                        );

                        if self.port_string.parse::<u16>().is_err() {
                            ui.add(
                                Label::new(RichText::new("Invalid port").color(egui::Color32::RED))
                            );
                            no_errors = false;
                        } else {
                            if resp.changed() {
                                self.restart_queued = true;
                            }
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

                        // ui.end_row();

                        ui.label("gain (dB)");

                        let response = ui.add(Slider::new(&mut self.lt_server_opts.gain, -50.0..=50.0).text(""));
                        if response.changed() {
                            self.restart_queued = true;
                        }

                        ui.end_row();
                    });
            });

            // Handle restart
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
                    stop_lt_server(
                        &mut self.lt_server_opts,
                        &mut self.lt_server,
                        &mut self.lt_device_monitor
                    );
                    start_lt_server(
                        &mut self.lt_server_opts,
                        &mut self.lt_server,
                        &mut self.lt_device_monitor
                    );
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

pub fn start_lt_server(
    lt_server_opts: &mut LTServerOpts,
    lt_server: &mut Option<LunaTechServer>,
    lt_device_monitor: &mut Option<DeviceMonitor>
) {
    if lt_server_opts.lt_server_state == LTServerState::Running {
        println!("{}", "Server is already running, please stop it first".bold().red());
        return;
    }

    let host = cpal::default_host();
    let device = (
        if lt_server_opts.input_mode {
            host.default_input_device()
        } else {
            host.default_output_device()
        }
    ).expect("Failed to get default device");

    *lt_server = Some(LunaTechServer::new(lt_server_opts.port));
    *lt_device_monitor = Some(
        DeviceMonitor::new(lt_server_opts.sample_rate, lt_server_opts.buffer_size, lt_server_opts.gain)
    );

    let (tx, rx) = crossbeam::channel::unbounded();

    if let Some(lt_server) = lt_server {
        lt_server.set_thread_receiver(rx);
    } else {
        panic!("{}", "Failed to initialize and start server".bold().red());
    }

    if let Some(device_monitor) = lt_device_monitor {
        device_monitor.set_thread_sender(tx);
        device_monitor
            .build_stream_from_device(&device)
            .unwrap_or_else(|_| { panic!("{}", "Failed to set device".bold().red().to_string()) });
    } else {
        panic!("{}", "Failed to initialize and start device monitor".bold().red());
    }

    lt_server_opts.lt_server_state = LTServerState::Running;
    println!(
        "{}{} {}",
        "Luna".red().bold(),
        "Tech".purple().bold(),
        "server is now running".bold()
    );
    if let Some(lt_server) = lt_server {
        lt_server.start_server();
    }
}

pub fn stop_lt_server(
    lt_server_opts: &mut LTServerOpts,
    lt_server: &mut Option<server::LunaTechServer>,
    device_monitor: &mut Option<device_monitor::DeviceMonitor>
) {
    if let Some(server) = lt_server {
        server.stop_server();
    }

    if lt_server_opts.lt_server_state == LTServerState::Stopped {
        println!("{}", "Server is already stopped, please start it first".bold().red());
        return;
    }

    if let Some(device_monitor) = device_monitor {
        device_monitor.stop_device_monitor();
    }

    lt_server_opts.lt_server_state = LTServerState::Stopped;
    println!(
        "{}{} {}",
        "Luna".red().bold(),
        "Tech".purple().bold(),
        "server is now stopped".bold()
    );
}
