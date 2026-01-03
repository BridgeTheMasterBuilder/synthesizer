use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn parse_scala_file(filename: &str) -> [f64; 128] {
    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);

        let base_freq = 13.75;

        let mut frequencies = [base_freq; 128];

        let mut read_desc = false;
        let mut read_count = false;

        let mut count;
        let mut idx = 10;

        for line in reader.lines() {
            let line = line.unwrap();
            let data = line.trim();

            if data.starts_with('!') {
                continue;
            } else if !read_desc {
                read_desc = true;
                continue;
            } else if !read_count {
                read_count = true;
                count = data.parse::<usize>().unwrap();
                if count != 128 {
                    panic!("Not implemented");
                }
                continue;
            } else if data.contains('.') {
                let cents = data.parse::<f64>().unwrap();

                // let octave = idx as f64 / count as f64;

                // frequencies[idx] = base_freq * 2.0_f64.powf((cents + octave * 1200.0) / 1200.0);
                frequencies[idx] = base_freq * 2.0_f64.powf(cents / 1200.0);

                idx += 1;
            } else {
                let interval = if data.contains('/') {
                    let ratio = data.split('/').collect::<Vec<&str>>();
                    let numerator = ratio[0].parse::<f64>().unwrap();
                    let denominator = ratio[1].parse::<f64>().unwrap();

                    numerator / denominator
                } else {
                    data.parse::<f64>().unwrap()
                };

                // let octave = (idx / count) as f64;

                // frequencies[idx] = base_freq * 2.0_f64.powf(octave) * interval;
                frequencies[idx] = base_freq * interval;
                idx += 1;
            }
        }

        dbg!(&frequencies);

        frequencies
    } else {
        panic!("WARNING ")
    }
}
