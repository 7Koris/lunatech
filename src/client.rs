use crate::analyzer::AudioFeatures;
use crate::service;

use mdns_sd::{DaemonEvent, ServiceDaemon, ServiceEvent};
use rosc::OscPacket;

use core::panic;
use std::env;
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str::FromStr;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub struct Client {
    pub socket: Socket,
}

impl Client {
    
    pub fn new() -> Self {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        let bindaddr: SockAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 3000).into();
        socket.set_broadcast(true).expect("Failed to set broadcast"); 
        socket.set_reuse_address(true).expect("Failed to set reuse address");
        socket.bind(&bindaddr).expect("Failed to bind socket");

        Self { socket }
    }
    
    pub fn get_osc_features(&self) {
        println!("Waiting for OSC packet...");
        let mut buf = [0u8; rosc::decoder::MTU];
        
        let mut buf = unsafe { &mut *(&mut buf as *mut [u8] as *mut [MaybeUninit<u8>]) };
        match self.socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let buf = buf.iter().map(|x| unsafe { x.assume_init() }).collect::<Vec<_>>();
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                Self::handle_packet(packet);
                // TODO pass off audio features data?
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                // TODO return last audio features or skip?
            }
        }
    }

    fn handle_packet(packet: OscPacket) -> AudioFeatures {
        match packet {
            OscPacket::Message(msg) => {
                println!("OSC address: {}", msg.addr);
                println!("OSC arguments: {:?}", msg.args);
                AudioFeatures::default()
            }
            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);
                AudioFeatures::default()
            }
        }
    }
    
}




    

