use std:: io::Write;
use rayon::prelude::*;

use lt_server::analyzer::Analyzer;

fn main() {
    let file_dir = "/home/koris/lunatech/testing/src/wav4/";
    let files = std::fs::read_dir(file_dir).unwrap().map(|x| x.unwrap().path()).collect::<Vec<std::path::PathBuf>>();
    std::fs::create_dir_all("lt_wav_csv2").unwrap();
    
    let now = std::time::Instant::now();
    files.par_iter().for_each(|file| {
        let base_name: &str = file.file_name().unwrap().to_str().unwrap();

        let csv = std::fs::File::create(format!("lt_wav_csv2/{}.csv", base_name)).unwrap();
        let mut writer = std::io::BufWriter::new(csv);

        let file_p = file;
        let mut reader = hound::WavReader::open(file_p).unwrap();

        let mut analyzer = Analyzer::new(1, 44100);
        let sample_vec = reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .iter()
            .map(|x| *x as f32 / i16::MAX as f32)
            .collect::<Vec<f32>>();

        writer.write_all("b_size,time,rms,zcr,spectral_centroid,latency\n".to_string().as_bytes()).unwrap();
        for i in 8..17 {
            let now = std::time::Instant::now();
            let mut chunk_cnt = 0;
            let b_size = 2_i32.pow(i) as usize;
            let chunks = sample_vec.chunks(b_size);
            let first_chunk_len = chunks.clone().next().unwrap().len() / 2; // wasteful!
            
            chunks.for_each(|x| {
                analyzer.feed_data(x);
                writer.write_all(format!("{},{},{},{},{},{}\n", b_size,chunk_cnt as f32 / 44100. * (first_chunk_len) as f32, analyzer.audio_features.broad_range_rms.get(), analyzer.audio_features.zcr.get(), analyzer.audio_features.spectral_centroid.get(), now.elapsed().as_millis()).as_bytes()).unwrap();
                chunk_cnt += 1;
            }); 
        }

        writer.flush().unwrap();
    });

    println!("elapsed: {:?}", now.elapsed());
}
