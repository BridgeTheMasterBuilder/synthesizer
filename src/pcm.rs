use alsa::direct::pcm::MmapPlayback;
use alsa::pcm::{Frames, State};
use alsa::{pcm, Direction, PollDescriptors, ValueOr, PCM};
use anyhow::{anyhow, Result};

use crate::hw::{SAMPLE_RATE, SF};
use crate::synth::Synth;

const BUFFER_SIZE: Frames = 512;
const PERIOD_SIZE: Frames = BUFFER_SIZE / 4;

pub struct OutputDevice {
    device: PCM,
}

impl OutputDevice {
    pub fn new(card: &str) -> Result<Self> {
        Ok(Self {
            device: Self::open_audio_device(card)?,
        })
    }

    pub fn device(&self) -> &PCM {
        &self.device
    }

    pub fn get(&self) -> alsa::Result<Vec<alsa::poll::pollfd>> {
        self.device.get()
    }

    pub fn open_audio_device(card: &str) -> Result<PCM> {
        fn set_hw_params(device: &PCM) -> Result<()> {
            let hw_params = pcm::HwParams::any(device)?;
            hw_params.set_channels(2)?;
            hw_params.set_rate(SAMPLE_RATE, ValueOr::Nearest)?;
            hw_params.set_format(pcm::Format::s16())?;
            hw_params.set_access(pcm::Access::MMapInterleaved)?;
            hw_params.set_buffer_size(BUFFER_SIZE)?;
            hw_params.set_period_size(PERIOD_SIZE, ValueOr::Nearest)?;
            device.hw_params(&hw_params)?;

            Ok(())
        }

        fn set_sw_params(device: &PCM) -> Result<()> {
            let swp = device.sw_params_current()?;
            swp.set_start_threshold(BUFFER_SIZE - PERIOD_SIZE)?;
            swp.set_avail_min(PERIOD_SIZE)?;
            device.sw_params(&swp)?;

            Ok(())
        }

        let device = PCM::new(card, Direction::Playback, false)?;

        set_hw_params(&device)?;
        set_sw_params(&device)?;

        println!("Opening audio device");

        Ok(device)
    }

    pub fn write_samples_direct(
        p: &PCM,
        mmap: &mut MmapPlayback<SF>,
        mixer: &mut Synth,
    ) -> Result<()> {
        loop {
            if mmap.avail() > 0 {
                mmap.write(mixer);
            }

            match mmap.status().state() {
                State::Running => {
                    return Ok(());
                }
                State::Prepared => p.start()?,
                State::XRun => {
                    eprintln!("Underrun");
                    p.prepare()?
                }
                State::Suspended => p.resume()?,
                n => Err(anyhow!(format!("Unexpected pcm state {:?}", n)))?,
            }
        }
    }
}
