use std::{net::{ Ipv4Addr, SocketAddr}, sync::Arc, thread, time::{Duration, UNIX_EPOCH}};
use colored::Colorize;
use rosc::{encoder, OscBundle, OscError, OscMessage, OscPacket, OscTime, OscType};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use crossbeam::channel::Receiver;

use lt_utilities::{audio_features::Features, OscAddresses};


pub struct LunaTechServer {
    socket: Arc<Socket>,
    addrs: SockAddr,
    rx: Option<Arc<Receiver<Features>>>,
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

    pub fn set_thread_receiver(&mut self, rx: Receiver<Features>) {
        self.rx = Some(rx.into());
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
        
        let rx = receiver.clone(); 
        thread::spawn(move || {
            let start_time: std::time::Instant = std::time::Instant::now();
            let epoch_start: Duration = UNIX_EPOCH.elapsed().unwrap_or(Duration::from_secs(0));
            let mut audio_features: Result<Features, crossbeam::channel::RecvError>;
            loop {
                audio_features = rx.recv();
                if let Ok(features) = audio_features {
                    let secs = (epoch_start.as_secs() + start_time.elapsed().as_secs()) as u32;
                    let frac = start_time.elapsed().subsec_micros();
                    if let Ok(buf) = features_to_osc(features, secs, frac) {
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

fn features_to_osc(features: Features, secs: u32, frac: u32) -> Result<Vec<u8>, OscError> {
    let (broad_range_peak_rms, low_range_rms, mid_range_rms, high_range_rms) = features;

    encoder::encode(&OscPacket::Bundle(OscBundle {
        timetag: {
            OscTime::from((secs, frac))
        },
        content: vec![
            OscPacket::Message(OscMessage {
                addr: OscAddresses::BROAD_RMS.to_string(),
                args: vec![
                    OscType::Float(broad_range_peak_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OscAddresses::LOW_RMS.to_string(),
                args: vec![
                    OscType::Float(low_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OscAddresses::MID_RMS.to_string(),
                args: vec![
                    OscType::Float(mid_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OscAddresses::HIGH_RMS.to_string(),
                args: vec![
                    OscType::Float(high_range_rms),
                ],
            }),
        ],
    }))
}