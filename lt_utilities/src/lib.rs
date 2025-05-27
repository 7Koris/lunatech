use std::sync::{ Arc, Mutex };

pub mod feature;
pub mod audio_features;

pub type ArcMutex<T> = Arc<Mutex<T>>;
#[macro_export]
macro_rules! ArcMutex {
    ($val:expr) => {
        std::sync::Arc::new(std::sync::Mutex::new($val))
    };
}

// pub struct OscAddresses {}
// impl OscAddresses {
//     pub const BROAD_RMS: OscAddress = "/lt/broad_rms";
//     pub const LOW_RMS: OscAddress = "/lt/low_rms";
//     pub const MID_RMS: OscAddress = "/lt/mid_rms";
//     pub const HIGH_RMS: OscAddress = "/lt/high_rms";
//     pub const ZCR: OscAddress = "/lt/zcr";
// }
