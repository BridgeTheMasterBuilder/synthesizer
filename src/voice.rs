use std::f64::consts::TAU;

use crate::envelope::Envelope;
use crate::hw::SAMPLE_RATE;
use crate::oscillator::{Oscillator, Waveform};

#[derive(Debug, Clone)]
pub struct Voice {
    pub enabled: bool,
    oscillator: Oscillator,
    buffer: Option<i16>,
    env: Envelope,
    lfo: Oscillator,
    modulator: Oscillator,
    modulator_ratio: f64,
    modulator_amount: f64,
}

impl Voice {
    // const RATIO: f64 = 0.0;
    // const AMOUNT: f64 = 0.0;

    pub fn new(freq: f64, vol: u16) -> Self {
        Self {
            enabled: true,
            oscillator: Oscillator::new(freq),
            buffer: None,
            env: Envelope::new(0.5, 1, 1, vol, 1),
            lfo: Oscillator::new(0.0),
            modulator: Oscillator::new(0.0),
            modulator_ratio: 0.0,
            modulator_amount: 0.0,
        }
    }

    pub fn freq(&self) -> f64 {
        self.oscillator.freq()
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.oscillator.set_freq(freq);
        self.modulator.set_freq(freq * self.modulator_ratio);
    }

    pub fn set_volume(&mut self, vol: u16) {
        self.env.set_volume(vol);
    }

    pub fn set_vibrato(&mut self, freq: f64) {
        self.lfo.set_freq(freq);
    }

    pub fn output(&mut self) -> i16 {
        if self.buffer.is_some() {
            return self.buffer.take().unwrap();
        }

        let sample = self.oscillator.sample();

        let sample = sample * self.env.volume() as f64;

        let vibrato = self.lfo.output();
        // let delta = (self.freq * 2.0_f64.powf((5.0 * self.lfo.freq()) / 1200.0)) - self.freq;
        // let new_freq = self.freq + delta * vibrato;
        let delta = (self.oscillator.freq() * 2.0_f64.powf((5.0 * self.lfo.freq()) / 1200.0))
            - self.oscillator.freq();
        let new_freq = self.oscillator.freq() + delta * vibrato;

        let vibrato_phase_incr = new_freq / SAMPLE_RATE as f64;
        let modulation = self.modulator.output();
        let delta = (self.oscillator.freq() * self.modulator_amount) - self.oscillator.freq();
        let new_freq = self.oscillator.freq() + delta * modulation;

        let modulator_phase_incr = new_freq / SAMPLE_RATE as f64;

        self.enabled = !self.env.adjust_volume();

        self.oscillator
            .advance_phase(modulator_phase_incr + vibrato_phase_incr);

        self.buffer.replace(sample as i16);

        sample as i16
    }

    pub fn volume(&self) -> u16 {
        self.env.volume()
    }

    pub fn set_modulator_ratio(&mut self, value: u8) {
        self.modulator_ratio = value as f64 / 8.0;
        self.modulator
            .set_freq(self.oscillator.freq() * self.modulator_ratio);
    }

    pub fn set_modulator_amount(&mut self, value: u8) {
        self.modulator_amount = value as f64 / 4.0;
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.oscillator.set_waveform(waveform);
    }

    pub fn set_modulator_waveform(&mut self, waveform: Waveform) {
        self.modulator.set_waveform(waveform);
    }
    pub fn set_duty(&mut self, value: f64) {
        self.oscillator.set_duty(value);
    }

    pub fn set_modulator_duty(&mut self, value: f64) {
        self.modulator.set_duty(value);
    }

    pub fn set_gain(&mut self, value: u16) {
        self.env.set_gain(value as f64 / u16::MAX as f64);
    }
}
