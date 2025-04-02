use core::f32;
use std::{f32::consts::PI, ops::Range}; 
use lt_utilities::audio_features::{AtomicAudioFeatures, Features};
use realfft::{num_traits::Signed, RealFftPlanner};
use rayon::prelude::*;

use lt_utilities::ArcMutex;

const LOW_RANGE: Range<f32> = 0.0..250.; // Hz
const MID_RANGE: Range<f32> = 250.0..4000.; // Hz
const HIGH_RANGE: Range<f32> = 4000.0..20000.; // Hz 
// const FLUX_BUFF_SIZE: usize = 256 * 16;

pub struct Analyzer {
    fft_planner: ArcMutex<RealFftPlanner<f32>>, 
    channel_count: u16,
    sample_rate: u32,
    last_frame_buffer: Vec<ArcMutex<Vec<f32>>>,
    //oss_envelope: Vec<f32>,
    pub audio_features: AtomicAudioFeatures,
}

fn compute_zcr(input: &[f32]) -> f32 {
    let sign = |x: f32| {
        if x > 0. {
            1.
        } else {
            -1.
        }
    };

    let mut last_x = input[0];
    let zcr = input.iter().skip(1).map(|x| {
        let diff = sign(*x) + sign(last_x);
        last_x = *x;
        if diff == 0. {
            1.
        } else {
            0.
        }
    }).sum::<f32>() / input.len() as f32;
    zcr
}

fn compute_spectral_centroid(input: &[f32], freqs: &[f32]) -> f32 {
    let sum = input.iter().sum::<f32>();
    let spectral_centroid = if sum == 0. {
        0.
    } else {
        input.iter().enumerate().map(|(i, &x)| {
            freqs[i] * x.abs()
        }).sum::<f32>() / sum
    };
    spectral_centroid
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
            last_frame_buffer: vec![ArcMutex!(Vec::new()); channel_count as usize],
            // oss_envelope: vec![0.0; FLUX_BUFF_SIZE],
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
        
        // TODO: Make proper multithreaded
        let channel_lock = channels.lock();
        let channel_features = if let Ok(channels) = channel_lock {
            let result: Vec<Option<Features>> = channels.iter().enumerate() .map(|(channel_index, channel_data)| {
                if let Ok(mut fft_planner) = self.fft_planner.lock() {          

                    // TODO: automatic gain correction?

                    let fft_plan = fft_planner.plan_fft_forward(channel_data.len());
                    let mut input_vec = fft_plan.make_input_vec();
                    
                    input_vec.copy_from_slice(channel_data.as_slice());
                    //apply_hann_window(&mut input_vec);
                    
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
                        z
                    }).collect::<Vec<f32>>();

                    let broad_range_magnitudes_log_compressed = broad_range_magnitudes.iter().map(|x| {
                        (1.0 + x).log(10.0) / (1.0) // Log-Compression step, no GAMMA
                    }).collect::<Vec<f32>>();
                    
                    //  https://www.ap.com/news/more-about-ffts (getting frequencies)
                    let bin_size = self.sample_rate as f32 / spectrum_vec.len() as f32;
                    let freqs = &broad_range_magnitudes.iter().enumerate().map(|(i, &_)| bin_size * i as f32).collect::<Vec<f32>>();
         
                    let low_range_magnitudes = get_filtered_by_range(broad_range_magnitudes_log_compressed.as_slice(), freqs.as_slice(), LOW_RANGE);
                    let mid_range_magnitudes = get_filtered_by_range(broad_range_magnitudes_log_compressed.as_slice(), freqs.as_slice(), MID_RANGE);
                    let high_range_magnitudes = get_filtered_by_range(broad_range_magnitudes_log_compressed.as_slice(), freqs.as_slice(), HIGH_RANGE);

                    let zcr = compute_zcr(channel_data);
                    let spectral_centroid = compute_spectral_centroid(broad_range_magnitudes.as_slice(), freqs.as_slice());

                    // Spectral flux
                    let mut last_buf = self.last_frame_buffer.get(channel_index).unwrap().lock().unwrap();    
                    let last_frame = last_buf.iter().cloned().collect::<Vec<f32>>();
                    last_buf.clear();
                    last_buf.extend(channel_data.iter().cloned());
                    
                    // check which vec is longer and cut it down to the length of the shorter one
                    let broad_length = broad_range_magnitudes.len();
                    let last_frame_length = last_frame.len();
                    let broad_slice: &[f32] = &broad_range_magnitudes[..std::cmp::min(broad_length, last_frame_length)];
                    let last_frame_slice: &[f32] = &last_frame[..std::cmp::min(broad_length, last_frame_length)];

                    let flux = broad_slice.iter().enumerate().map(|(i, &x)| {
                        (x - last_frame_slice[i]).powf(2.)
                    }).sum::<f32>().sqrt();
                    
                    Some((
                        compute_rms(&broad_range_magnitudes_log_compressed) / 2.0,
                        compute_rms(&low_range_magnitudes) / 2.0,
                        compute_rms(&mid_range_magnitudes) / 2.0,
                        compute_rms(&high_range_magnitudes) / 2.0,
                        zcr,
                        spectral_centroid,
                        flux,
                    ))
                } else {
                    None
                }
            }).collect();
           result
        } else {
            vec![None]
        };
        
        let channel_features: Vec<Features> = channel_features.into_iter().flatten().collect();
        self.audio_features.broad_range_rms.set((channel_features.iter().map(|x| x.0).sum::<f32>() / self.channel_count as f32).clamp(0., 1.));
        self.audio_features.low_range_rms.set((channel_features.iter().map(|x| x.1).sum::<f32>() / self.channel_count as f32).clamp(0., 1.));
        self.audio_features.mid_range_rms.set((channel_features.iter().map(|x| x.2).sum::<f32>() / self.channel_count as f32).clamp(0., 1.));
        self.audio_features.high_range_rms.set((channel_features.iter().map(|x| x.3).sum::<f32>() / self.channel_count as f32).clamp(0., 1.));
        self.audio_features.spectral_centroid.set(channel_features.iter().map(|x| x.5).sum::<f32>() / self.channel_count as f32);
        self.audio_features.zcr.set(channel_features.iter().map(|x| x.4).sum::<f32>() / self.channel_count as f32);
        self.audio_features.flux.set(channel_features.iter().map(|x| x.6).sum::<f32>() / self.channel_count as f32);
    }
} 

pub fn compute_rms(magnitudes: &[f32]) -> f32 {
    let sum: f32 = magnitudes.iter().map(|x| x.powf(2.)).sum();
    let mean = sum / magnitudes.len() as f32;
    mean.sqrt()
}

pub fn compute_peak_rms(magnitudes: &[f32]) -> f32 {
    let sum: f32 = magnitudes.iter().map(|x| x.powf(2.)).sum();
    let mean = sum / magnitudes.len() as f32;
    mean.sqrt() * f32::sqrt(2.0)
}

// pub fn compute_fundamental_frequency(spec_values: &[f32], freqs: &[f32]) -> f32 { 
//     let peak_index = 
//     freqs[peak_index] // * 2.0
// }

pub fn get_filtered_by_range(spec_values: &[f32], freqs: &[f32], range: Range<f32>) -> Vec<f32> {
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

pub fn get_normalized_mags(magnitudes: &[f32]) -> Vec<f32> {
    let minx = magnitudes.iter().fold(f32::INFINITY, |acc, x| acc.min(*x));
    let maxx = magnitudes.iter().fold(f32::NEG_INFINITY, |acc, x| acc.max(*x));
    magnitudes.iter().map(|x|  {
        if x == &0.0 {
            return 0.0;
        }
        (x - minx) / (maxx - minx)
    }).collect::<Vec<f32>>()
}
