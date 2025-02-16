use std::ops::Range; 
use lt_utilities::audio_features::{AtomicAudioFeatures, Features};
use realfft::RealFftPlanner;
use rayon::prelude::*;

use lt_utilities::ArcMutex;

const LOW_RANGE: Range<f32> = 0.0..250.0; // Hz
const MID_RANGE: Range<f32> = 250.0..4000.0; // Hz
const HIGH_RANGE: Range<f32> = 4000.0..20000.0; // Hz 
const GAMMA: f32 = 1.8; // Used for log compression

pub struct Analyzer {
    fft_planner: ArcMutex<RealFftPlanner<f32>>, 
    channel_count: u16,
    sample_rate: u32,
    pub audio_features: AtomicAudioFeatures,
}

impl Analyzer {

    pub fn new(channel_count: u16, sample_rate: u32) -> Self { 
        if channel_count < 1 {
            panic!("Channel count must be greater than 0");
        }

        Self {
            fft_planner: ArcMutex!(RealFftPlanner::new()),
            channel_count,
            sample_rate,
            audio_features: AtomicAudioFeatures::default(),
        }
    }

    pub fn feed_data(&mut self, data: &[f32]) {
        assert!(self.channel_count > 0);
        
        let channels: ArcMutex<Vec<Vec<f32>>> = ArcMutex!(Vec::new());
        (0..self.channel_count).collect::<Vec<u16>>().par_iter().for_each(|channel_index| {
            let channel_data  = data.iter().skip((*channel_index) as usize).cloned().collect::<Vec<f32>>();
            if let Ok(mut channels) = channels.lock() {
                channels.push(channel_data);
            }
        });

        let channel_lock = channels.lock();
        let mut channel_features = if let Ok(channels) = channel_lock {
            let result: Vec<Option<Features>> = channels.iter().map(|channel_data| {  
                if let Ok(mut fft_planner) = self.fft_planner.lock() {
                    let fft_plan = fft_planner.plan_fft_forward(channel_data.len());
                    let mut input_vec = fft_plan.make_input_vec();
                    input_vec.copy_from_slice(channel_data.as_slice());
                    let mut spectrum_vec = fft_plan.make_output_vec();
                    let _ = fft_plan.process(&mut input_vec, &mut spectrum_vec); // realfft halves data length (avoiding redundant data)
                    
                    // |a| / (b^2 + w^2)^1/2, let |a| = 1 (https://pages.jh.edu/signals/spectra/spectra.html)
                    let size = spectrum_vec.len() as f32;
                    let broad_range_magnitudes = spectrum_vec.iter().map(|x| {
                        let a = x.norm_sqr();
                        
                        if a == 0.0 {
                            return 0.0;
                        }

                        let z = a / size.sqrt(); // Normalization step
                        (1.0 + z * GAMMA).log(10.0) / (1.0 + z) // Log-Compression step
                       //z
                    }).collect::<Vec<f32>>();
                    

                    
                    //  https://www.ap.com/news/more-about-ffts (getting frequencies)
                    let bin_size = self.sample_rate as f32 / spectrum_vec.len() as f32;
                    let freqs = &broad_range_magnitudes.iter().enumerate().map(|(i, &_)| bin_size * i as f32).collect::<Vec<f32>>();
         
                    let low_range_magnitudes = filter_freq_range(broad_range_magnitudes.as_slice(), freqs.as_slice(), LOW_RANGE);
                    let mid_range_magnitudes = filter_freq_range(broad_range_magnitudes.as_slice(), freqs.as_slice(), MID_RANGE);
                    let high_range_magnitudes = filter_freq_range(broad_range_magnitudes.as_slice(), freqs.as_slice(), HIGH_RANGE);
                    
                    //TODO HIGPASS FILTER

                    // normalize broad_range mags even further 
                    // TODO: DO THIS BETTER!
                    let minx = broad_range_magnitudes.iter().fold(f32::INFINITY, |acc, x| acc.min(*x));
                    let maxx = broad_range_magnitudes.iter().fold(f32::NEG_INFINITY, |acc, x| acc.max(*x));
                    let broad_range_magnitudes = broad_range_magnitudes.iter().map(|x|  {
                        if x == &0.0 {
                            return 0.0;
                        }
                        (x - minx) / (maxx - minx)
                    }).collect::<Vec<f32>>();

                    
                    Some((
                        compute_rms(&broad_range_magnitudes),
                        compute_rms(&low_range_magnitudes),
                        compute_rms(&mid_range_magnitudes),
                        compute_rms(&high_range_magnitudes),
                    ))
                } else {
                    None
                }
            }).collect();
           result
        } else {
            vec![None]
        };

        channel_features.retain(|x| x.is_some());
        let channel_features: Vec<Features> = channel_features.into_iter().flatten().collect();
        self.audio_features.broad_range_peak_rms.set(channel_features.iter().map(|x| x.0).sum::<f32>() / self.channel_count as f32);
        self.audio_features.low_range_rms.set(channel_features.iter().map(|x| x.1).sum::<f32>() / self.channel_count as f32);
        self.audio_features.mid_range_rms.set(channel_features.iter().map(|x| x.2).sum::<f32>() / self.channel_count as f32);
        self.audio_features.high_range_rms.set(channel_features.iter().map(|x| x.3).sum::<f32>() / self.channel_count as f32);
    }
} 

pub fn compute_rms(magnitudes: &[f32]) -> f32 {
    let sum: f32 = magnitudes.iter().sum();
    let mean = sum / magnitudes.len() as f32;
    mean.sqrt()
}

pub fn compute_peak_rms(magnitudes: &[f32]) -> f32 {
    let sum: f32 = magnitudes.iter().sum();
    let mean = sum / magnitudes.len() as f32;
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
