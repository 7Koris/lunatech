use colored::Colorize;
use rosc::{encoder, OscBundle, OscMessage, OscPacket, OscTime, OscType};
use std::{net::{ Ipv4Addr, SocketAddr}, sync::Arc, time::{Duration, UNIX_EPOCH}};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{net::UdpSocket, sync::{mpsc, Mutex}};

use crate::analyzer::AudioFeatures;

pub struct LunaTechServer {
    socket: Arc<UdpSocket>,
    addrs: SocketAddr,
    rx: Arc<Mutex<Option<mpsc::Receiver<AudioFeatures>>>>,
}

impl LunaTechServer {
    pub fn new(port: u16) -> Self {
        let socket: Socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        socket.set_broadcast(true).expect("Failed to set broadcast");
        socket.set_reuse_address(true).expect("Failed to set reuse address");
        let socket: UdpSocket = UdpSocket::from_std(socket.into()).expect("Failed to create UDP socket");
        let addrs: SocketAddr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), port);
        
        Self { socket: socket.into(), addrs, rx: Arc::new(Mutex::new(None))}
    }

    pub fn set_thread_receiver(&mut self, rx: mpsc::Receiver<AudioFeatures>) {
        self.rx = Arc::new(Mutex::new(Some(rx)));
    }

    pub async fn start_server(&self) {
        let mut rx = self.rx.lock().await.take();
        let addrs = self.addrs;
        let socket = self.socket.clone();

        tokio::spawn(async move {
            let start_time: std::time::Instant = std::time::Instant::now();
            let epoch_start: Duration = UNIX_EPOCH.elapsed().unwrap_or(Duration::from_secs(0));
        
            if let Some(rx) = &mut rx {
                while let Some(features) = rx.recv().await {
                    let msg_buf = encoder::encode(&OscPacket::Bundle(OscBundle {
                        timetag: {
                            let secs = (epoch_start.as_secs() + start_time.elapsed().as_secs()) as u32;
                            let frac = start_time.elapsed().subsec_micros();
                            OscTime::from((secs, frac))
                        },
                        content: vec![
                            OscPacket::Message(OscMessage {
                                addr: "/lt/broad_rms".to_string(),
                                args: vec![
                                    OscType::Float(features.broad_range_peak_rms),
                                ],
                            }),
                            OscPacket::Message(OscMessage {
                                addr: "/lt/low_rms".to_string(),
                                args: vec![
                                    OscType::Float(features.low_range_rms),
                                ],
                            }),
                            OscPacket::Message(OscMessage {
                                addr: "/lt/mid_rms".to_string(),
                                args: vec![
                                    OscType::Float(features.mid_range_rms),
                                ],
                            }),
                            OscPacket::Message(OscMessage {
                                addr: "/lt/high_rms".to_string(),
                                args: vec![
                                    OscType::Float(features.high_range_rms),
                                ],
                            }),
                        ],
                    }));

                    match msg_buf {
                        Ok(msg_buf) => {
                            if let Err(e) = socket.send_to(&msg_buf, &addrs).await {
                                println!("Error sending audio features: {}", e.to_string().bold().red());
                            }
                        },
                        Err(e) => {
                            println!("Error serializing audio features: {}", e.to_string().bold().red());
                        }
                    }
                }
            } else {
                println!("{}", "Cannot start server without data input channel".bold().red());
            }
        });
    }
}