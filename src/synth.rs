use std::collections::{BTreeMap, BTreeSet};

use tables::PYTHAGOREAN;

use crate::hw::SF;
use crate::oscillator::TriangleOscillator;
use crate::synth::Mode::{Anchored, Fluid};
use crate::tables::TABLES;

enum Mode {
    Anchored,
    Fluid,
}

pub struct Synth {
    voices: BTreeMap<u8, TriangleOscillator>,
    active_voices: BTreeSet<u8>,
    table: usize,
    last_note: u8,
    last_freq: f64,
    mode: Mode,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            voices: BTreeMap::new(),
            active_voices: BTreeSet::new(),
            table: PYTHAGOREAN as usize,
            last_note: 60,
            last_freq: 264.0,
            mode: Anchored,
        }
    }

    pub fn change_tuning(&mut self, note: u8) {
        match note - 33 {
            1 => self.table ^= 0b10_0000,
            4 => self.table ^= 0b00_0010,
            6 => self.table ^= 0b00_1000,
            7 => self.table ^= 0b00_0001,
            8 => self.table ^= 0b01_0000,
            10 => self.table ^= 0b00_0100,
            12 => self.table = 0b00_0001,
            _ => {}
        }

        self.retune();
    }

    pub fn change_fundamental(&mut self, note: u8) {
        let normalized_base = note + 36;
        let interval = (normalized_base - 60) as i8;

        if let Some(freq) = Self::transform_freq(264.0, interval, &TABLES[PYTHAGOREAN as usize]) {
            self.last_note = normalized_base;
            self.last_freq = freq;
            self.retune();
        }
        // self.log();
    }

    fn retune(&mut self) {
        if !self.active_voices.is_empty() {
            let (base_freq, current_note) = match self.mode {
                Anchored => (self.last_freq, self.last_note),
                Fluid => {
                    let current_note = self.active_voices.first().unwrap();
                    let base_freq = self.voices.get(current_note).unwrap().freq();

                    (base_freq, *current_note)
                }
            };

            for note in &self.active_voices {
                let oscillator = self.voices.get_mut(note).unwrap();

                let interval = (note - current_note) as i8;

                if let Some(freq) = Self::transform_freq(base_freq, interval, &TABLES[self.table]) {
                    oscillator.set_freq(freq);
                }
            }
        }
        // self.log();
    }

    fn transform_freq(freq: f64, midi_interval: i8, interval_table: &[f64]) -> Option<f64> {
        let sign = midi_interval.signum();

        let interval = interval_table[midi_interval.unsigned_abs() as usize];

        if interval == 0.0 {
            None
        } else if sign >= 0 {
            Some(freq * interval)
        } else {
            Some(freq / interval)
        }
    }

    fn play_note_with_freq_and_vol(&mut self, note: u8, freq: f64, vol: u8) {
        let vol = vol as i16 * 8;

        if let Some(oscillator) = self.voices.get_mut(&note) {
            oscillator.set_freq(freq);
            oscillator.set_vol(vol);
        } else {
            let oscillator = TriangleOscillator::new(freq, vol);

            self.voices.insert(note, oscillator);
        }

        self.active_voices.insert(note);
    }

    pub fn set_vibrato(&mut self, freq: f64) {
        for (_, voice) in &mut self.voices {
            voice.set_vibrato(freq);
        }
    }

    pub fn play(&mut self, note: u8, velocity: u8) {
        match self.mode {
            Anchored => {
                let interval = (note - self.last_note) as i8;

                if let Some(freq) =
                    Self::transform_freq(self.last_freq, interval, &TABLES[self.table])
                {
                    self.play_note_with_freq_and_vol(note, freq, velocity);
                }
            }
            Fluid => {
                let lowest_voice = self.active_voices.first();

                let (base_freq, current_note) =
                    lowest_voice.map_or((self.last_freq, self.last_note), |note| {
                        let voice = self.voices.get(note).unwrap();

                        (voice.freq(), *note)
                    });

                let interval = (note - current_note) as i8;

                if let Some(freq) = Self::transform_freq(base_freq, interval, &TABLES[self.table]) {
                    self.play_note_with_freq_and_vol(note, freq, velocity);

                    self.last_note = note;
                    self.last_freq = freq;
                }
            }
        }

        self.log();
    }

    // fn log(&self) {
    //     println!("Fundamental: {} - {}", self.last_note, self.last_freq);
    //     println!("Currently active voices:");
    //     for (note, osc) in &self.voices {
    //         if !osc.enabled {
    //             continue;
    //         }
    //
    //         println!("{note}: {}", osc.freq());
    //     }
    // }

    pub fn silence(&mut self, note: u8) {
        if let Some(voice) = self.voices.get_mut(&note) {
            voice.set_vol(0);
        };

        self.active_voices.remove(&note);
    }
}

impl Iterator for Synth {
    type Item = SF;
    fn next(&mut self) -> Option<Self::Item> {
        let sum = self
            .voices
            .values_mut()
            .filter(|voice| voice.enabled)
            .fold(0, |sum, sample| sum + sample.output());

        let sample = sum;

        Some(sample)
    }
}
