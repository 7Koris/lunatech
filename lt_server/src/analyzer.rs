use std::{ops::Range, thread}; 
use realfft::RealFftPlanner;

const LOW_RANGE: Range<f32> = 0.0..250.0; // Hz
const MID_RANGE: Range<f32> = 250.0..4000.0; // Hz
const HIGH_RANGE: Range<f32> = 4000.0..20000.0; // Hz 

#[derive(Default, Clone)]
#[non_exhaustive]
pub struct AudioFeatures {
    pub broad_range_peak_rms: f32,
    pub low_range_rms: f32,
    pub mid_range_rms: f32,
    pub high_range_rms: f32,
    pub fundamental_frequency: f32,
}

pub struct Analyzer {
    fft_planner: RealFftPlanner<f32>,
    channel_count: u16,
    sample_rate: u32,
    pub features: AudioFeatures,
}

impl Analyzer {
    pub fn new(channel_count: u16, sample_rate: u32) -> Self { 
        if channel_count < 1 {
            panic!("Channel count must be greater than 0");
        }

        Self {
            fft_planner: RealFftPlanner::new(),
            channel_count,
            sample_rate,
            features: AudioFeatures::default(),
        }
    }

    // "RealFFT matches the behaviour of RustFFT and does not normalize 
    // the output of either forward or inverse FFT. To get normalized results,
    // each element must be scaled by 1/sqrt(length), where length is the length
    // of the real-valued signal. If the processing involves both an FFT and an 
    // iFFT step, it is advisable to merge the two normalization steps to a single,
    // by scaling by 1/length."
    // TODO: normalize

    pub fn feed_data(&mut self, data: &[f32]) {
        assert!(self.channel_count > 0);
        
        let mut new_features = AudioFeatures::default();
        let mut broad_range_rms = vec![0.0; self.channel_count.into()];
        let mut low_range_rms = vec![0.0; self.channel_count.into()];
        let mut mid_range_rms = vec![0.0; self.channel_count.into()];
        let mut high_range_rms = vec![0.0; self.channel_count.into()];

    
        for channel_index in 0..self.channel_count {
            let channel_data = &data.iter().skip(channel_index.into()).cloned().collect::<Vec<f32>>(); // CECK SKIPPIG
            let fft_plan = self.fft_planner.plan_fft_forward(channel_data.len());
            let mut input_vec = fft_plan.make_input_vec();
            input_vec.copy_from_slice(channel_data.as_slice());
            let mut spectrum_vec = fft_plan.make_output_vec();
            let _ = fft_plan.process(&mut input_vec, &mut spectrum_vec); // realfft halves data length (avoiding redundant data)

            // |a| / (b^2 + w^2)^1/2, let |a| = 1 (https://pages.jh.edu/signals/spectra/spectra.html)
            let broad_range_magnitudes = spectrum_vec.iter().map(|x| x.norm_sqr()).collect::<Vec<f32>>();
            // https://www.ap.com/news/more-about-ffts (getting frequencies)
            let bin_size = self.sample_rate as f32 / spectrum_vec.len() as f32;
            let freqs = &broad_range_magnitudes.iter().enumerate().map(|(i, &_)| bin_size * i as f32).collect::<Vec<f32>>();
            
            let low_range_magnitudes = filter_freq_range(broad_range_magnitudes.as_slice(), freqs.as_slice(), LOW_RANGE);
            let mid_range_magnitudes = filter_freq_range(broad_range_magnitudes.as_slice(), freqs.as_slice(), MID_RANGE);
            let high_range_magnitudes = filter_freq_range(broad_range_magnitudes.as_slice(), freqs.as_slice(), HIGH_RANGE);

            //TODO HIGPASS FILTER

            broad_range_rms[channel_index as usize] = compute_rms(&broad_range_magnitudes);
            low_range_rms[channel_index as usize] = compute_rms(&low_range_magnitudes);
            mid_range_rms[channel_index as usize] = compute_rms(&mid_range_magnitudes);
            high_range_rms[channel_index as usize] = compute_rms(&high_range_magnitudes);
        }

        
        new_features.broad_range_peak_rms = broad_range_rms.iter().sum::<f32>() / self.channel_count as f32;
        new_features.low_range_rms = low_range_rms.iter().sum::<f32>() / self.channel_count as f32;
        new_features.mid_range_rms = mid_range_rms.iter().sum::<f32>() / self.channel_count as f32;
        new_features.high_range_rms = high_range_rms.iter().sum::<f32>() / self.channel_count as f32;

        self.features = new_features;
    }
} 

pub fn compute_rms(magnitudes: &[f32]) -> f32 {
    let half = magnitudes.len() / 2;
    let sum: f32 = magnitudes[..half].iter().sum();
    let mean = sum / magnitudes[..half].len() as f32;
    mean.sqrt()
}

pub fn compute_peak_rms(magnitudes: &[f32]) -> f32 {
    let half = magnitudes.len() / 2;
    let sum: f32 = magnitudes[half..].iter().sum();
    let mean = sum / magnitudes[half..].len() as f32;
    mean.sqrt() * f32::sqrt(2.0)
}

// pub fn compute_fundamental_frequency(spec_values: &[f32], freqs: &[f32]) -> f32 { 
//     let peak_index = 
//     freqs[peak_index] // * 2.0
// }

pub fn filter_freq_range(spec_values: &[f32], freqs: &[f32], range: Range<f32>) -> Vec<f32> {
    spec_values.iter().enumerate().filter_map(
        |(i, &mag)| {
            if range.contains(&freqs[i]) {
                Some(mag)
            } else {
                None
            }
        }
    ).collect::<Vec<f32>>()
}
