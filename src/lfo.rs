use std::f64::consts::TAU;

use crate::hw::SAMPLE_RATE;

#[derive(Clone, Debug)]
pub struct Lfo {
    freq: f64,
    phase: f64,
    phase_incr: f64,
}

impl Lfo {
    pub fn new(freq: f64) -> Self {
        Self {
            freq,
            phase: 0.0,
            phase_incr: freq / SAMPLE_RATE as f64,
        }
    }

    pub fn freq(&self) -> f64 {
        self.freq
    }

    pub fn set_freq(&mut self, freq: f64) {
        if freq == 0.0 {
            self.phase = 0.0;
        }

        self.freq = freq;
        self.phase_incr = freq / SAMPLE_RATE as f64;
    }

    pub fn output(&mut self) -> f64 {
        let sample = (self.phase * TAU).sin();

        self.phase += self.phase_incr;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}
