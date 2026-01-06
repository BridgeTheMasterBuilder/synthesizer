use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Div, Mul};

#[derive(Clone, Copy, Debug)]
pub enum Interval {
    Ratio(usize, usize),
    Cents(f64),
}

impl Interval {
    pub fn reciprocal(self) -> Interval {
        match self {
            Interval::Ratio(n, m) => Interval::Ratio(m, n),
            Interval::Cents(c) => Interval::Cents(-c),
        }
    }
}

impl Mul for Interval {
    type Output = Self;

    fn mul(self, other: Interval) -> Interval {
        match self {
            Interval::Ratio(a, b) => match other {
                Interval::Ratio(c, d) => Interval::Ratio(a * c, b * d),
                Interval::Cents(c) => {
                    let cents = (a as f64 / b as f64).log2() * 1200.0;

                    Interval::Cents(c + cents)
                }
            },
            Interval::Cents(x) => match other {
                Interval::Ratio(n, m) => {
                    let cents = (n as f64 / m as f64).log2() * 1200.0;

                    Interval::Cents(x + cents)
                }
                Interval::Cents(y) => Interval::Cents(x + y),
            },
        }
    }
}

impl Div for Interval {
    type Output = Self;

    fn div(self, other: Interval) -> Interval {
        self * other.reciprocal()
    }
}

impl Mul<Interval> for f64 {
    type Output = Self;

    fn mul(self, other: Interval) -> f64 {
        match other {
            Interval::Ratio(n, m) => self * (n as f64 / m as f64),
            Interval::Cents(c) => self * 2.0_f64.powf(c / 1200.0),
        }
    }
}

impl Div<Interval> for f64 {
    type Output = Self;

    fn div(self, other: Interval) -> f64 {
        self * other.reciprocal()
    }
}

#[derive(Debug)]
pub struct Scale {
    intervals: Vec<Interval>,
}

impl Scale {
    pub fn new(intervals: Vec<Interval>) -> Scale {
        Scale { intervals }
    }

    pub fn size(&self) -> usize {
        self.intervals.len()
    }

    pub fn mode(&self, offset: usize) -> Vec<Interval> {
        if offset > self.size() {
            panic!("Oops");
        }

        let mut intervals = self.intervals.clone();

        intervals.rotate_right(offset);

        if offset > 0 {
            intervals[0..offset]
                .iter_mut()
                .for_each(|v| *v = *v / Interval::Ratio(2, 1));
        }

        intervals
    }

    pub fn interval_at(&self, i: isize) -> Interval {
        self.intervals[i.rem_euclid(self.intervals.len() as isize) as usize]
    }

    pub fn frequencies(&self, base_freq: f64, base_note: usize) -> Vec<f64> {
        if base_note > self.size() {
            panic!("Oops");
        }

        let mode = self.mode(base_note);

        dbg!(&mode);

        // TODO
        mode.iter().map(|&interval| base_freq * interval).collect()
    }

    // TODO non-octave
    pub fn frequency_at(&self, base_freq: f64, idx: isize) -> f64 {
        let size = self.size();
        // let octaves = (idx.abs() as usize).saturating_sub(1) / size;
        let octaves = (idx.abs() as usize) / size;
        // let i = idx.rem_euclid(size as isize) as usize;

        let interval = self.interval_at(idx);

        if idx.signum() < 0 {
            let octaves = (idx.abs() as usize).saturating_sub(1) / size;
            let octaves = octaves + 1;

            dbg!(
                idx,
                octaves,
                interval / Interval::Ratio(2_usize.pow(octaves as u32), 1)
            );
            base_freq * (interval / Interval::Ratio(2_usize.pow(octaves as u32), 1))
        } else {
            let octaves = (idx.abs() as usize) / size;
            dbg!(
                idx,
                octaves,
                interval * Interval::Ratio(2_usize.pow(octaves as u32), 1)
            );
            base_freq * (interval * Interval::Ratio(2_usize.pow(octaves as u32), 1))
        }
    }
}

// TODO ability to specify base frequency/note for cyclical scales
pub fn parse_scala_file(filename: &str) -> Scale {
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

        let mut intervals = Vec::with_capacity(count);

        // intervals.push(Interval::Ratio(1, 1));

        for i in 0..count {
            let data = &lines[idx + i];

            if data.starts_with('!') {
                continue;
            } else if data.contains('.') {
                let cents = data.parse::<f64>().unwrap();

                intervals.push(Interval::Cents(cents));
            } else {
                if data.contains('/') {
                    let ratio = data.split('/').collect::<Vec<&str>>();
                    let numerator = ratio[0].parse::<usize>().unwrap();
                    let denominator = ratio[1].parse::<usize>().unwrap();

                    intervals.push(Interval::Ratio(numerator, denominator));
                } else {
                    let n = data.parse::<usize>().unwrap();

                    intervals.push(Interval::Ratio(n, 1));
                }
            }
        }

        Scale::new(intervals)
    } else {
        panic!("WARNING ")
    }
}

pub fn scale_to_tuning(scale: Scale, base_freq: f64, base_note: u8) -> [f64; 128] {
    let mut tuning = [0.0; 128];
    tuning[base_note as usize] = base_freq;

    for i in base_note as usize + 1..128 {
        let idx = i as isize - base_note as isize - 1;
        tuning[i] = scale.frequency_at(base_freq, idx);
    }

    for i in 0..base_note as usize {
        let idx = -(i as isize + 1);
        tuning[base_note as usize - i] = scale.frequency_at(base_freq, idx);
    }

    tuning
}

// pub fn parse_scala_file(filename: &str) -> [f64; 128] {
//     if let Ok(file) = File::open(filename) {
//         let reader = BufReader::new(file);
//
//         // TODO
//         let lines: Vec<String> = reader
//             .lines()
//             .map(|x| x.map(|x| x.trim().to_string()))
//             .collect::<Result<_, _>>()
//             .unwrap();
//
//         let mut idx = 0;
//
//         while lines[idx].starts_with('!') {
//             idx += 1;
//         }
//
//         idx += 1;
//
//         while lines[idx].starts_with('!') {
//             idx += 1;
//         }
//
//         let count = lines[idx].parse::<usize>().unwrap();
//         // if count != 127 {
//         //     panic!("Not implemented");
//         // }
//
//         idx += 1;
//
//         while lines[idx].starts_with('!') {
//             idx += 1;
//         }
//
//         let base_freq = 8.25;
//
//         let mut frequencies = [base_freq; 128];
//
//         let start = idx;
//
//         for i in 1..128 {
//             let data = &lines[idx];
//
//             if data.starts_with('!') {
//                 continue;
//             } else if data.contains('.') {
//                 let cents = data.parse::<f64>().unwrap();
//
//                 let octave = ((i - 1) / count) as f64;
//
//                 frequencies[i] = base_freq * 2.0_f64.powf((cents + octave * 1200.0) / 1200.0);
//                 // frequencies[idx] = base_freq * 2.0_f64.powf(cents / 1200.0);
//             } else {
//                 let interval = if data.contains('/') {
//                     let ratio = data.split('/').collect::<Vec<&str>>();
//                     let numerator = ratio[0].parse::<f64>().unwrap();
//                     let denominator = ratio[1].parse::<f64>().unwrap();
//
//                     numerator / denominator
//                 } else {
//                     data.parse::<f64>().unwrap()
//                 };
//
//                 let octave = ((i - 1) / count) as f64;
//
//                 frequencies[i] = base_freq * 2.0_f64.powf(octave) * interval;
//                 // frequencies[idx] = base_freq * interval;
//             }
//
//             idx = if idx - start + 1 == count {
//                 start
//             } else {
//                 idx + 1
//             };
//
//             // dbg!(idx);
//         }
//         // dbg!(&frequencies);
//
//         frequencies
//     } else {
//         panic!("WARNING ")
//     }
// }
