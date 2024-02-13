use ::alsa::seq::EventType;
use alsa::seq::{EvCtrl, EvNote};
use anyhow::Result;
use bpaf::Bpaf;

use crate::hw::IO;
use crate::synth::Synth;

mod envelope;
pub mod file;
pub mod hw;
mod lfo;
mod midi;
mod oscillator;
mod pcm;
pub mod synth;
mod tables;

#[derive(Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short('m'), long, argument)]
    pub main_port: i32,
    #[bpaf(short('a'), long, argument)]
    pub aux_port: i32,
    #[bpaf(short('e'), long, argument)]
    pub expr_port: i32,
    #[bpaf(short('c'), long, argument)]
    pub card: String,
}

pub fn run(options: Options) -> Result<()> {
    let main_port = options.main_port;
    let aux_port = options.aux_port;
    let expr_port = options.expr_port;
    let card = options.card;

    let mut io = IO::new(main_port, aux_port, expr_port, &card)?;
    let mut synth = Synth::new();

    // let mut collecting = false;
    // TODO magic numbers
    let mut temporary_fundamental = 48;
    let mut current_fundamental = 48;
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
                            0 => match note {
                                // TODO magic numbers
                                // 48..=59 => collecting = false,
                                // 60..=72 if !collecting => {
                                48..=59 => {
                                    if !ignore_note_off {
                                        synth.change_fundamental(current_fundamental);
                                    }

                                    active_control_notes -= 1;

                                    if active_control_notes == 0 {
                                        ignore_note_off = false;
                                    }
                                }
                                60..=72 => {
                                    if !ignore_note_off {
                                        synth.change_tuning(note);
                                    }

                                    active_control_notes -= 1;

                                    if active_control_notes == 0 {
                                        ignore_note_off = false;
                                    }
                                }
                                _ => (),
                            },
                            // 1 if sustain => {
                            //     sustained_notes.insert(note);
                            // }
                            1 => synth.silence(note),
                            _ => unreachable!(),
                        }
                    }
                }
                EventType::Noteon => {
                    if let Some(EvNote { channel, note, .. }) = event.get_data() {
                        match channel {
                            0 => match note {
                                // TODO magic numbers
                                48..=59 => {
                                    // collecting = true;
                                    active_control_notes += 1;

                                    temporary_fundamental = note;

                                    synth.change_fundamental(temporary_fundamental);
                                }
                                60..=72 => {
                                    active_control_notes += 1;
                                    synth.change_tuning(note);
                                }
                                _ => (),
                            },
                            1 => {
                                // sustained_notes.remove(&note);
                                synth.play(note);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                EventType::Controller => {
                    if let Some(EvCtrl {
                        // TODO magic numbers
                        param: 21,
                        value,
                        ..
                    }) = event.get_data()
                    {
                        if config_mode {
                            synth.set_modulator_amount(value as u8)
                        } else {
                            synth.set_volume(value as u8)
                        }
                    } else if let Some(EvCtrl {
                        // TODO magic numbers
                        param: 22,
                        value,
                        ..
                    }) = event.get_data()
                    {
                        if config_mode {
                            synth.set_modulator_ratio(value as u8)
                        } else {
                            synth.set_vibrato((value / 14) as f64)
                        }
                    } else if let Some(EvCtrl {
                        param: 64,
                        value,
                        channel,
                        ..
                    }) = event.get_data()
                    {
                        match channel {
                            0 => {
                                config_mode = (value == 127);
                            }
                            1 if value == 127 => {
                                ignore_note_off = true;
                                current_fundamental = temporary_fundamental;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        io.poll()?;
    }
}
