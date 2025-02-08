use std::{net::UdpSocket, thread, time::Instant};
use colored::{ColoredString, Colorize};
use cpal::{traits::{DeviceTrait, StreamTrait}, BuildStreamError, SampleRate, StreamConfig};

use crate::analyzer::Analyzer;
use crate::service;

pub struct DeviceMonitor {
    /// The name of the device
    device_name: ColoredString, 
    /// The data stream of the device
    stream: cpal::Stream
}

impl DeviceMonitor {

    pub fn new(device: &cpal::Device, sample_rate: u32, buffer_size: u32) -> Self {
        let stream = Self::get_built_stream(device, sample_rate, buffer_size);

        let device_name = match device.name() {
            Ok(name) => {
                name.bold().green()
            },
            Err(_) => {
                "Unknown Device".to_string().bold().red()
            }
        };

        Self {   
            device_name,
            stream
        }
    }

    pub fn monitor_device(&self) {
        match self.stream.play() {
            Ok(_) => {
                println!("Monitoring device: {}", self.device_name);
                thread::park();
            },
            Err(e) => { 
                println!("{}", e.to_string().bold().red());
                panic!();
            }
        }
    }

    fn try_building_stream(device: &cpal::Device, config: &StreamConfig) -> Result<cpal::Stream, BuildStreamError>  {
        let mut analyzer = Analyzer::new(config.channels, config.sample_rate.0);
        let service = service::Service::new();

        let data_callback = move |sample_data: &[f32], _: &cpal::InputCallbackInfo| {   
            analyzer.feed_data(sample_data);
            service.send_osc_features(&analyzer.features);
        };
    
        let error_callback = move |e: cpal::StreamError| {
            println!("{}", e.to_string().bold().red());
        };
        
        device.build_input_stream(config, data_callback, error_callback, None)
    }

    fn get_built_stream(device: &cpal::Device, sample_rate: u32, buffer_size: u32) -> cpal::Stream {
        let config = &StreamConfig {
            channels: 
                match  device.default_input_config() {
                    Ok(config) => {
                        println!("Using default config {}", config.channels().to_string().bold().green());
                        config.channels()
                    },
                    Err(e) => {
                        println!("{}", e.to_string().bold().red());
                        panic!();
                    }
                },
            sample_rate: SampleRate(sample_rate), 
            buffer_size: cpal::BufferSize::Fixed(buffer_size),
        };

        let stream =  Self::try_building_stream(device, config);

        

        let stream = if stream.is_ok() {
            stream
        } else {
            println!("{}", stream.err().unwrap().to_string().bold().red());
            println!("{}", "Main config failed, attempting to use backup config".bold().yellow());

            let config = device.default_input_config();
            let config = match config {
                Ok(config) => config,
                Err(e) => {
                    println!("{}", e.to_string().bold().red());
                    panic!();
                }
            };

            Self::try_building_stream(device, &config.config())
        };

        match stream {
            Ok(stream) => stream,
            Err(e) => {
                println!("{}", e.to_string().bold().red());
                println!("{}", "Main config failed, attempting to use backup config".bold().yellow());

                let config = device.default_input_config();
                let config = match config {
                    Ok(config) => config,
                    Err(e) => {
                        println!("{}", e.to_string().bold().red());
                        panic!();
                    }
                };

                let stream = Self::try_building_stream(device, &config.config());
                if stream.is_err() {
                    println!("{}", e.to_string().bold().red());
                    panic!();
                } else {
                    stream.unwrap()
                }
            }
        }
    }
}





