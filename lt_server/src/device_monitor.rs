use std::{error::Error, sync::{Arc, Mutex}};
use colored::Colorize;
use crossbeam::channel::Sender;
use cpal::{traits::{DeviceTrait, StreamTrait}, SampleRate, StreamConfig};

use crate::analyzer::{Analyzer, AudioFeatures};

type ArcMutex<T> = Arc<Mutex<T>>;

#[derive(Default)]
pub struct DeviceMonitor {
    /// The name of the device
    sample_rate: u32,
    buffer_size: u32,
    device_name: Option<String>, 
    /// The data stream of the device
    stream: Option<cpal::Stream>,
    tx: Option<ArcMutex<Sender<AudioFeatures>>>,
    // Struct containing audio features
    pub audio_features: Arc<Mutex<AudioFeatures>>,
}

impl DeviceMonitor {

    pub fn new(sample_rate: u32, buffer_size: u32) -> Self {
        Self {   
            sample_rate,
            buffer_size,
            device_name: None,
            stream: None,
            tx: None,
            audio_features: Arc::new(Mutex::new(AudioFeatures::default())),
        }
    }

    pub fn set_thread_sender(&mut self, tx: crossbeam::channel::Sender<AudioFeatures>) {
        self.tx = Some(Arc::new(Mutex::new(tx)));
    }

    pub fn start_device_monitor(&self) {
        if let (Some(stream), Some(device_name)) = (&self.stream, &self.device_name) {
            match stream.play() {
                Ok(_) => {
                    println!("Monitoring device: {}", device_name.bold().green());
                },
                Err(e) => { 
                    println!("{}", e.to_string().bold().red());
                    panic!();
                }
            }
        } else {
            println!("{}", "Stream is not initialized".bold().red());
        }
    }

    pub fn stop_device_monitor(&self) {
        if let (Some(stream), Some(device_name)) = (&self.stream, &self.device_name) {
            match stream.pause() {
                Ok(_) => {
                    println!("Stopped device: {}", device_name.bold().green());
                },
                Err(e) => { 
                    println!("Failed to stop device monitor: {}", e.to_string().bold().red());
                    panic!();
                }
            }
        } else {
            println!("{}", "Stream is not initialized".bold().red());
        }
    }

    pub fn build_stream_from_device(&mut self, device: &cpal::Device) -> Result<(), cpal::DeviceNameError> {
        let device_name = match device.name() {
            Ok(name) => {
                name
            },
            Err(e) => {
                println!("Failed to get device name: {}", e.to_string().bold().red());
                return Err(e);
            }
        };

        self.stream = Some(self.get_built_stream(device, self.sample_rate, self.buffer_size));
        self.device_name = Some(device_name);
        Ok(())
    }

    fn try_building_stream(&self, device: &cpal::Device, config: &StreamConfig) -> Result<cpal::Stream, Box<dyn Error>>  {
        let mut analyzer = Analyzer::new(config.channels, config.sample_rate.0);
        
        let sender = match &self.tx {
            Some(sender) => sender,
            None => return Err("Sender not initialized".into()),
        };
        
        let shared_sender = Arc::clone(sender);

        let data_callback = move |sample_data: &[f32], _: &cpal::InputCallbackInfo| {
            analyzer.feed_data(sample_data);
            let tx = shared_sender.clone();
            let tx_lock = tx.lock();
            if let Ok(tx) = tx_lock {
                let _ = tx.send(analyzer.features.clone());
            }
        };

        let error_callback = move |e: cpal::StreamError| {
            println!("{}", e.to_string().bold().red());
        };

        match device.build_input_stream(config, data_callback, error_callback, None) {
            Ok(input_stream) => {
                Ok(input_stream)
            },
            Err(e) => {
                Err(Box::new(e))
            }
        }
    }

    fn get_built_stream(&self, device: &cpal::Device, sample_rate: u32, buffer_size: u32) -> cpal::Stream {
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

        

        match self.try_building_stream(device, config) {
            Ok(new_stream) => new_stream,
            Err(e) => {
                println!("{}", e.to_string().bold().red());
                println!("{}", "Main config failed, attempting to use backup config".bold().yellow());

                let config = match device.default_input_config() {
                    Ok(config) => config,
                    Err(e) => {
                        println!("{}", e.to_string().bold().red());
                        panic!();
                    }
                };

                self.try_building_stream(device, &config.config()).unwrap_or_else(|e| {
                    println!("Failed to build input stream: {}", e.to_string().bold().red());
                    panic!();
                })
            }
        }
    }
}





