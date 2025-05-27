use std::sync::atomic::Ordering;
use std::sync::Arc;

pub type OscAddress = &'static str;
use paste::paste;

#[macro_export]
macro_rules! features {
    (
        $struct_vis:vis struct $struct_name:ident {
            $($feature_vis:vis $feature_name:ident: $feature_type:ty),* $(,)?
        }
    ) => {
            $(
                paste! {
                    pub type [<$feature_name:camel>] = f32;  
                }
      
                // Creates an OSC address such as: pub const OSC_ADDR_RMS: OscAddress = "/lt/RMS";
                paste! {
                    pub const [<OSC_ADDR_$feature_name:upper>]: OscAddress = concat!("/lt/", stringify!($feature_name));
                }

                paste! {
                    pub struct [<$feature_name:camel Atomic>] {
                        value: std::sync::atomic::AtomicU32,
                    }

                    impl [<$feature_name:camel Atomic>] {
                        pub fn new(val: f32) -> Self {
                            Self {
                                value: std::sync::atomic::AtomicU32::new((val).to_bits()),
                            }
                        }

                        pub fn get(&self) -> f32 {
                            f32::from_bits(self.value.load(Ordering::SeqCst))
                        }

                        pub fn set(&self, val: f32) {
                            self.value.store(val.to_bits(), Ordering::SeqCst);
                        }
                    }
                }
            )*

            paste! {
                $struct_vis struct [<$struct_name:camel>] {
                    $(
                        pub $feature_name: f32,
                    )*
                }
            }

            paste! {
                $struct_vis struct [<Atomic $struct_name:camel>] {
                    $(
                        pub $feature_name: Arc<[<$feature_name:camel Atomic>]>,
                    )*
                }
            }

            paste! {
                impl Default for [<Atomic $struct_name:camel>] {
                    fn default() -> Self {
                        Self {
                            $(
                                $feature_name: Arc::new([<$feature_name:camel Atomic>]::new(0.0)),
                            )*
                        }
                    }
                }
            }


    };
}

features! {
    pub struct Features {
        rms: f32,
        bass: f32,
        mid: f32,
        treble: f32,
        zcr: f32,
        centroid: f32,
        flux: f32,
        rolloff: f32,
        tv: f32,
    }
}