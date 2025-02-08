use crate::analyzer::AudioFeatures;

use colored::Colorize;
use rosc::{encoder, OscBundle, OscMessage, OscPacket, OscTime, OscType};
use std::{fmt::format, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}, os::fd::AsFd, result, str::FromStr, time::{Duration, UNIX_EPOCH}};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub struct Service {
    socket: Socket,
    addrs: SockAddr,
    start_time: std::time::Instant,
    epoch_start: Duration
}


impl Service {
    pub fn new() -> Self {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        let addrs: SockAddr = SocketAddrV4::new(Ipv4Addr::BROADCAST, 3000).into(); 
        socket.set_broadcast(true).expect("Failed to set broadcast");
        socket.set_reuse_address(true).expect("Failed to set reuse address");

        let start_time = std::time::Instant::now();
        let epoch_start = UNIX_EPOCH.elapsed().unwrap_or(Duration::from_secs(0));

        Self { socket, addrs, start_time, epoch_start }
    }
    
    pub fn send_osc_features(&self, features: &AudioFeatures) {
        let msg_buf = encoder::encode(&OscPacket::Bundle(OscBundle {
            timetag: {
                let secs = (self.epoch_start.as_secs() + self.start_time.elapsed().as_secs()) as u32;
                let frac = (self.start_time.elapsed().subsec_micros());
                OscTime::from((secs, frac))
            },
            content: vec![
                OscPacket::Message(OscMessage {
                    addr: "/lt/broad_rms".to_string(),
                    args: vec![
                        OscType::Float(features.broad_range_peak_rms),
                        OscType::Float(features.low_range_rms),
                        OscType::Float(features.mid_range_rms),
                        OscType::Float(features.high_range_rms),
                    ],
                }),
            ],
        }));
        
        match msg_buf {
            Ok(msg_buf) => {
                let res = self.socket.send_to(&msg_buf, &self.addrs);
                if res.is_err() {
                    println!("Error broadcasting: {}", res.err().unwrap().to_string().bold().red());
                }
            },
            Err(e) => {
                println!("Error serializing audio features: {}", e.to_string().bold().red());
            }
        }
    }
}