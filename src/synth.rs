use std::array;
use std::collections::BTreeSet;

use tables::PYTHAGOREAN;

use crate::hw::SF;
use crate::oscillator::Waveform;
use crate::tables::TABLES;
use crate::voice::Voice;

// TODO implement sustain in here
#[derive(Clone)]
pub struct Synth {
    voices: [Voice; 109],
    active_voices: BTreeSet<u8>,
    table: usize,
    last_note: u8,
    last_freq: f64,
    volume: f64,
}

// TODO Refactor with forall_voices or something similar
impl Synth {
    const VOLUME: u8 = 127;

    // TODO Magic numbers
    pub fn new() -> Self {
        Self {
            voices: array::from_fn(|_| Voice::new(0.0, 0)),
            active_voices: BTreeSet::new(),
            table: PYTHAGOREAN as usize,
            last_note: 60,
            last_freq: 264.0,
            volume: 1.0,
        }
    }

    // TODO Make an enum and perform the translation elsewhere?
    // TODO Magic numbers
    pub fn change_tuning(&mut self, note: u8) {
        match note - 60 {
            1 => self.table ^= 0b00_0010_0000,
            3 => self.table ^= 0b00_0100_0000,
            4 => self.table ^= 0b00_0000_0010,
            5 => self.table ^= 0b00_0000_1000,
            6 => self.table ^= 0b00_1000_0000,
            7 => self.table ^= 0b00_0000_0001,
            8 => self.table ^= 0b00_0001_0000,
            9 => self.table ^= 0b00_0000_0100,
            10 => self.table ^= 0b01_0000_0000,
            11 => self.table ^= 0b10_0000_0000,
            12 => self.table = 0b00_0000_0001,
            _ => {}
        }

        self.retune();
    }

    // TODO Magic numbers
    pub fn change_fundamental(&mut self, note: u8) {
        let normalized_base = (note + 12) as i8;
        let interval = normalized_base - 60;

        if let Some(freq) = Self::transform_freq(264.0, interval, &TABLES[self.table]) {
            self.last_note = normalized_base as u8;
            self.last_freq = freq;
            self.retune();
        }
        // self.log();
    }

    fn retune(&mut self) {
        if !self.active_voices.is_empty() {
            let (base_freq, current_note) = (self.last_freq, self.last_note);

            for &note in &self.active_voices {
                let oscillator = &mut self.voices[note as usize];

                let interval = note as i8 - current_note as i8;

                if let Some(freq) = Self::transform_freq(base_freq, interval, &TABLES[self.table]) {
                    oscillator.set_freq(freq);
                }
            }
        }
        // self.log();
    }

    fn transform_freq(mut freq: f64, mut midi_interval: i8, interval_table: &[f64]) -> Option<f64> {
        while midi_interval < 0 {
            midi_interval += 12;
            freq /= 2.0;
        }

        let interval = interval_table[midi_interval as usize];

        if interval == 0.0 {
            None
        } else {
            Some(freq * interval)
        }
    }

    fn play_note_with_freq_and_vol(&mut self, note: u8, freq: f64, vol: u8) {
        let vol = vol as u16;

        let voice = &mut self.voices[note as usize];
        voice.enabled = true;
        voice.set_freq(freq);
        voice.set_volume(vol);

        self.active_voices.insert(note);
    }

    pub fn set_vibrato(&mut self, freq: f64) {
        for voice in &mut self.voices {
            voice.set_vibrato(freq);
        }
    }

    pub fn set_volume(&mut self, vol: u8) {
        self.volume = vol as f64 / 127.0;
    }

    pub fn play(&mut self, note: u8) {
        let note = note as i8;
        let last_note = self.last_note as i8;
        let interval = note - last_note;

        if let Some(freq) = Self::transform_freq(self.last_freq, interval, &TABLES[self.table]) {
            self.play_note_with_freq_and_vol(note as u8, freq, Self::VOLUME);
        }
        // self.log();
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
        self.voices[note as usize].set_volume(0);

        self.active_voices.remove(&note);
    }

    pub fn set_modulator_ratio(&mut self, value: u8) {
        self.voices
            .iter_mut()
            .for_each(|oscillator| oscillator.set_modulator_ratio(value));
    }
    pub fn set_modulator_amount(&mut self, value: u8) {
        self.voices
            .iter_mut()
            .for_each(|oscillator| oscillator.set_modulator_amount(value));
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_waveform(waveform));
    }

    pub fn set_modulator_waveform(&mut self, waveform: Waveform) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_modulator_waveform(waveform));
    }

    pub fn set_duty(&mut self, value: f64) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_duty(value));
    }

    pub fn set_modulator_duty(&mut self, value: f64) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_modulator_duty(value));
    }

    // TODO make the envelope public so you don't need these wrappers?
    pub fn set_gain(&mut self, value: u16) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_gain(value));
    }

    pub fn set_attack(&mut self, value: u8) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_attack(value));
    }

    pub fn set_decay(&mut self, value: u8) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_decay(value));
    }

    pub fn set_sustain(&mut self, value: u8) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_sustain(value));
    }

    pub fn set_release(&mut self, value: u8) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_release(value));
    }
}

impl Iterator for Synth {
    type Item = SF;
    fn next(&mut self) -> Option<Self::Item> {
        let sum: i16 = self
            .voices
            .iter_mut()
            .filter(|voice| voice.enabled)
            .fold(0, |sum, sample| sum.saturating_add(sample.output()));

        let sample = (sum as f64 * self.volume) as i16;

        Some(sample)
    }
}
