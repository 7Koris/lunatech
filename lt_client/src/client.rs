use std::mem::{self, MaybeUninit};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::thread;
use lt_utilities::audio_features::AtomicAudioFeatures;
use rosc::OscPacket;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use lt_utilities::OscAddresses;

pub struct LunaTechClient {
    socket: Arc<Socket>,
    pub audio_features: Arc<AtomicAudioFeatures>
}

impl LunaTechClient {
    
    pub fn new(port: u16) -> Self {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect("Failed to create socket");
        let bindaddr: SockAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into();
        socket.set_broadcast(true).expect("Failed to set broadcast"); 
        socket.set_reuse_address(true).expect("Failed to set reuse address");
        socket.bind(&bindaddr).expect("Failed to bind socket");
        
        let client = Self { 
            socket: socket.into(), 
            audio_features: AtomicAudioFeatures::default().into()
        };

        client.start_client();
        client
    }

    fn start_client(&self) {
        let socket_clone = self.socket.clone();
        let audio_features = self.audio_features.clone();

        thread::spawn(move || {
            let mut buf = [MaybeUninit::<u8>::uninit(); rosc::decoder::MTU];
            loop {
                let result = socket_clone.recv(&mut buf);

                // Todo: Safe?
                let data = unsafe { mem::transmute::<[std::mem::MaybeUninit<u8>; rosc::decoder::MTU], [u8; 1536]>(buf) };
                
                if let Ok(size) = result {
                    match rosc::decoder::decode_udp(&data[..size]) {
                        Ok((_, packet)) => {
                            Self::handle_packet(packet, &audio_features);
                        }
                        Err(e) => {
                            println!("Got invalid packet: {}", e);
                        }
                    }
                }
            }
        });
    }

    fn handle_packet(packet: OscPacket, audio_features: &AtomicAudioFeatures) {
            match packet {
                OscPacket::Message(_msg) => {}
                OscPacket::Bundle(bundle) => {
                    //println!("OSC Bundle: {:?}", bundle);
                    // TODO: Use timetag
                    bundle.content.iter().for_each(|packet| {
                        if let OscPacket::Message(msg) = packet {
                            if let Some(val) = <rosc::OscType as Clone>::clone(&msg.args[0]).float() {
                                match msg.addr.as_str() {
                                    OscAddresses::BROAD_RMS => {
                                        audio_features.broad_range_peak_rms.set(val);
                                    }
                                    OscAddresses::LOW_RMS => {
                                        audio_features.low_range_rms.set(val);
                                    }
                                    OscAddresses::MID_RMS => {
                                        audio_features.mid_range_rms.set(val);
                                    }
                                    OscAddresses::HIGH_RMS => {
                                        audio_features.high_range_rms.set(val);
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





    

