use ::alsa::seq::EventType;
use alsa::seq::{EvCtrl, EvNote};
use anyhow::Result;
use bpaf::Bpaf;

use crate::hw::IO;
use crate::synth::Synth;

mod envelope;
mod hw;
mod lfo;
mod midi;
mod oscillator;
mod pcm;
mod synth;
mod tables;

#[derive(Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short('m'), long, argument)]
    pub main_port: i32,
    #[bpaf(short('a'), long, argument)]
    pub aux_port: i32,
}
// #[cfg(test)]
// mod tests {
//     use crate::hw::open_midi_device;
//
//     #[test]
//     fn f() {
//         open_midi_device();
//     }
// }

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
                    if let Some(EvNote { note, .. }) = event.get_data() {
                        match note {
                            21..=33 => collecting = false,
                            34..=45 if !collecting => {
                                synth.change_tuning(note);
                            }
                            _ => {
                                synth.silence(note);
                            }
                        }
                    }
                }
                EventType::Noteon => {
                    if let Some(EvNote { note, velocity, .. }) = event.get_data() {
                        match note {
                            note @ 21..=33 => {
                                collecting = true;
                                synth.change_fundamental(note);
                            }
                            34..=45 => {
                                synth.change_tuning(note);
                            }
                            _ => {
                                synth.play(note, velocity);
                            }
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
