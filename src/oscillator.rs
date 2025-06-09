use std::f64::consts::TAU;

use crate::hw::SAMPLE_RATE;

#[derive(Clone, Debug, Copy)]
pub enum Waveform {
    Sine,
    Pulse,
    Triangle,
    Sawtooth,
}

#[derive(Clone, Debug)]
pub struct Oscillator {
    freq: f64,
    phase: f64,
    phase_incr: f64,
    duty: f64,
    waveform: Waveform,
}

impl Oscillator {
    pub fn new(freq: f64) -> Self {
        Self {
            freq,
            phase: 0.0,
            phase_incr: freq / SAMPLE_RATE as f64,
            duty: 0.5,
            waveform: Waveform::Sine,
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

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_duty(&mut self, duty: f64) {
        self.duty = duty;
    }

    pub fn output(&mut self) -> f64 {
        let sample = self.sample();

        self.advance_phase(0.0);

        sample
    }

    pub fn sample(&self) -> f64 {
        match self.waveform {
            Waveform::Sine => (self.phase * TAU).sin(),
            Waveform::Pulse => (((self.phase <= self.duty) as i64) * 2 - 1) as f64,
            Waveform::Triangle => (self.phase * TAU).sin().asin(),
            Waveform::Sawtooth => (self.phase - self.phase.floor()) * 2.0 - 1.0,
        }
    }

    pub fn advance_phase(&mut self, incr: f64) {
        self.phase += self.phase_incr + incr;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }
}
