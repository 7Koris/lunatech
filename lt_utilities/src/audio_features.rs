use std::sync::{atomic::{Ordering}, Arc};

use crate::atomic_float;

atomic_float!(BroadRangeRMS);
atomic_float!(LowRangeRMS);
atomic_float!(MidRangeRMS);
atomic_float!(HighRangeRMS);

pub type Features = (BroadRangeRMS, LowRangeRMS, MidRangeRMS, HighRangeRMS);

pub struct AtomicAudioFeatures {
    pub broad_range_peak_rms: Arc<BroadRangeRMSAtomic>,
    pub low_range_rms: Arc<LowRangeRMSAtomic>,
    pub mid_range_rms: Arc<MidRangeRMSAtomic>,
    pub high_range_rms: Arc<HighRangeRMSAtomic>,
}

impl Default for AtomicAudioFeatures {
    fn default() -> Self {
        Self {
            broad_range_peak_rms: Arc::new(BroadRangeRMSAtomic::new(0.0)),
            low_range_rms: Arc::new(LowRangeRMSAtomic::new(0.0)),
            mid_range_rms: Arc::new(MidRangeRMSAtomic::new(0.0)),
            high_range_rms: Arc::new(HighRangeRMSAtomic::new(0.0)),
        }
    }
}