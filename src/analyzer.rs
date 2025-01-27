use std::ops::Range;

const LOW_RANGE: Range<f32> = 0.0..250.0; // Hz
const MID_RANGE: Range<f32> = 250.0..4000.0; // Hz
const HIGH_RANGE: Range<f32> = 4000.0..20000.0; // Hz 

pub struct Analyzer {
    // fft_planner: FftPlanner<f32>,
    // sample_rate: u32,
}

impl Analyzer {
    pub fn new(sample_rate: u32) -> Self { 
        Self {
            // fft_planner: FftPlanner::new(),
            // sample_rate
        }
    }

    pub fn feed_data(&mut self, data: &[f32]) {
        // TODO
    }
}