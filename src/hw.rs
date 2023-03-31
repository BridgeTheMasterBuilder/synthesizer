use alsa::direct::pcm::MmapPlayback;
use alsa::poll::poll;
#[cfg(not(test))]
use alsa::seq::Event;
use alsa::Direction;
use alsa::PollDescriptors;
use anyhow::Result;

use crate::midi::MidiInputStream;
use crate::pcm::OutputDevice;
use crate::synth::Synth;

pub const SAMPLE_RATE: u32 = 44100;

pub type SF = i16;

pub struct IO {
    input_stream: MidiInputStream,
    output_device: OutputDevice,
    mmap: MmapPlayback<SF>,
    fds: Vec<alsa::poll::pollfd>,
}

impl IO {
    pub fn new(main_port: i32, aux_port: i32, card: &str) -> Result<Self> {
        let input_stream = MidiInputStream::new(main_port, aux_port)?;
        let output_device = OutputDevice::new(card)?;

        let mut fds = output_device.get()?;

        fds.append(&mut (input_stream.device(), Some(Direction::Capture)).get()?);

        Ok(Self {
            input_stream,
            mmap: output_device.device().direct_mmap_playback::<SF>()?,
            fds,
            output_device,
        })
    }

    pub fn poll(&mut self) -> Result<usize> {
        Ok(poll(&mut self.fds, -1)?)
    }

    pub fn read(&mut self) -> Result<Option<Event>> {
        MidiInputStream::read_midi_event(self.input_stream.input())
    }

    pub fn write(&mut self, synth: &mut Synth) -> Result<()> {
        OutputDevice::write_samples_direct(self.output_device.device(), &mut self.mmap, synth)
    }
}
