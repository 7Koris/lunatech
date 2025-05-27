use core::f32;
use std::ops::{ Range, RangeInclusive };
use lt_utilities::audio_features::{ AudioFeatures, Features };
use realfft::{ num_complex::Complex32, RealFftPlanner };

const BASS_RANGE: RangeInclusive<f32> = 0.0..=250.0; // Hz
const MID_RANGE: RangeInclusive<f32> = 250.0..=4000.0; // Hz
const TREBLE_RANGE: RangeInclusive<f32> = 4000.0..=15000.0; // Hz
const ROLLOFF_THRESHOLD: f32 = 0.75;
// const FLUX_BUFF_SIZE: usize = 256 * 16;

pub struct Analyzer {
    fft_planner: RealFftPlanner<f32>,
    channel_count: u16,
    sample_rate: u32,
    last_frame_buffer: Vec<Vec<f32>>,
    //oss_envelope: Vec<f32>,
    pub audio_features: AudioFeatures,
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
            last_frame_buffer: vec![Vec::new(); channel_count as usize],
            // oss_envelope: vec![0.0; FLUX_BUFF_SIZE],
            audio_features: AudioFeatures::default(),
        }
    }

    pub fn feed_data(&mut self, data: &[f32]) {
        assert!(self.channel_count > 0);
        assert!(data.len() % (self.channel_count as usize) == 0);

        let mut channels: Vec<Vec<f32>> = Vec::new();
        for _ in 0..self.channel_count {
            channels.push(Vec::new());
        }

        // Iterate for each channel and collect every nth element
        // Separates interleaved audio data into separate channels
        for chunk in data.chunks_exact(self.channel_count as usize) {
            (0..self.channel_count as usize).for_each(|channel_index| {
                channels[channel_index].push(chunk[channel_index]);
            });
        }

        let channel_features = {
            let result: Vec<Option<Features>> = channels
                .iter()
                .enumerate()
                .map(|(channel_index, channel_data)| {
                    // TODO: gain slider
                    let gain = 1.0;
                    let channel_data = &channel_data
                        .iter()
                        .map(|x| x * gain)
                        .collect::<Vec<f32>>();

                    let fft_plan = self.fft_planner.plan_fft_forward(channel_data.len());
                    let mut input_vec = fft_plan.make_input_vec();
                    input_vec.copy_from_slice(channel_data.as_slice());

                    let mut complex_spectrum = fft_plan.make_output_vec();
                    let _ = fft_plan.process(&mut input_vec, &mut complex_spectrum);

                    let bin_size = (self.sample_rate as f32) / (complex_spectrum.len() as f32);
                    let freqs = (0..complex_spectrum.len())
                        .into_iter()
                        .map(|i| bin_size * (i as f32))
                        .collect::<Vec<f32>>();

                    // separate channels first then calc mag spectrums separately
                    let bass_spectrum = filter_complex_by_range(
                        &complex_spectrum.as_slice(),
                        freqs.as_slice(),
                        BASS_RANGE
                    );

                    let mid_spectrum = filter_complex_by_range(
                        &complex_spectrum.as_slice(),
                        freqs.as_slice(),
                        MID_RANGE
                    );

                    let treble_spectrum = filter_complex_by_range(
                        &complex_spectrum.as_slice(),
                        freqs.as_slice(),
                        TREBLE_RANGE
                    );

                    let magnitude_spectrum = compute_magnitude_spectrum(&complex_spectrum);
                    let bass_power_spectrum = compute_power_spectrum(&bass_spectrum);
                    let mid_power_spectrum = compute_power_spectrum(&mid_spectrum);
                    let treble_power_spectrum = compute_power_spectrum(&treble_spectrum);
                    let power_spectrum = compute_power_spectrum(&complex_spectrum);

                    let zcr = compute_zcr(channel_data);

                    // spectral centroid
                    let spectral_centroid = compute_spectral_centroid(
                        magnitude_spectrum.as_slice(),
                        freqs.as_slice()
                    );
                    let spectral_centroid = 1.0 * (spectral_centroid / freqs[freqs.len() - 1]);

                    // Spectral rolloff
                    let rolloff: f32 = compute_rolloff(
                        magnitude_spectrum.as_slice(),
                        freqs.as_slice()
                    );
                    let rolloff = 1.0 * (rolloff / freqs[freqs.len() - 1]);

                    // total variation
                    let tv =
                        magnitude_spectrum
                            .windows(2)
                            .map(|x| (x[0] - x[1]).abs())
                            .sum::<f32>() / (magnitude_spectrum.len() as f32);

                    // Spectral flux
                    let last_frame = self.last_frame_buffer[channel_index]
                        .iter()
                        .cloned()
                        .collect::<Vec<f32>>();
                    self.last_frame_buffer[channel_index].clone_from(&power_spectrum);

                    // check which vec is longer and cut it down to the length of the shorter one
                    let min_idx = std::cmp::min(magnitude_spectrum.len(), last_frame.len());
                    let broad_slice: &[f32] = &magnitude_spectrum[..min_idx];
                    let last_frame_slice: &[f32] = &last_frame[..min_idx];

                    let max_flux = (1.0_f32).powf(2.0) * ((broad_slice.len() as f32) / 2.0);
                    let min_flux = 0.0;

                    let flux = broad_slice
                        .iter()
                        .enumerate()
                        .map(|(i, &x)| { (x - last_frame_slice[i]).powf(2.0) })
                        .sum::<f32>()
                        .sqrt();

                    let flux = ((flux - min_flux) / (max_flux - min_flux)).clamp(0.0, 1.0);

                    Some((
                        compute_rms(&power_spectrum, power_spectrum.len() as f32),
                        compute_rms(&bass_power_spectrum, power_spectrum.len() as f32),
                        compute_rms(&mid_power_spectrum, power_spectrum.len() as f32),
                        compute_rms(&treble_power_spectrum, power_spectrum.len() as f32),
                        zcr,
                        spectral_centroid,
                        flux,
                        rolloff,
                        tv,
                    ))
                })
                .collect();
            result
        };

        let channel_features: Vec<Features> = channel_features.into_iter().flatten().collect();
        self.audio_features.rms.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.0)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.bass.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.1)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.mid.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.2)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.treble.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.3)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.centroid.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.5)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.zcr.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.4)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.flux.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.6)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.rolloff.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.7)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
        self.audio_features.tv.set(
            (
                channel_features
                    .iter()
                    .map(|x| x.8)
                    .sum::<f32>() / (self.channel_count as f32)
            ).clamp(0.0, 1.0)
        );
    }
}

pub fn compute_magnitude_spectrum(spectrum: &[Complex32]) -> Vec<f32> {
    spectrum
        .iter()
        .map(|x| { x.norm() })
        .collect::<Vec<f32>>()
}

pub fn compute_power_spectrum(complex_spectrum: &[Complex32]) -> Vec<f32> {
    complex_spectrum
        .iter()
        .map(|x| { x.norm_sqr() / (complex_spectrum.len() as f32).sqrt() })
        .collect::<Vec<f32>>()
}

pub fn compute_rms(power_spectrum: &[f32], len: f32) -> f32 {
    let sum: f32 = power_spectrum.iter().sum();
    let mean = sum / len;
    mean.sqrt()
}

fn compute_zcr(magnitude_spectrum: &[f32]) -> f32 {
    let sign = |x: f32| {
        if x > 0.0 { 1.0 } else { -1.0 }
    };

    let mut last_x = magnitude_spectrum[0];
    let zcr =
        magnitude_spectrum
            .iter()
            .skip(1)
            .map(|x| {
                let diff = sign(*x) + sign(last_x);
                last_x = *x;
                if diff == 0.0 {
                    1.0
                } else {
                    0.0
                }
            })
            .sum::<f32>() / (magnitude_spectrum.len() as f32);
    zcr
}

fn compute_spectral_centroid(magnitude_spectrum: &[f32], freqs: &[f32]) -> f32 {
    let sum = magnitude_spectrum.iter().sum::<f32>();
    let spectral_centroid = if sum == 0.0 {
        0.0
    } else {
        magnitude_spectrum
            .iter()
            .enumerate()
            .map(|(i, &x)| { freqs[i] * x.abs() })
            .sum::<f32>() / sum
    };
    spectral_centroid
}

fn compute_rolloff(magnitude_spectrum: &[f32], freqs: &[f32]) -> f32 {
    let threshold = ROLLOFF_THRESHOLD * magnitude_spectrum.iter().sum::<f32>();
    let mut sum = 0.0;

    for i in 0..magnitude_spectrum.len() {
        sum += magnitude_spectrum[i];
        if sum > threshold {
            return freqs[i];
        }
    }
    return 0.0;
}

pub fn filter_magnitudes_by_range(
    spec_values: &[f32],
    freqs: &[f32],
    range: Range<f32>
) -> Vec<f32> {
    spec_values
        .iter()
        .enumerate()
        .filter_map(|(i, &mag)| {
            if range.contains(&freqs[i]) { Some(mag) } else { None }
        })
        .collect::<Vec<f32>>()
}

pub fn filter_complex_by_range(
    spec_values: &[Complex32],
    freqs: &[f32],
    range: RangeInclusive<f32>
) -> Vec<Complex32> {
    spec_values
        .iter()
        .enumerate()
        .filter_map(|(i, &mag)| {
            if range.contains(&freqs[i]) { Some(mag) } else { None }
        })
        .collect::<Vec<Complex32>>()
}
