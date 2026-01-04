use std::fs::File;
use std::io::{BufRead, BufReader};

// TODO ability to specify base frequency/note for cyclical scales
pub fn parse_scala_file(filename: &str) -> [f64; 128] {
    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);

        // TODO
        let lines: Vec<String> = reader
            .lines()
            .map(|x| x.map(|x| x.trim().to_string()))
            .collect::<Result<_, _>>()
            .unwrap();

        let mut idx = 0;

        while lines[idx].starts_with('!') {
            idx += 1;
        }

        idx += 1;

        while lines[idx].starts_with('!') {
            idx += 1;
        }

        let count = lines[idx].parse::<usize>().unwrap();
        // if count != 127 {
        //     panic!("Not implemented");
        // }

        idx += 1;

        while lines[idx].starts_with('!') {
            idx += 1;
        }

        let base_freq = 8.25;

        let mut frequencies = [base_freq; 128];

        let start = idx;

        for i in 1..128 {
            let data = &lines[idx];

            if data.starts_with('!') {
                continue;
            } else if data.contains('.') {
                let cents = data.parse::<f64>().unwrap();

                let octave = ((i - 1) / count) as f64;

                frequencies[i] = base_freq * 2.0_f64.powf((cents + octave * 1200.0) / 1200.0);
                // frequencies[idx] = base_freq * 2.0_f64.powf(cents / 1200.0);
            } else {
                let interval = if data.contains('/') {
                    let ratio = data.split('/').collect::<Vec<&str>>();
                    let numerator = ratio[0].parse::<f64>().unwrap();
                    let denominator = ratio[1].parse::<f64>().unwrap();

                    numerator / denominator
                } else {
                    data.parse::<f64>().unwrap()
                };

                let octave = ((i - 1) / count) as f64;

                frequencies[i] = base_freq * 2.0_f64.powf(octave) * interval;
                // frequencies[idx] = base_freq * interval;
            }

            idx = if idx - start + 1 == count {
                start
            } else {
                idx + 1
            };

            // dbg!(idx);
        }
        // dbg!(&frequencies);

        frequencies
    } else {
        panic!("WARNING ")
    }
}
