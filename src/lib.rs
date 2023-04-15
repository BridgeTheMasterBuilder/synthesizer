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
}

pub fn run(options: Options) -> Result<()> {
    let main_port = options.main_port;
    let aux_port = options.aux_port;

    let mut io = IO::new(main_port, aux_port, "hw:0")?;
    let mut synth = Synth::new();

    let mut collecting = false;

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
                            1 => synth.silence(note),
                            _ => unreachable!(),
                        }
                    }
                }
                EventType::Noteon => {
                    if let Some(EvNote {
                        channel,
                        note,
                        velocity,
                        ..
                    }) = event.get_data()
                    {
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
                            1 => synth.play(note, velocity),
                            _ => unreachable!(),
                        }
                    }
                }
                EventType::Controller => {
                    if let Some(EvCtrl {
                        param: 64, value, ..
                    }) = event.get_data()
                    {
                        if value > 0 {
                            synth.set_vibrato(5.0)
                        } else {
                            synth.set_vibrato(0.0)
                        }
                    }
                }
                _ => {}
            }
        }

        io.poll()?;
    }
}
