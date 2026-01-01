use std::array;
use std::collections::BTreeSet;

use crate::oscillator::Waveform;
use crate::tables::TABLES;
use crate::voice::Voice;
use tables::PYTHAGOREAN;
mod envelope;
mod modulator;
pub mod oscillator;
mod tables;
mod voice;

pub const SAMPLE_RATE: u32 = 44100;

pub type SF = i16;

#[derive(Clone)]
pub enum Mode {
    Fixed,
    Dynamic,
}

#[derive(Clone, Copy)]
pub struct SynthSetting {
    oscillator_waveform: Waveform,
    oscillator_duty: u8,
    oscillator_attack: u8,
    oscillator_decay: u8,
    oscillator_sustain: u8,
    oscillator_release: u8,
    modulator1_waveform: Waveform,
    modulator1_duty: u8,
    modulator1_ratio: u8,
    modulator1_amount: u8,
    modulator1_attack: u8,
    modulator1_decay: u8,
    modulator1_sustain: u8,
    modulator1_release: u8,
    modulator1_env_repeat: bool,
    modulator2_waveform: Waveform,
    modulator2_duty: u8,
    modulator2_ratio: u8,
    modulator2_amount: u8,
    modulator2_attack: u8,
    modulator2_decay: u8,
    modulator2_sustain: u8,
    modulator2_release: u8,
    modulator2_env_repeat: bool,
}

impl Default for SynthSetting {
    fn default() -> Self {
        Self {
            oscillator_waveform: Waveform::Sine,
            oscillator_duty: 0,
            oscillator_attack: 0,
            oscillator_decay: 0,
            oscillator_sustain: 127,
            oscillator_release: 0,
            modulator1_waveform: Waveform::Sine,
            modulator1_duty: 0,
            modulator1_ratio: 0,
            modulator1_amount: 0,
            modulator1_attack: 0,
            modulator1_decay: 0,
            modulator1_sustain: 127,
            modulator1_release: 0,
            modulator1_env_repeat: false,
            modulator2_waveform: Waveform::Sine,
            modulator2_duty: 0,
            modulator2_ratio: 0,
            modulator2_amount: 0,
            modulator2_attack: 0,
            modulator2_decay: 0,
            modulator2_sustain: 127,
            modulator2_release: 0,
            modulator2_env_repeat: false,
        }
    }
}

#[derive(Clone)]
pub struct Synth {
    pub mode: Mode,
    voices: [Voice; 109],
    active_voices: BTreeSet<u8>,
    table: usize,
    last_note: u8,
    last_freq: f64,
    volume: f64,
    sustain: bool,
    sustained_voices: BTreeSet<u8>,
    timbre_presets: [SynthSetting; 8],
    timbre_index: usize,
    tuning_presets: Option<[[f64; 128]; 8]>,
    tuning_index: usize,
}

// TODO Refactor with forall_voices or something similar
impl Synth {
    const VOLUME: u8 = 127;

    // TODO Magic numbers
    // TODO active_tuning to ignore tuning note offs when fixing
    pub fn new(timbre_presets: [SynthSetting; 8], tuning_presets: Option<[[f64; 128]; 8]>) -> Self {
        let mode = if tuning_presets.is_some() {
            Mode::Fixed
        } else {
            Mode::Dynamic
        };

        Self {
            voices: array::from_fn(|_| Voice::new(0.0, 0)),
            active_voices: BTreeSet::new(),
            table: PYTHAGOREAN as usize,
            last_note: 60,
            last_freq: 264.0,
            volume: 1.0,
            sustain: false,
            sustained_voices: BTreeSet::new(),
            mode,
            timbre_presets,
            timbre_index: 0,
            tuning_presets,
            tuning_index: 0,
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
        // TODO why is this check necessary?
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

    fn retune_fixed(&mut self) {
        for &note in &self.active_voices {
            let oscillator = &mut self.voices[note as usize];

            let freq = self.tuning_presets.unwrap()[self.tuning_index][note as usize];

            oscillator.set_freq(freq);
        }
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
        voice.env.set_volume(vol);
        voice.modulator1_env.set_volume(255);
        voice.modulator2_env.set_volume(255);

        self.active_voices.insert(note);

        if self.sustain {
            self.sustained_voices.insert(note);
        }
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
    pub fn play_fixed(&mut self, note: u8) {
        // TODO unwrap_unchecked?
        let freq = self.tuning_presets.unwrap()[self.tuning_index][note as usize];

        self.play_note_with_freq_and_vol(note, freq, Self::VOLUME);
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
        self.active_voices.remove(&note);

        if !self.sustained_voices.contains(&note) {
            self.voices[note as usize].env.set_volume(0);
        }
    }

    pub fn set_modulator1_ratio(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator1_ratio = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1.set_ratio(value, voice.oscillator.freq()));
    }
    pub fn set_modulator1_amount(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator1_amount = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1.set_amount(value));
    }

    pub fn set_modulator2_ratio(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator2_ratio = value;

        self.voices.iter_mut().for_each(|voice| {
            voice
                .modulator2
                .set_ratio(value, voice.modulator1.oscillator.freq())
        });
    }
    pub fn set_modulator2_amount(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator2_amount = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2.set_amount(value));
    }
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.timbre_presets[self.timbre_index].oscillator_waveform = waveform;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.oscillator.set_waveform(waveform));
    }

    pub fn set_modulator1_waveform(&mut self, waveform: Waveform) {
        self.timbre_presets[self.timbre_index].modulator1_waveform = waveform;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1.oscillator.set_waveform(waveform));
    }

    pub fn set_modulator2_waveform(&mut self, waveform: Waveform) {
        self.timbre_presets[self.timbre_index].modulator2_waveform = waveform;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2.oscillator.set_waveform(waveform));
    }
    pub fn set_duty(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].oscillator_duty = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.oscillator.set_duty(value));
    }

    pub fn set_modulator1_duty(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator1_duty = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1.oscillator.set_duty(value));
    }
    pub fn set_modulator2_duty(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator2_duty = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2.oscillator.set_duty(value));
    }

    pub fn set_gain(&mut self, value: u16) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.set_gain(value));
    }

    pub fn set_attack(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].oscillator_attack = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.env.set_attack(value));
    }

    pub fn set_decay(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].oscillator_decay = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.env.set_decay(value));
    }

    pub fn set_sustain(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].oscillator_sustain = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.env.set_sustain(value));
    }

    pub fn set_release(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].oscillator_release = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.env.set_release(value));
    }

    pub fn enable_sustain(&mut self) {
        self.sustain = true;

        for &note in self.active_voices.iter() {
            self.sustained_voices.insert(note);
        }
    }

    pub fn disable_sustain(&mut self) {
        self.sustain = false;

        for note in self.sustained_voices.iter() {
            if !self.active_voices.contains(&note) {
                self.voices[*note as usize].env.set_volume(0);
            }
        }

        self.sustained_voices.clear();
    }
    pub fn set_modulator1_attack(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator1_attack = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1_env.set_attack(value));
    }

    pub fn set_modulator1_decay(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator1_decay = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1_env.set_decay(value));
    }

    pub fn set_modulator1_sustain(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator1_sustain = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1_env.set_sustain(value));
    }

    pub fn set_modulator1_release(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator1_release = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1_env.set_release(value));
    }
    pub fn set_modulator2_attack(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator2_attack = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2_env.set_attack(value));
    }

    pub fn set_modulator2_decay(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator2_decay = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2_env.set_decay(value));
    }

    pub fn set_modulator2_sustain(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator2_sustain = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2_env.set_sustain(value));
    }

    pub fn set_modulator2_release(&mut self, value: u8) {
        self.timbre_presets[self.timbre_index].modulator2_release = value;

        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2_env.set_release(value));
    }

    pub fn toggle_modulator1_env_repeat(&mut self) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator1_env.toggle_repeat());
    }
    pub fn toggle_modulator2_env_repeat(&mut self) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.modulator2_env.toggle_repeat());
    }

    // TODO reset envelopes?
    pub fn change_timbre_bank(&mut self, index: usize) {
        self.timbre_index = index;

        let settings = self.timbre_presets[self.timbre_index];

        self.set_waveform(settings.oscillator_waveform);
        self.set_duty(settings.oscillator_duty);
        self.set_attack(settings.oscillator_attack);
        self.set_decay(settings.oscillator_decay);
        self.set_sustain(settings.oscillator_sustain);
        self.set_release(settings.oscillator_release);
        self.set_modulator1_waveform(settings.modulator1_waveform);
        self.set_modulator1_duty(settings.modulator1_duty);
        self.set_modulator1_ratio(settings.modulator1_ratio);
        self.set_modulator1_amount(settings.modulator1_amount);
        self.set_modulator1_attack(settings.modulator1_attack);
        self.set_modulator1_decay(settings.modulator1_decay);
        self.set_modulator1_sustain(settings.modulator1_sustain);
        self.set_modulator1_release(settings.modulator1_release);
        // TODO set repeat
        self.toggle_modulator1_env_repeat();
        self.set_modulator2_waveform(settings.modulator2_waveform);
        self.set_modulator2_duty(settings.modulator2_duty);
        self.set_modulator2_ratio(settings.modulator2_ratio);
        self.set_modulator2_amount(settings.modulator2_amount);
        self.set_modulator2_attack(settings.modulator2_attack);
        self.set_modulator2_decay(settings.modulator2_decay);
        self.set_modulator2_sustain(settings.modulator2_sustain);
        self.set_modulator2_release(settings.modulator2_release);
        // TODO set repeat
        self.toggle_modulator2_env_repeat();
    }

    pub fn change_tuning_bank(&mut self, index: usize) {
        self.tuning_index = index;

        self.retune_fixed();
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
