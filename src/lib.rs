use ::alsa::poll::poll;
use ::alsa::seq::EventType;
use ::alsa::{Direction, PollDescriptors};
use alsa::seq::{EvCtrl, EvNote};
use anyhow::Result;

use crate::hw::{open_audio_device, open_midi_device, read_midi_event, write_samples_direct, SF};
use crate::synth::Synth;

mod envelope;
mod hw;
mod lfo;
mod oscillator;
mod synth;
mod tables;

pub fn run() -> Result<()> {
    let audio_dev = open_audio_device()?;
    let midi_dev = open_midi_device()?;
    let mut midi_input = midi_dev.input();
    let mut synth = Synth::new();
    let mut mmap = audio_dev.direct_mmap_playback::<SF>()?;
    let mut fds = audio_dev.get()?;
    fds.append(&mut (&midi_dev, Some(Direction::Capture)).get()?);

    let mut collecting = false;

    loop {
        write_samples_direct(&audio_dev, &mut mmap, &mut synth)?;

        if let Some(event) = read_midi_event(&mut midi_input)? {
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

            continue;
        }

        poll(&mut fds, -1)?;
    }
}
