use std::ffi::CString;

use alsa::seq;
use alsa::seq::{Event, EventType, Input};
use anyhow::Result;

pub struct MidiInputStream {
    device: alsa::Seq,
}

impl MidiInputStream {
    pub fn new(main_port: i32, aux_port: i32) -> Result<Self> {
        Ok(Self {
            device: Self::open_midi_device(main_port, aux_port)?,
        })
    }

    pub fn device(&self) -> &alsa::Seq {
        &self.device
    }

    pub fn input(&self) -> Input {
        self.device.input()
    }

    pub fn open_midi_device(main_port: i32, aux_port: i32) -> Result<alsa::Seq> {
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
                    if port.get_client() == main_port {
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

    pub fn read_midi_event<'a>(mut input: Input) -> Result<Option<Event<'a>>> {
        if input.event_input_pending(true)? == 0 {
            return Ok(None);
        }
        let event = input.event_input()?.into_owned();

        Ok(match event.get_type() {
            EventType::Noteon | EventType::Noteoff | EventType::Controller => Some(event),
            _ => None,
        })
    }
}
