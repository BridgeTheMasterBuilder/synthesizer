use crate::SAMPLE_RATE;
use crate::envelope::Envelope;
use crate::modulator::Modulator;
use crate::oscillator::Oscillator;

#[derive(Debug, Clone)]
pub struct Voice {
    pub enabled: bool,
    pub env: Envelope,
    pub modulator1: Modulator,
    pub modulator1_env: Envelope,
    pub modulator2: Modulator,
    pub modulator2_env: Envelope,
    pub oscillator1: Oscillator,
    pub oscillator2: Oscillator,
    pub lfo: Oscillator,
    buffer: Option<i16>,
    vibrato_depth: u8,
    oscillator_balance: f64,
}

impl Voice {
    // TODO vol as f64 / u16::MAX as f64?
    // TODO vol unnecessary?
    pub fn new(freq: f64, _vol: u16) -> Self {
        Self {
            enabled: true,
            oscillator1: Oscillator::new(freq),
            oscillator2: Oscillator::new(freq),
            buffer: None,
            env: Envelope::new(0.5, 1, 1, 127, 1, false),
            modulator1_env: Envelope::new(1.0, 1, 1, 127, 1, false),
            lfo: Oscillator::new(0.0),
            modulator1: Modulator::new(),
            modulator2: Modulator::new(),
            modulator2_env: Envelope::new(1.0, 1, 1, 127, 1, false),
            vibrato_depth: 5,
            oscillator_balance: 0.5,
        }
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.oscillator1.set_freq(freq);
        self.oscillator2.set_freq(freq);
        self.modulator1.set_freq(freq);
        self.modulator2.set_freq(self.modulator1.oscillator.freq());
    }

    pub fn set_vibrato(&mut self, freq: f64) {
        self.lfo.set_freq(freq);
    }

    pub fn output(&mut self) -> i16 {
        if self.buffer.is_some() {
            return self.buffer.take().unwrap();
        }

        let sample = self.oscillator1.sample() * self.oscillator_balance;
        let sample = sample + self.oscillator2.sample() * (1.0 - self.oscillator_balance);

        let sample = sample * self.env.volume() as f64;

        let vibrato = self.lfo.output();
        let delta = (self.oscillator1.freq()
            * 2.0_f64.powf((self.vibrato_depth as f64 * self.lfo.freq()) / 1200.0))
            - self.oscillator1.freq();
        let new_freq = self.oscillator1.freq() + delta * vibrato;

        let vibrato_phase_incr = new_freq / SAMPLE_RATE as f64;

        let pre_modulation = self.modulator2.output();

        let pre_modulation_phase_incr = pre_modulation
            * self.modulator2.amount()
            * (self.modulator1.oscillator.freq() / SAMPLE_RATE as f64)
            * self.modulator2_env.normalized_volume();

        self.modulator1
            .oscillator
            .advance_phase(Some(pre_modulation_phase_incr));

        let modulation = self.modulator1.output();

        let modulator_phase_incr = modulation
            * self.modulator1.amount()
            * (self.oscillator1.freq() / SAMPLE_RATE as f64)
            * self.modulator1_env.normalized_volume();

        self.modulator1_env.adjust_volume();
        self.modulator2_env.adjust_volume();

        self.enabled = !self.env.adjust_volume();

        self.oscillator1
            .advance_phase(Some(modulator_phase_incr + vibrato_phase_incr));
        self.oscillator2
            .advance_phase(Some(modulator_phase_incr + vibrato_phase_incr));

        self.buffer.replace(sample as i16);

        sample as i16
    }

    // TODO unnecessary?
    pub fn _volume(&self) -> u16 {
        self.env.volume()
    }

    pub fn set_gain(&mut self, value: u16) {
        self.env.set_gain(value as f64 / u16::MAX as f64);
    }

    pub fn set_vibrato_depth(&mut self, depth: u8) {
        self.vibrato_depth = depth;
    }

    pub fn set_oscillator_balance(&mut self, value: u8) {
        let balance = value as f64 / 127.0;
        self.oscillator_balance = balance;
    }
}
