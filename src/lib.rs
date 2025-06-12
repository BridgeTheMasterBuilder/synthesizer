use ::alsa::seq::EventType;
use alsa::seq::{EvCtrl, EvNote};
use anyhow::Result;
use bpaf::Bpaf;

use crate::hw::IO;
use crate::oscillator::Waveform;
use crate::synth::Synth;

mod envelope;
pub mod file;
pub mod hw;
mod midi;
mod oscillator;
mod pcm;
pub mod synth;
mod tables;
mod voice;

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
    #[bpaf(short('c'), long, argument)]
    pub card: String,
}

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
const VOLUME: u32 = 21;
const VIBRATO: u32 = 22;
const DAMPER: u32 = 64;
const WAVEFORM: u32 = 16;
const MODULATOR_WAVEFORM: u32 = 17;
const DUTY: u32 = 20;
const MODULATOR_RATIO: u32 = 25;
const MODULATOR_AMOUNT: u32 = 29;
const MODULATOR_DUTY: u32 = 21;
const ATTACK: u32 = 46;
const DECAY: u32 = 50;
const SUSTAIN: u32 = 54;
const RELEASE: u32 = 58;

pub fn run(options: Options) -> Result<()> {
    let main_port = options.main_port;
    let aux_port = options.aux_port;
    let expr_port = options.expr_port;
    let mixer_port = options.mixer_port;
    let card = options.card;

    let mut io = IO::new(main_port, aux_port, expr_port, mixer_port, &card)?;
    let mut synth = Synth::new();

    // let mut collecting = false;
    let mut temporary_fundamental = C3;
    let mut current_fundamental = C3;
    // let mut sustain = false;

    // let mut sustained_notes = BTreeSet::new();
    let mut active_control_notes = 0;
    let mut ignore_note_off = false;
    let mut config_mode = false;

    loop {
        io.write(&mut synth)?;

        if let Some(event) = io.read()? {
            match event.get_type() {
                EventType::Noteoff => {
                    if let Some(EvNote { channel, note, .. }) = event.get_data() {
                        match channel {
                            CONTROL => match note {
                                // TODO magic numbers
                                // 48..=59 => collecting = false,
                                // 60..=72 if !collecting => {
                                C3..=H3 => {
                                    // if !ignore_note_off {
                                    synth.change_fundamental(current_fundamental);
                                    // }

                                    // active_control_notes -= 1;

                                    // if active_control_notes == 0 {
                                    //     ignore_note_off = false;
                                    // }
                                }
                                C4..=C5 => {
                                    // if !ignore_note_off {
                                    synth.change_tuning(note);
                                    // }

                                    // active_control_notes -= 1;
                                    //
                                    // if active_control_notes == 0 {
                                    //     ignore_note_off = false;
                                    // }
                                }
                                _ => (),
                            },
                            // 1 if sustain => {
                            //     sustained_notes.insert(note);
                            // }
                            MANUAL => synth.silence(note),
                            _ => unreachable!(),
                        }
                    }
                }
                EventType::Noteon => {
                    if let Some(EvNote { channel, note, .. }) = event.get_data() {
                        match channel {
                            CONTROL => match note {
                                C3..=H3 => {
                                    // collecting = true;
                                    // active_control_notes += 1;

                                    // temporary_fundamental = note;
                                    current_fundamental = note;

                                    // synth.change_fundamental(temporary_fundamental);
                                    synth.change_fundamental(current_fundamental);
                                }
                                C4..=C5 => {
                                    // active_control_notes += 1;
                                    synth.change_tuning(note);
                                }
                                CIS5..=C6 => {
                                    // active_control_notes += 1;
                                    synth.change_tuning(note - 12);
                                }
                                _ => (),
                            },
                            MANUAL => {
                                // sustained_notes.remove(&note);
                                synth.play(note);
                            }
                            _ => unreachable!(),
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
                        EXPRESSION => match param {
                            VOLUME => synth.set_gain(value as u16 * 512),
                            VIBRATO => synth.set_vibrato((value / 14) as f64),
                            // DAMPER => {
                            //     if value == 127 {
                            //         ignore_note_off = true;
                            //         current_fundamental = temporary_fundamental;
                            //     }
                            // }
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
                            MODULATOR_WAVEFORM => {
                                let waveform = match value / (128 / 4) {
                                    0 => Waveform::Sine,
                                    1 => Waveform::Pulse,
                                    2 => Waveform::Triangle,
                                    3 => Waveform::Sawtooth,
                                    _ => unreachable!(),
                                };
                                synth.set_modulator_waveform(waveform);
                            }
                            DUTY => synth.set_duty(value as f64 / 127.0),
                            MODULATOR_RATIO => synth.set_modulator_ratio(value as u8),
                            MODULATOR_AMOUNT => synth.set_modulator_amount(value as u8),
                            MODULATOR_DUTY => synth.set_modulator_duty(value as f64 / 127.0),
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
