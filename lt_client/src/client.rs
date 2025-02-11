use lt_server::analyzer::AudioFeatures;

use rosc::OscPacket;
use core::task;
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{net::UdpSocket, sync::{mpsc, Mutex}};

pub struct LunaTechClient {
    pub socket: Arc<UdpSocket>,
    pub audio_features: AudioFeatures,
    tk_runtime: tokio::runtime::Runtime
}

impl LunaTechClient {
    
    pub fn new(port: u16) -> Self {
        let tk_runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let _guard = tk_runtime.enter();
        
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        let bindaddr: SockAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into();
        socket.set_broadcast(true).expect("Failed to set broadcast"); 
        socket.set_reuse_address(true).expect("Failed to set reuse address");
        socket.bind(&bindaddr).expect("Failed to bind socket");
        let socket: UdpSocket = UdpSocket::from_std(socket.into()).expect("Failed to create UDP socket");
        //let addrs: SocketAddr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), port);
        
        
        let client = Self { 
            socket: socket.into(), 
            audio_features: AudioFeatures::default(),
            tk_runtime
        };

        client
    }

    pub fn start_client(&self) {
        let socket_clone = self.socket.clone();
        self.tk_runtime.spawn(async move {
            loop {
                let mut buf: [u8; 1536] = [0u8; rosc::decoder::MTU];

                match socket_clone.recv(buf.as_mut()).await {
                    Ok(size) => {
                        let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                        // //Self::handle_packet(packet);
                        println!("Received packet: {:?}", packet);
                        
                    }
                    Err(e) => {
                        println!("Error receiving from socket: {}", e);
                        // TODO return last audio features or skip?
                    }
                }
           }
        });

    }

    // fn handle_packet(packet: OscPacket) {
    //     match packet {
    //         OscPacket::Message(msg) => {
    //             println!("OSC address: {}", msg.addr);
    //             println!("OSC arguments: {:?}", msg.args);
    //             AudioFeatures::default()
    //         }
    //         OscPacket::Bundle(bundle) => {
    //             println!("OSC Bundle: {:?}", bundle);
    //             for packet in bundle.content {
    //                 Self::handle_packet(packet);
    //             }
    //         }
    //     }
    // }
    
}




    

