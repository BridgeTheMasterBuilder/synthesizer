use std::fs::File;

use wav::{BitDepth, WAV_FORMAT_PCM};

use crate::hw::SAMPLE_RATE;

pub fn render_to_file(filename: &str, data: Vec<i16>) {
    let mut file = File::create(filename).unwrap();
    let header = wav::Header::new(WAV_FORMAT_PCM, 2, SAMPLE_RATE, 16);

    wav::write(header, &BitDepth::Sixteen(data), &mut file).unwrap();
}
