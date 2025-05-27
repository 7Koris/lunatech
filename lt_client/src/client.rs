use std::mem::{ self, MaybeUninit };
use std::net::{ Ipv4Addr, SocketAddrV4 };
use std::sync::Arc;
use std::thread;
use lt_utilities::audio_features::{
    AudioFeatures,
    OSC_ADDR_BASS,
    OSC_ADDR_FLUX,
    OSC_ADDR_MID,
    OSC_ADDR_RMS,
    OSC_ADDR_CENTROID,
    OSC_ADDR_TREBLE,
    OSC_ADDR_ZCR,
    OSC_ADDR_ROLLOFF,
    OSC_ADDR_TV,
};
use rosc::OscPacket;
use socket2::{ Domain, Protocol, SockAddr, Socket, Type };

pub struct LunaTechClient {
    socket: Arc<Socket>,
    pub audio_features: Arc<AudioFeatures>,
}

impl LunaTechClient {
    pub fn new(port: u16) -> Self {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).expect(
            "Failed to create socket"
        );
        let bindaddr: SockAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into();
        socket.set_broadcast(true).expect("Failed to set broadcast");
        socket.set_reuse_address(true).expect("Failed to set reuse address");
        socket.bind(&bindaddr).expect("Failed to bind socket");

        let client = Self {
            socket: socket.into(),
            audio_features: AudioFeatures::default().into(),
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
                let data = unsafe {
                    mem::transmute::<[std::mem::MaybeUninit<u8>; rosc::decoder::MTU], [u8; 1536]>(
                        buf
                    )
                };

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

    fn handle_packet(packet: OscPacket, audio_features: &AudioFeatures) {
        match packet {
            OscPacket::Message(_msg) => {}
            OscPacket::Bundle(bundle) => {
                //println!("OSC Bundle: {:?}", bundle);
                // TODO: Use timetag
                bundle.content.iter().for_each(|packet| {
                    if let OscPacket::Message(msg) = packet {
                        if let Some(val) = <rosc::OscType as Clone>::clone(&msg.args[0]).float() {
                            match msg.addr.as_str() {
                                OSC_ADDR_RMS => {
                                    audio_features.rms.set(val);
                                }
                                OSC_ADDR_BASS => {
                                    audio_features.bass.set(val);
                                }
                                OSC_ADDR_MID => {
                                    audio_features.mid.set(val);
                                }
                                OSC_ADDR_TREBLE => {
                                    audio_features.treble.set(val);
                                }
                                OSC_ADDR_ZCR => {
                                    audio_features.zcr.set(val);
                                }
                                OSC_ADDR_CENTROID => {
                                    audio_features.centroid.set(val);
                                }
                                OSC_ADDR_FLUX => {
                                    audio_features.flux.set(val);
                                }
                                OSC_ADDR_ROLLOFF => {
                                    audio_features.rolloff.set(val);
                                }
                                OSC_ADDR_TV => {
                                    audio_features.tv.set(val);
                                }
                                _ => {}
                            }
                        }
                    }
                });
            }
        }
    }
}
