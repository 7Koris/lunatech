use std::{net::{ Ipv4Addr, SocketAddr}, sync::{Arc, Mutex}, thread, time::{Duration, UNIX_EPOCH}};
use colored::Colorize;
use rosc::{encoder, OscBundle, OscError, OscMessage, OscPacket, OscTime, OscType};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use crossbeam::channel::Receiver;

use crate::analyzer::AudioFeatures;

type ArcMutex<T> = Arc<Mutex<T>>;

pub struct LunaTechServer {
    socket: Arc<Socket>,
    addrs: SockAddr,
    /// Receiver channel for audio features
    rx: Option<Arc<Mutex<Receiver<AudioFeatures>>>>,
}

impl LunaTechServer {
    pub fn new(port: u16) -> Self {
        let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        socket.set_broadcast(true).expect("Failed to set broadcast");
        socket.set_reuse_address(true).expect("Failed to set reuse address");

        let addrs: SocketAddr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), port);
        let addrs: SockAddr = SockAddr::from(addrs);
        
        Self { socket: socket.into(), addrs, rx: None}
    }

    pub fn set_thread_receiver(&mut self, rx: Receiver<AudioFeatures>) {
        self.rx = Some(ArcMutex::new(rx.into()));
    }

    pub fn start_server(&self) {
        if self.rx.is_none() {
            panic!("Cannot start server without data input channel");
        }

        let addrs = self.addrs.clone();
        let socket = self.socket.clone();

        let receiver = match &self.rx {
            Some(receiver) => receiver, 
            None => panic!("Cannot start server without data input channel"),
        };
        
        let shared_sender: ArcMutex<Receiver<AudioFeatures>> = Arc::clone(receiver);
        

        thread::spawn(move || {
            let start_time: std::time::Instant = std::time::Instant::now();
            let epoch_start: Duration = UNIX_EPOCH.elapsed().unwrap_or(Duration::from_secs(0));
            let rx = shared_sender.clone();
            let rx = rx.lock().unwrap(); // TODO
            let mut audio_features: Result<AudioFeatures, crossbeam::channel::RecvError>;
            loop {
                audio_features = rx.recv();
                if let Ok(features) = audio_features {
                    let secs = (epoch_start.as_secs() + start_time.elapsed().as_secs()) as u32;
                    let frac = start_time.elapsed().subsec_micros();
                    if let Ok(buf) = features_to_osc(&features, secs, frac) {
                        let result = socket.send_to(&buf, &addrs);
                        match result {
                            Ok(_) => {},
                            Err(e) => {
                                println!("Error sending audio features: {}", e.to_string().bold().red());
                            }
                        }
                    }
                }
         
        }});  
        }
    }


type OscAddress = &'static str;
pub struct OscAddresses {}
impl OscAddresses {
    pub const BROAD_RMS: OscAddress = "/lt/broad_rms";
    pub const LOW_RMS: OscAddress = "/lt/low_rms";
    pub const MID_RMS: OscAddress = "/lt/mid_rms";
    pub const HIGH_RMS: OscAddress = "/lt/high_rms";
}


fn features_to_osc(features: &AudioFeatures, secs: u32, frac: u32) -> Result<Vec<u8>, OscError> {
    encoder::encode(&OscPacket::Bundle(OscBundle {
        timetag: {
            OscTime::from((secs, frac))
        },
        content: vec![
            OscPacket::Message(OscMessage {
                addr: OscAddresses::BROAD_RMS.to_string(),
                args: vec![
                    OscType::Float(features.broad_range_peak_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OscAddresses::LOW_RMS.to_string(),
                args: vec![
                    OscType::Float(features.low_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OscAddresses::MID_RMS.to_string(),
                args: vec![
                    OscType::Float(features.mid_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OscAddresses::HIGH_RMS.to_string(),
                args: vec![
                    OscType::Float(features.high_range_rms),
                ],
            }),
        ],
    }))
}