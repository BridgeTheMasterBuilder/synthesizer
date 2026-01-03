use crate::hw::IO;
use ::alsa::seq::EventType;
use alsa::seq::{EvCtrl, EvNote};
use anyhow::Result;
use bpaf::Bpaf;
use std::fs::File;
use std::io::Read;
use synth::oscillator::Waveform;
use synth::{Mode, Synth, SynthSetting};

pub mod hw;
mod midi;
mod pcm;
mod scala;

#[derive(Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short('p'), long, argument)]
    pub main_port: i32,
    #[bpaf(short('a'), long, argument)]
    pub aux_port: i32,
    #[bpaf(short('e'), long, argument)]
    pub expr_port: i32,
    #[bpaf(short('m'), long, argument)]
    pub mixer_port: i32,
    #[bpaf(short('f'), long, argument)]
    pub pedal_port: i32,
    #[bpaf(short('c'), long, argument)]
    pub card: String,
    #[bpaf(short('s'), long, argument)]
    pub settings_filename: Option<String>,
    #[bpaf(short('t'), long, argument)]
    pub tuning_preset_filename: Option<String>,
}

const C0: u8 = 12;
const H0: u8 = 23;
const C1: u8 = 24;
const CIS1: u8 = 25;
const D1: u8 = 26;
const H1: u8 = 35;
const C2: u8 = 36;
const C3: u8 = 48;
const H3: u8 = 59;
const C4: u8 = 60;
const C5: u8 = 72;
const CIS5: u8 = 73;
const C6: u8 = 84;
const MIXER: u8 = 0;
const MANUAL: u8 = 1;
const CONTROL: u8 = 2;
const EXPRESSION: u8 = 3;
const PEDALS: u8 = 4;
const VOLUME: u32 = 21;
const VIBRATO: u32 = 22;
const DAMPER: u32 = 64;
const WAVEFORM: u32 = 16;
const DUTY: u32 = 20;
const ATTACK: u32 = 46;
const DECAY: u32 = 50;
const SUSTAIN: u32 = 54;
const RELEASE: u32 = 58;
const MODULATOR1_WAVEFORM: u32 = 17;
const MODULATOR1_DUTY: u32 = 21;
const MODULATOR1_RATIO: u32 = 25;
const MODULATOR1_AMOUNT: u32 = 29;
const MODULATOR1_ATTACK: u32 = 47;
const MODULATOR1_DECAY: u32 = 51;
const MODULATOR1_SUSTAIN: u32 = 55;
const MODULATOR1_RELEASE: u32 = 59;
const MODULATOR2_WAVEFORM: u32 = 18;
const MODULATOR2_DUTY: u32 = 22;
const MODULATOR2_RATIO: u32 = 26;
const MODULATOR2_AMOUNT: u32 = 30;
const MODULATOR2_ATTACK: u32 = 48;
const MODULATOR2_DECAY: u32 = 52;
const MODULATOR2_SUSTAIN: u32 = 56;
const MODULATOR2_RELEASE: u32 = 60;
const TIMBRE_BANK_1: u8 = 1;
const TIMBRE_BANK_2: u8 = 4;
const TIMBRE_BANK_3: u8 = 7;
const TIMBRE_BANK_4: u8 = 10;
const TIMBRE_BANK_5: u8 = 13;
const TIMBRE_BANK_6: u8 = 16;
const TIMBRE_BANK_7: u8 = 19;
const TIMBRE_BANK_8: u8 = 22;
const TUNING_BANK_1: u8 = 3;
const TUNING_BANK_2: u8 = 6;
const TUNING_BANK_3: u8 = 9;
const TUNING_BANK_4: u8 = 12;
const TUNING_BANK_5: u8 = 15;
const TUNING_BANK_6: u8 = 18;
const TUNING_BANK_7: u8 = 21;
const TUNING_BANK_8: u8 = 24;
const SAVE_TIMBRE_PRESETS: u8 = 27;

fn parse_settings_file(settings_filename: &str) -> [SynthSetting; 8] {
    let mut settings: [SynthSetting; 8] = [SynthSetting::default(); 8];

    if let Ok(mut settings_file) = File::open(settings_filename) {
        let mut data = Vec::new();

        if settings_file.read_to_end(&mut data).is_ok() {
            settings = serde_json::from_slice(&data)
                .map_err(|_| eprintln!("WARNING: "))
                .unwrap_or(settings);
        } else {
            eprintln!("WARNING: ");
        }
    }

    settings
}

fn write_settings_to_file(settings_filename: &str, settings: [SynthSetting; 8]) {
    if let Ok(settings_file) = File::create(settings_filename) {
        if serde_json::to_writer(settings_file, &settings).is_ok() {
        } else {
            eprintln!("WARNING: ");
        }
    } else {
        eprintln!("WARNING: ");
    }
}

fn parse_tuning_preset_file(tuning_preset_filename: &str) -> [[f64; 128]; 8] {
    [scala::parse_scala_file(tuning_preset_filename); 8]
}

pub fn run(options: Options) -> Result<()> {
    let main_port = options.main_port;
    let aux_port = options.aux_port;
    let expr_port = options.expr_port;
    let mixer_port = options.mixer_port;
    let pedal_port = options.pedal_port;
    let card = options.card;
    let settings_filename = options.settings_filename;
    let tuning_preset_filename = options.tuning_preset_filename;

    // TODO ugly hacks
    let (settings, settings_filename) = settings_filename.map_or(
        ([SynthSetting::default(); 8], "test".to_string()),
        |filename| (parse_settings_file(filename.as_str()), filename.clone()),
    );
    let tuning_preset = tuning_preset_filename.map_or(None, |filename| {
        Some(parse_tuning_preset_file(filename.as_str()))
    });

    let mut io = IO::new(
        main_port, aux_port, expr_port, mixer_port, pedal_port, &card,
    )?;
    let mut synth = Synth::new(settings, tuning_preset);
    // let mut control = Synth::new();
    // let mut pedals = Synth::new();

    synth.change_timbre_bank(0);

    loop {
        io.write(&mut synth)?;

        if let Some(event) = io.read()? {
            match event.get_type() {
                EventType::Noteoff => {
                    if let Some(EvNote { channel, note, .. }) = event.get_data() {
                        match synth.mode {
                            Mode::Fixed => match channel {
                                // TODO
                                // CONTROL => control.silence(note),
                                // MANUAL => synth.silence(note),
                                // PEDALS => pedals.silence(note),
                                // _ => {}
                                _ => synth.silence(note),
                            },
                            Mode::Dynamic => {
                                match channel {
                                    CONTROL => match note {
                                        // C3..=H3 => {
                                        //     synth.change_fundamental(note);
                                        // }
                                        C4..=C5 => {
                                            synth.change_tuning(note);
                                        }
                                        _ => (),
                                    },
                                    MANUAL => synth.silence(note),
                                    PEDALS => match note {
                                        // C1..=H1 => {
                                        //     synth.change_fundamental(note);
                                        // }
                                        C1..=H1 => {
                                            synth.change_tuning(note + 36);
                                        }
                                        C2..=C5 => {
                                            synth.silence(note - 24);
                                        }
                                        _ => (),
                                    },
                                    // _ => unreachable!(),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                EventType::Noteon => {
                    if let Some(EvNote { channel, note, .. }) = event.get_data() {
                        match synth.mode {
                            Mode::Fixed => match channel {
                                MIXER => match note {
                                    CIS1 => {
                                        synth.toggle_modulator1_env_repeat();
                                    }
                                    D1 => {
                                        synth.toggle_modulator2_env_repeat();
                                    }
                                    TIMBRE_BANK_1 => synth.change_timbre_bank(0),
                                    TIMBRE_BANK_2 => synth.change_timbre_bank(1),
                                    TIMBRE_BANK_3 => synth.change_timbre_bank(2),
                                    TIMBRE_BANK_4 => synth.change_timbre_bank(3),
                                    TIMBRE_BANK_5 => synth.change_timbre_bank(4),
                                    TIMBRE_BANK_6 => synth.change_timbre_bank(5),
                                    TIMBRE_BANK_7 => synth.change_timbre_bank(6),
                                    TIMBRE_BANK_8 => synth.change_timbre_bank(7),
                                    TUNING_BANK_1 => synth.change_tuning_bank(0),
                                    TUNING_BANK_2 => synth.change_tuning_bank(1),
                                    TUNING_BANK_3 => synth.change_tuning_bank(2),
                                    TUNING_BANK_4 => synth.change_tuning_bank(3),
                                    TUNING_BANK_5 => synth.change_tuning_bank(4),
                                    TUNING_BANK_6 => synth.change_tuning_bank(5),
                                    TUNING_BANK_7 => synth.change_tuning_bank(6),
                                    TUNING_BANK_8 => synth.change_tuning_bank(7),
                                    SAVE_TIMBRE_PRESETS => write_settings_to_file(
                                        settings_filename.as_str(),
                                        synth.timbre_presets,
                                    ),
                                    _ => {}
                                },
                                // TODO
                                // CONTROL => control.play(note),
                                // MANUAL => synth.play(note),
                                // PEDALS => pedals.play(note),
                                // _ => {}
                                _ => synth.play_fixed(note),
                            },
                            Mode::Dynamic => {
                                match channel {
                                    CONTROL => match note {
                                        C3..=H3 => {
                                            synth.change_fundamental(note);
                                        }
                                        C4..=C5 => {
                                            synth.change_tuning(note);
                                        }
                                        CIS5..=C6 => {
                                            synth.change_tuning(note - 12);
                                        }
                                        _ => (),
                                    },
                                    MANUAL => {
                                        synth.play(note);
                                    }
                                    PEDALS => match note {
                                        C0..=H0 => {
                                            synth.change_fundamental(note + 36);
                                        }
                                        C1..=H1 => {
                                            synth.change_tuning(note + 36);
                                        }
                                        C2..=C5 => {
                                            synth.play(note - 24);
                                        }
                                        _ => (),
                                    },
                                    MIXER => match note {
                                        CIS1 => {
                                            synth.toggle_modulator1_env_repeat();
                                        }
                                        D1 => {
                                            synth.toggle_modulator2_env_repeat();
                                        }
                                        TIMBRE_BANK_1 => synth.change_timbre_bank(0),
                                        TIMBRE_BANK_2 => synth.change_timbre_bank(1),
                                        TIMBRE_BANK_3 => synth.change_timbre_bank(2),
                                        TIMBRE_BANK_4 => synth.change_timbre_bank(3),
                                        TIMBRE_BANK_5 => synth.change_timbre_bank(4),
                                        TIMBRE_BANK_6 => synth.change_timbre_bank(5),
                                        TIMBRE_BANK_7 => synth.change_timbre_bank(6),
                                        TIMBRE_BANK_8 => synth.change_timbre_bank(7),
                                        TUNING_BANK_1 => synth.change_tuning_bank(0),
                                        TUNING_BANK_2 => synth.change_tuning_bank(1),
                                        TUNING_BANK_3 => synth.change_tuning_bank(2),
                                        TUNING_BANK_4 => synth.change_tuning_bank(3),
                                        TUNING_BANK_5 => synth.change_tuning_bank(4),
                                        TUNING_BANK_6 => synth.change_tuning_bank(5),
                                        TUNING_BANK_7 => synth.change_tuning_bank(6),
                                        TUNING_BANK_8 => synth.change_tuning_bank(7),
                                        SAVE_TIMBRE_PRESETS => write_settings_to_file(
                                            settings_filename.as_str(),
                                            synth.timbre_presets,
                                        ),
                                        _ => {}
                                    },
                                    // _ => unreachable!(),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                EventType::Controller => match event.get_data() {
                    Some(EvCtrl {
                        param,
                        value,
                        channel,
                        ..
                    }) => match channel {
                        MANUAL => match param {
                            DAMPER => {
                                if value == 127 {
                                    synth.enable_sustain()
                                } else {
                                    synth.disable_sustain()
                                }
                            }
                            _ => {}
                        },
                        EXPRESSION => match param {
                            VOLUME => synth.set_gain(value as u16 * 512),
                            VIBRATO => synth.set_vibrato((value / 14) as f64),
                            _ => {}
                        },
                        MIXER => match param {
                            WAVEFORM => {
                                let waveform = match value / (128 / 4) {
                                    0 => Waveform::Sine,
                                    1 => Waveform::Pulse,
                                    2 => Waveform::Triangle,
                                    3 => Waveform::Sawtooth,
                                    _ => unreachable!(),
                                };
                                synth.set_waveform(waveform);
                            }
                            MODULATOR1_WAVEFORM => {
                                let waveform = match value / (128 / 4) {
                                    0 => Waveform::Sine,
                                    1 => Waveform::Pulse,
                                    2 => Waveform::Triangle,
                                    3 => Waveform::Sawtooth,
                                    _ => unreachable!(),
                                };
                                synth.set_modulator1_waveform(waveform);
                            }
                            MODULATOR2_WAVEFORM => {
                                let waveform = match value / (128 / 4) {
                                    0 => Waveform::Sine,
                                    1 => Waveform::Pulse,
                                    2 => Waveform::Triangle,
                                    3 => Waveform::Sawtooth,
                                    _ => unreachable!(),
                                };
                                synth.set_modulator2_waveform(waveform);
                            }
                            DUTY => synth.set_duty(value as u8),
                            MODULATOR1_RATIO => synth.set_modulator1_ratio(value as u8),
                            MODULATOR1_AMOUNT => synth.set_modulator1_amount(value as u8),
                            MODULATOR1_DUTY => synth.set_modulator1_duty(value as u8),
                            MODULATOR1_ATTACK => synth.set_modulator1_attack(value as u8),
                            MODULATOR1_DECAY => synth.set_modulator1_decay(value as u8),
                            MODULATOR1_SUSTAIN => synth.set_modulator1_sustain(value as u8),
                            MODULATOR1_RELEASE => synth.set_modulator1_release(value as u8),
                            MODULATOR2_RATIO => synth.set_modulator2_ratio(value as u8),
                            MODULATOR2_AMOUNT => synth.set_modulator2_amount(value as u8),
                            MODULATOR2_DUTY => synth.set_modulator2_duty(value as u8),
                            MODULATOR2_ATTACK => synth.set_modulator2_attack(value as u8),
                            MODULATOR2_DECAY => synth.set_modulator2_decay(value as u8),
                            MODULATOR2_SUSTAIN => synth.set_modulator2_sustain(value as u8),
                            MODULATOR2_RELEASE => synth.set_modulator2_release(value as u8),
                            ATTACK => synth.set_attack(value as u8),
                            DECAY => synth.set_decay(value as u8),
                            SUSTAIN => synth.set_sustain(value as u8),
                            RELEASE => synth.set_release(value as u8),
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        io.poll()?;
    }
}
