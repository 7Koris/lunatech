use std::{ error::Error, sync::Arc };
use colored::Colorize;
use crossbeam::channel::Sender;
use cpal::{ traits::{ DeviceTrait, StreamTrait }, SampleRate, StreamConfig };
use rustfft::num_traits::Pow;

use crate::analyzer::Analyzer;

use lt_utilities::features;

#[derive(Default)]
pub struct DeviceMonitor {
    buffer_size: u32,
    linear_factor: f32,
    device_name: Option<String>,
    /// The data stream of the device
    stream: Option<cpal::Stream>,
    tx: Option<Arc<Sender<features::Features>>>,
    // TODO: Ensure Proper error passing
    error_msg: Option<String>,
}

impl DeviceMonitor {
    pub fn new(buffer_size: u32, gain: f32) -> Self {
        Self {
            buffer_size,
            linear_factor: 10.0_f32.pow(gain / 20.0),
            device_name: None,
            stream: None,
            tx: None,
            error_msg: None,
        }
    }

    pub fn set_thread_sender(&mut self, tx: crossbeam::channel::Sender<features::Features>) {
        self.tx = Some(tx.into());
    }

    pub fn start_device_monitor(&self) {
        if let (Some(stream), Some(device_name)) = (&self.stream, &self.device_name) {
            match stream.play() {
                Ok(_) => {
                    println!("Monitoring device: {}", device_name.bold().green());
                }
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
                }
                Err(e) => {
                    println!("Failed to stop device monitor: {}", e.to_string().bold().red());
                    panic!();
                }
            }
        } else {
            println!("{}", "Stream is not initialized".bold().red());
        }
    }

    pub fn build_stream_from_device(
        &mut self,
        device: &cpal::Device
    ) -> Result<(), cpal::DeviceNameError> {
        let device_name = match device.name() {
            Ok(name) => { name }
            Err(e) => {
                println!("Failed to get device name: {}", e.to_string().bold().red());
                return Err(e);
            }
        };

        self.stream = Some(self.get_built_stream(device ,self.buffer_size));
        self.device_name = Some(device_name);
        Ok(())
    }

    fn try_building_stream(
        &self,
        device: &cpal::Device,
        config: &StreamConfig
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        let mut analyzer = Analyzer::new(config.channels, config.sample_rate.0, self.linear_factor);

        let sender = match &self.tx {
            Some(sender) => sender,
            None => {
                return Err("Sender not initialized".into());
            }
        };

        let shared_sender = sender.clone();

        let data_callback = move |mut sample_data: &[f32], _: &cpal::InputCallbackInfo| {
            analyzer.feed_data(sample_data);
            let tx = shared_sender.clone();
            let _ = tx.send(features::Features {
                rms: analyzer.audio_features.rms.get(),
                bass: analyzer.audio_features.bass.get(),
                mid: analyzer.audio_features.mid.get(),
                treble: analyzer.audio_features.treble.get(),
                zcr: analyzer.audio_features.zcr.get(),
                centroid: analyzer.audio_features.centroid.get(),
                flux: analyzer.audio_features.flux.get(),
                rolloff: analyzer.audio_features.rolloff.get(),
                tv: analyzer.audio_features.tv.get(),
            });
        };

        let error_callback = move |e: cpal::StreamError| {
            println!("{}", e.to_string().bold().red());
        };

        match device.build_input_stream(config, data_callback, error_callback, None) {
            Ok(input_stream) => { Ok(input_stream) }
            Err(e) => { Err(Box::new(e)) }
        }
    }

    fn get_built_stream(
        &self,
        device: &cpal::Device,
        buffer_size: u32
    ) -> cpal::Stream {
        let config = &(StreamConfig {
            channels: match device.default_input_config() {
                Ok(config) => {
                    println!(
                        "Using default config {}",
                        config.channels().to_string().bold().green()
                    );
                    config.channels()
                }
                Err(e) => {
                    println!("{}", e.to_string().bold().red());
                    panic!();
                }
            },
            sample_rate: SampleRate(device.default_input_config().unwrap().sample_rate().0),
            buffer_size: cpal::BufferSize::Fixed(buffer_size),
        });

        match self.try_building_stream(device, config) {
            Ok(new_stream) => new_stream,
            Err(e) => {
                println!("{}", e.to_string().bold().red());
                println!(
                    "{}",
                    "Main config failed, attempting to use backup config".bold().yellow()
                );

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
