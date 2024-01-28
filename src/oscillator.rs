use std::f64::consts::{PI, TAU};

use crate::envelope::Envelope;
use crate::hw::SAMPLE_RATE;
use crate::lfo::Lfo;

#[derive(Debug, Clone)]
pub struct Oscillator {
    pub enabled: bool,
    freq: f64,
    phase: f64,
    phase_incr: f64,
    buffer: Option<i16>,
    env: Envelope,
    lfo: Lfo,
}

impl Oscillator {
    pub fn new(freq: f64, vol: u16) -> Self {
        Self {
            enabled: true,
            freq,
            phase: 0.0,
            phase_incr: freq / SAMPLE_RATE as f64,
            buffer: None,
            env: Envelope::new(vol),
            lfo: Lfo::new(0.0),
        }
    }

    pub fn freq(&self) -> f64 {
        self.freq
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
        self.phase_incr = freq / SAMPLE_RATE as f64;
    }

    pub fn adjust_vibrato(&mut self, freq: f64) {
        self.phase_incr = freq / SAMPLE_RATE as f64;
    }

    pub fn set_vol(&mut self, vol: u16) {
        self.env.set_volume(vol);
    }

    pub fn set_vibrato(&mut self, freq: f64) {
        self.lfo.set_freq(freq);
    }

    fn sample_sawtooth(phase: f64) -> f64 {
        phase - phase.floor()
    }
    fn sample_triangle(phase: f64) -> f64 {
        (phase * TAU).sin().asin()
    }

    pub fn output(&mut self) -> i16 {
        if self.buffer.is_some() {
            return self.buffer.take().unwrap();
        }

        // let sample = Self::sample_triangle(self.phase);
        let sample = Self::sample_sawtooth(self.phase);
        let sample = sample * (self.env.volume() * 2.0) / PI;

        let vibrato = self.lfo.output();
        let delta = (self.freq * 2.0_f64.powf(10.0 / 1200.0)) - self.freq;
        let new_freq = self.freq + delta * vibrato;

        self.adjust_vibrato(new_freq);

        self.enabled = !self.env.adjust_volume();

        self.phase += self.phase_incr;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        self.buffer.replace(sample as i16);

        sample as i16
    }
}
