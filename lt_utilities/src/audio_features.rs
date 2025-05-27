use std::sync::{ atomic::{ Ordering }, Arc };

use crate::feature;

// if you add a feature, you need to update the following files:
// - server.rs (handle OSCADDR)
// - client.rs (handle OSCADDR)
// - analyzer.rs (send new bindings)
// - device_monitor.rs (push new bindings to server)
// not the best system but works for now

feature!(RMS);
feature!(Bass);
feature!(Mid);
feature!(Treble);
feature!(ZCR);
feature!(Centroid);
feature!(Flux);
feature!(RollOff);
feature!(TV);

pub type Features = (RMS, Bass, Mid, Treble, ZCR, Centroid, Flux, RollOff, TV);

pub struct AudioFeatures {
    pub rms: Arc<RMSAtomic>,
    pub bass: Arc<BassAtomic>,
    pub mid: Arc<MidAtomic>,
    pub treble: Arc<TrebleAtomic>,
    pub zcr: Arc<ZCRAtomic>,
    pub centroid: Arc<CentroidAtomic>,
    pub flux: Arc<FluxAtomic>,
    pub rolloff: Arc<RollOffAtomic>,
    pub tv: Arc<TVAtomic>,
}

impl Default for AudioFeatures {
    fn default() -> Self {
        Self {
            rms: Arc::new(RMSAtomic::new(0.0)),
            bass: Arc::new(BassAtomic::new(0.0)),
            mid: Arc::new(MidAtomic::new(0.0)),
            treble: Arc::new(TrebleAtomic::new(0.0)),
            zcr: Arc::new(ZCRAtomic::new(0.0)),
            centroid: Arc::new(CentroidAtomic::new(0.0)),
            flux: Arc::new(FluxAtomic::new(0.0)),
            rolloff: Arc::new(RollOffAtomic::new(0.0)),
            tv: Arc::new(TVAtomic::new(0.0)),
        }
    }
}
