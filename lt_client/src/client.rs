use std::mem::{self, MaybeUninit};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use rosc::OscPacket;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use lt_server::analyzer::AudioFeatures;
use lt_server::server::OscAddresses;

type ArcMutex<T> = Arc<Mutex<T>>;

pub struct LunaTechClient {
    pub socket: Arc<Socket>,
    pub audio_features: ArcMutex<AudioFeatures>
}


impl LunaTechClient {
    
    pub fn new(port: u16) -> Self {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        let bindaddr: SockAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into();
        socket.set_broadcast(true).expect("Failed to set broadcast"); 
        socket.set_reuse_address(true).expect("Failed to set reuse address");
        socket.bind(&bindaddr).expect("Failed to bind socket");
        
        Self { 
            socket: socket.into(), 
            audio_features: Arc::new(Mutex::new(AudioFeatures::default()))
        }
    }

    pub fn get_audio_features(&self) -> Result<AudioFeatures, PoisonError<MutexGuard<'_, AudioFeatures>>> {
        let result = self.audio_features.lock();

        match result {
            Ok(features) => Ok(features.clone()),
            Err(e) => Err(e)
        }
    }

    pub fn start_client(&self) {
        let socket_clone = self.socket.clone();
        let audio_features_clone = Arc::clone(&self.audio_features);

        thread::spawn(move || {
            let mut buf = [MaybeUninit::<u8>::uninit(); rosc::decoder::MTU];
            loop {
                let result = socket_clone.recv(&mut buf);

                let data = unsafe { mem::transmute::<[std::mem::MaybeUninit<u8>; 1536], [u8; 1536]>(buf) };
                
                if let Ok(size) = result {
                    match rosc::decoder::decode_udp(&data[..size]) {
                        Ok((_, packet)) => {
                            Self::handle_packet(packet, &audio_features_clone);
                        }
                        Err(e) => {
                            println!("Got invalid packet: {}", e);
                        }
                    }
                }
            }
        });
    }

    fn handle_packet(packet: OscPacket, audio_features: &Mutex<AudioFeatures>) {
        let audio_features = audio_features.lock();
        if let Ok(mut audio_features) = audio_features {
            match packet {
                OscPacket::Message(_msg) => {}
                OscPacket::Bundle(bundle) => {
                    //println!("OSC Bundle: {:?}", bundle);
                    // TODO: Use timetag
                    bundle.content.iter().for_each(|packet| {
                        if let OscPacket::Message(msg) = packet {
                            if let Some(f) = <rosc::OscType as Clone>::clone(&msg.args[0]).float() {
                                match msg.addr.as_str() {
                                    OscAddresses::BROAD_RMS => {
                                        audio_features.broad_range_peak_rms = f;
                                    }
                                    OscAddresses::LOW_RMS => {
                                        audio_features.low_range_rms = f;
                                    }
                                    OscAddresses::MID_RMS => {
                                        audio_features.mid_range_rms = f;
                                    }
                                    OscAddresses::HIGH_RMS => {
                                        audio_features.high_range_rms = f;
                                    }
                                    _ => {}
                                }
                            }
                        };
                    });
                }
            }
        }
    }
}




    

