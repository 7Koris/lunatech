use std::{net::{ Ipv4Addr, SocketAddr}, sync::{Arc, Mutex}, thread, time::{Duration, Instant, UNIX_EPOCH}};
use colored::Colorize;
use rosc::{encoder, OscBundle, OscError, OscMessage, OscPacket, OscTime, OscType};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use crossbeam::channel::Receiver;

use lt_utilities::audio_features::{Features, OSC_ADDR_BROADRANGERMS, OSC_ADDR_FLUX, OSC_ADDR_HIGHRANGERMS, OSC_ADDR_LOWRANGERMS, OSC_ADDR_MIDRANGERMS, OSC_ADDR_SPECTRALCENTROID, OSC_ADDR_ZCR};

pub struct LunaTechServer {
    socket: Arc<Socket>,
    addrs: SockAddr,
    rx: Option<Arc<Receiver<Features>>>,
    thread_terminator: Arc<Mutex<bool>>,
}

impl LunaTechServer {
    pub fn new(port: u16) -> Self {
        let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        socket.set_broadcast(true).expect("Failed to set broadcast");
        socket.set_reuse_address(true).expect("Failed to set reuse address");

        let addrs: SocketAddr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), port);
        let addrs: SockAddr = SockAddr::from(addrs);
        
        Self { socket: socket.into(), addrs, rx: None, thread_terminator: Arc::new(false.into())}
    }

    // Currently unused
    pub fn start_heartbeat_thread(&self) {
        let socket = self.socket.clone();
        let addrs = self.addrs.clone();
        thread::spawn(move || {
            loop {
                let current_time_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                let data = format!("{{\"host\":\"lt\",\"time\":{}}}", current_time_ms); 
                let utf8_buffer = data.as_bytes();
                let _ = socket.send_to(&utf8_buffer, &addrs);
                thread::sleep(Duration::from_millis(60));
            }
        });
    }

    pub fn set_thread_receiver(&mut self, rx: Receiver<Features>) {
        self.rx = Some(rx.into());
    }

    pub fn stop_server(&mut self) {
        *self.thread_terminator.lock().unwrap() = true;
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

        // self.start_heartbeat_thread();
        let thread_terminator_ref = self.thread_terminator.clone();
        let rx = receiver.clone(); 
        let time = Instant::now();
        thread::spawn(move || {
            let start_time: std::time::Instant = std::time::Instant::now();
            let epoch_start: Duration = UNIX_EPOCH.elapsed().unwrap_or(Duration::from_secs(0));
            let mut audio_features: Result<Features, crossbeam::channel::RecvError>;
            loop {
                // TODO: handle unwraps better?
                if *thread_terminator_ref.lock().unwrap() {
                    break;
                }

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
    let (broad_range_rms, 
        low_range_rms, 
        mid_range_rms, 
        high_range_rms, 
        zcr, 
        spectral_centroid, 
        flux) = features;

    encoder::encode(&OscPacket::Bundle(OscBundle {
        timetag: {
            OscTime::from((secs, frac))
        },
        content: vec![
            OscPacket::Message(OscMessage {
                addr: OSC_ADDR_BROADRANGERMS.to_string(),
                args: vec![
                    OscType::Float(broad_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OSC_ADDR_LOWRANGERMS.to_string(),
                args: vec![
                    OscType::Float(low_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OSC_ADDR_MIDRANGERMS.to_string(),
                args: vec![
                    OscType::Float(mid_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OSC_ADDR_HIGHRANGERMS.to_string(),
                args: vec![
                    OscType::Float(high_range_rms),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OSC_ADDR_ZCR.to_string(),
                args: vec![
                    OscType::Float(zcr),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OSC_ADDR_SPECTRALCENTROID.to_string(),
                args: vec![
                    OscType::Float(spectral_centroid),
                ],
            }),
            OscPacket::Message(OscMessage {
                addr: OSC_ADDR_FLUX.to_string(),
                args: vec![
                    OscType::Float(flux),
                ],
            }),
        ],
    }))
}