use std::f64::consts::TAU;

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
    modulator: Lfo,
    modulator_ratio: f64,
    modulator_amount: f64,
}

impl Oscillator {
    // const RATIO: f64 = 0.0;
    // const AMOUNT: f64 = 0.0;

    pub fn new(freq: f64, vol: u16) -> Self {
        Self {
            enabled: true,
            freq,
            phase: 0.0,
            phase_incr: freq / SAMPLE_RATE as f64,
            buffer: None,
            env: Envelope::new(vol),
            lfo: Lfo::new(0.0),
            modulator: Lfo::new(0.0),
            modulator_ratio: 0.0,
            modulator_amount: 0.0,
        }
    }

    pub fn freq(&self) -> f64 {
        self.freq
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
        self.phase_incr = freq / SAMPLE_RATE as f64;
        self.modulator.set_freq(freq * self.modulator_ratio);
    }

    pub fn set_volume(&mut self, vol: u16) {
        self.env.set_volume(vol);
    }

    pub fn set_vibrato(&mut self, freq: f64) {
        self.lfo.set_freq(freq);
    }

    fn sample_sawtooth(phase: f64) -> f64 {
        phase - phase.floor()
    }
    fn sample_sine(phase: f64) -> f64 {
        (phase * TAU).sin()
    }
    fn sample_triangle(phase: f64) -> f64 {
        (phase * TAU).sin().asin()
    }

    pub fn output(&mut self) -> i16 {
        if self.buffer.is_some() {
            return self.buffer.take().unwrap();
        }

        // let sample = Self::sample_triangle(self.phase);
        // let sample = Self::sample_sawtooth(self.phase);
        let sample = Self::sample_sine(self.phase);
        // let sample = sample * (self.env.volume() * 2.0) / PI;
        let sample = sample * self.env.volume() as f64;
        // let sample = sample * (self.env.volume() );

        let vibrato = self.lfo.output();
        // let delta = (self.freq * 2.0_f64.powf((5.0 * self.lfo.freq()) / 1200.0)) - self.freq;
        // let new_freq = self.freq + delta * vibrato;
        let delta = (self.freq * 2.0_f64.powf((5.0 * self.lfo.freq()) / 1200.0)) - self.freq;
        let new_freq = self.freq + delta * vibrato;

        let vibrato_phase_incr = new_freq / SAMPLE_RATE as f64;
        // let vibrato_phase_incr = 0.0;
        let modulation = self.modulator.output();
        let delta = (self.freq * self.modulator_amount) - self.freq;
        let new_freq = self.freq + delta * modulation;
        // dbg!(self.modulator_amount, self.modulator_ratio);

        let modulator_phase_incr = new_freq / SAMPLE_RATE as f64;

        self.enabled = !self.env.adjust_volume();

        self.phase += self.phase_incr + modulator_phase_incr + vibrato_phase_incr;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        self.buffer.replace(sample as i16);

        sample as i16
    }

    pub fn volume(&self) -> u16 {
        self.env.volume()
    }

    pub fn set_modulator_ratio(&mut self, value: u8) {
        self.modulator_ratio = value as f64 / 8.0;
        self.modulator.set_freq(self.freq * self.modulator_ratio);
    }

    pub fn set_modulator_amount(&mut self, value: u8) {
        self.modulator_amount = value as f64 / 4.0;
    }
}
