use std::ffi::CString;

use alsa::direct::pcm::MmapPlayback;
use alsa::pcm::{Frames, State};
use alsa::seq::{Event, EventType};
use alsa::{pcm, seq};
use alsa::{Direction, ValueOr, PCM};
use anyhow::anyhow;
use anyhow::Result;

use crate::synth::Synth;

const CARD: &str = "hw:0";
pub const SAMPLE_RATE: u32 = 44100;
const BUFFER_SIZE: Frames = 512;
const PERIOD_SIZE: Frames = BUFFER_SIZE / 4;
const INPUT_PORT: i32 = 24;

pub type SF = i16;

pub fn open_midi_device() -> Result<alsa::Seq> {
    let s = alsa::Seq::open(None, Some(alsa::Direction::Capture), true)?;
    let cstr = CString::new("instrument").unwrap();
    s.set_client_name(&cstr)?;

    let mut dinfo = seq::PortInfo::empty().unwrap();
    dinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE);
    dinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
    dinfo.set_name(&cstr);
    s.create_port(&dinfo).unwrap();
    let dport = dinfo.get_port();

    let client = seq::ClientIter::new(&s)
        .find_map(|client| {
            seq::PortIter::new(&s, client.get_client()).find_map(|port| {
                if port.get_client() == INPUT_PORT {
                    Some(port)
                } else {
                    None
                }
            })
        })
        .unwrap();

    let sub = seq::PortSubscribe::empty()?;
    sub.set_sender(seq::Addr {
        client: client.get_client(),
        port: client.get_port(),
    });
    sub.set_dest(seq::Addr {
        client: s.client_id()?,
        port: dport,
    });
    s.subscribe_port(&sub)?;

    println!("Opening MIDI device");

    Ok(s)
}

pub fn open_audio_device() -> Result<PCM> {
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

    let device = PCM::new(CARD, Direction::Playback, false)?;

    set_hw_params(&device)?;
    set_sw_params(&device)?;

    println!("Opening audio device");

    Ok(device)
}

pub fn write_samples_direct(p: &PCM, mmap: &mut MmapPlayback<SF>, mixer: &mut Synth) -> Result<()> {
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
                eprintln!("underrun");
                p.prepare()?
            }
            State::Suspended => p.resume()?,
            n => Err(anyhow!(format!("Unexpected pcm state {:?}", n)))?,
        }
    }
}

pub fn read_midi_event<'a>(input: &'a mut seq::Input) -> Result<Option<Event<'a>>> {
    if input.event_input_pending(true)? == 0 {
        return Ok(None);
    }
    let event = input.event_input()?;

    Ok(match event.get_type() {
        EventType::Noteon | EventType::Noteoff | EventType::Controller => Some(event),
        _ => None,
    })
}
