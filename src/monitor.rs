use std::thread;

use colored::{ColoredString, Colorize};
use cpal::{traits::{DeviceTrait, StreamTrait}, BuildStreamError, SampleRate, StreamConfig};

pub struct DeviceMonitor {
    device_name: ColoredString,
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
        let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {   
            println!("{:?}", data);
        };
    
        let error_callback = move |e: cpal::StreamError| {
            println!("{}", e.to_string().bold().red());
        };
        
        device.build_input_stream(config, data_callback, error_callback, None)
    }

    fn get_built_stream(device: &cpal::Device, sample_rate: u32, buffer_size: u32) -> cpal::Stream {
        let config = &StreamConfig {
            channels: 1_u16, // device.default_output_config().unwrap().channels(), TODO: ???
            sample_rate: SampleRate(sample_rate), 
            buffer_size: cpal::BufferSize::Fixed(buffer_size),
        };

        let stream =  Self::try_building_stream(device, config);

       let stream = if stream.is_ok() {
            stream
        } else {
            println!("{}", stream.err().unwrap().to_string().bold().red());
            println!("{}", "Attempting to use backup config".bold().yellow());

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
                panic!();
            }
        }
    }
}





