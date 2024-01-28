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

    let mut collecting = false;
    // let mut sustain = false;

    // let mut sustained_notes = BTreeSet::new();

    loop {
        io.write(&mut synth)?;

        if let Some(event) = io.read()? {
            match event.get_type() {
                EventType::Noteoff => {
                    if let Some(EvNote { channel, note, .. }) = event.get_data() {
                        match channel {
                            0 => match note {
                                48..=59 => collecting = false,
                                60..=72 if !collecting => {
                                    synth.change_tuning(note);
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
                                48..=59 => {
                                    collecting = true;
                                    synth.change_fundamental(note);
                                }
                                60..=72 => {
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
                        param: 21, value, ..
                    }) = event.get_data()
                    {
                        // if value > 0 {
                        synth.set_volume(value as u8)
                        // } else {
                        //     synth.set_volume(0)
                        // }
                        // if value > 0 {
                        //     sustain = true;
                        //
                        //     for &note in &sustained_notes {
                        //         synth.silence(note);
                        //     }
                        //
                        //     sustained_notes.clear();
                        // } else {
                        //     sustain = false;
                        // }
                    } else if let Some(EvCtrl {
                        param: 22, value, ..
                    }) = event.get_data()
                    {
                        synth.set_vibrato((value / 16) as f64)
                    }
                }
                _ => {}
            }
        }

        io.poll()?;
    }
}
