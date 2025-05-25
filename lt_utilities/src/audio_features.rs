use std::sync::{ atomic::{ Ordering }, Arc };

use crate::atomic_float;

atomic_float!(RMS);
atomic_float!(LowRangeRMS);
atomic_float!(MidRangeRMS);
atomic_float!(HighRangeRMS);
atomic_float!(ZCR);
atomic_float!(SpectralCentroid);
atomic_float!(Flux);

pub type Features = (RMS, LowRangeRMS, MidRangeRMS, HighRangeRMS, ZCR, SpectralCentroid, Flux);

pub struct AtomicAudioFeatures {
    pub rms: Arc<RMSAtomic>,
    pub low_range_rms: Arc<LowRangeRMSAtomic>,
    pub mid_range_rms: Arc<MidRangeRMSAtomic>,
    pub high_range_rms: Arc<HighRangeRMSAtomic>,
    pub zcr: Arc<ZCRAtomic>,
    pub spectral_centroid: Arc<SpectralCentroidAtomic>,
    pub flux: Arc<FluxAtomic>,
}

impl Default for AtomicAudioFeatures {
    fn default() -> Self {
        Self {
            rms: Arc::new(RMSAtomic::new(0.0)),
            low_range_rms: Arc::new(LowRangeRMSAtomic::new(0.0)),
            mid_range_rms: Arc::new(MidRangeRMSAtomic::new(0.0)),
            high_range_rms: Arc::new(HighRangeRMSAtomic::new(0.0)),
            zcr: Arc::new(ZCRAtomic::new(0.0)),
            spectral_centroid: Arc::new(SpectralCentroidAtomic::new(0.0)),
            flux: Arc::new(FluxAtomic::new(0.0)),
        }
    }
}
