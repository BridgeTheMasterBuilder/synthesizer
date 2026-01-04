use crate::oscillator::Oscillator;

#[derive(Debug, Clone, Copy)]
pub struct Modulator {
    pub oscillator: Oscillator,
    ratio: f64,
    amount: f64,
    ratio_spectrum: u8,
    amount_spectrum: u8,
}

impl Modulator {
    pub fn new() -> Self {
        Self {
            oscillator: Oscillator::new(0.0),
            ratio: 0.0,
            amount: 0.0,
            ratio_spectrum: 16,
            amount_spectrum: 1,
        }
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.oscillator.set_freq(freq * self.ratio);
    }

    pub fn output(&mut self) -> f64 {
        self.oscillator.output()
    }

    pub fn amount(self) -> f64 {
        self.amount
    }
    pub fn set_ratio(&mut self, value: u8, carrier_freq: f64) {
        // TODO fine tune
        self.ratio = value as f64 / self.ratio_spectrum as f64;
        self.set_freq(carrier_freq);
    }

    pub fn set_amount(&mut self, value: u8) {
        // TODO fine tune
        self.amount = value as f64 / self.amount_spectrum as f64;
    }

    pub fn set_ratio_spectrum(&mut self, value: u8) {
        self.ratio_spectrum = value + 1;
    }

    pub fn set_amount_spectrum(&mut self, value: u8) {
        self.amount_spectrum = value + 1;
    }
}
