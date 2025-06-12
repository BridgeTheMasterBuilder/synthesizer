use crate::hw::SAMPLE_RATE;

#[derive(Clone, Debug)]
enum State {
    Waiting,
    Attack,
    Decay,
    Sustain,
    Release,
}
#[derive(Clone, Debug)]
pub struct Envelope {
    gain: f64,
    vol: u16,
    incr: u16,
    target: u16,
    attack: u32,
    attack_reload: u32,
    decay: u32,
    decay_reload: u32,
    sustain: u16,
    release: u32,
    release_reload: u32,
    state: State,
}

impl Envelope {
    const MAX_POLYPHONY: u16 = 88;
    pub const PEAK: u16 = u16::MAX / Self::MAX_POLYPHONY;
    pub const INCR: u16 = 1;
    // pub const MAX_DELAY: u32 = 10 * SAMPLE_RATE;
    // pub const DELAY: u32 = 10 * SAMPLE_RATE / 128;
    // pub const FACTOR: u32 = 10 * SAMPLE_RATE;
    pub const FACTOR: u32 = 1;
    // pub const MIN_DELAY: u32 = 1024;

    pub fn new(gain: f64, attack: u32, decay: u32, sustain: u16, release: u32) -> Self {
        Self {
            gain,
            vol: 0,
            incr: Self::INCR,
            target: 0,
            // delay: Self::MAX_DELAY / 128 + Self::MIN_DELAY,
            // delay: Self::MAX_DELAY / 128,
            attack: attack * Self::FACTOR,
            attack_reload: attack * Self::FACTOR,
            decay: decay * Self::FACTOR,
            decay_reload: decay * Self::FACTOR,
            sustain: sustain * 512 / Self::MAX_POLYPHONY,
            release: release * Self::FACTOR,
            release_reload: release * Self::FACTOR,
            state: State::Waiting,
        }
    }

    // TODO if note is silenced during attack phase, self.target will get overwritten
    pub fn adjust_volume(&mut self) -> bool {
        match self.state {
            State::Waiting => false,
            State::Attack => {
                if self.attack > 0 {
                    self.attack -= 1;
                } else {
                    self.vol = if self.vol + self.incr < self.target {
                        self.vol + self.incr
                    } else {
                        self.target
                    };

                    if self.vol + self.incr < self.target {
                        self.attack = self.attack_reload;

                        return false;
                    }

                    self.target = self.sustain;
                    // self.incr = (Self::PEAK - self.sustain) / self.decay_reload as u16;

                    self.state = State::Decay;
                }
                false
            }
            State::Decay => {
                if self.decay > 0 {
                    self.decay -= 1;
                } else {
                    self.vol = if self.vol > self.target {
                        self.vol.saturating_sub(self.incr)
                    } else {
                        self.target
                    };

                    // self.incr = self.sustain / self.release_reload as u16;
                    if self.vol > self.target {
                        self.decay = self.decay_reload;

                        return false;
                    }

                    self.state = State::Sustain;
                }
                false
            }
            State::Sustain => {
                if self.target == 0 {
                    self.state = State::Release
                }

                false
            }
            State::Release => {
                if self.release > 0 {
                    self.release -= 1;

                    false
                } else {
                    // self.vol = self.vol.saturating_sub(self.incr);
                    self.vol = if self.vol > self.target {
                        self.vol.saturating_sub(self.incr)
                    } else {
                        self.target
                    };

                    if self.vol > self.target {
                        self.release = self.release_reload;

                        return false;
                    }

                    self.state = State::Waiting;

                    true
                }
            }
        }

        // if self.target > 0 {
        //     if self.attack > 0 {
        //         self.attack -= 1;
        //
        //         self.vol = self.vol.saturating_add(self.incr);
        //
        //         return false;
        //     } else {
        //         println!("Attack phase is over");
        //         self.target = self.sustain;
        //         dbg!(self.target);
        //
        //         self.incr = (u16::MAX - self.sustain) / self.decay_reload as u16;
        //     }
        //
        //     if self.decay > 0 {
        //         println!("Decaying");
        //         dbg!(self.decay);
        //         dbg!(self.target);
        //         self.decay -= 1;
        //
        //         self.vol = self.vol.saturating_sub(self.incr);
        //
        //         false
        //     } else {
        //         println!("Decay phase is over");
        //         dbg!(self.target);
        //
        //         self.incr = self.sustain / self.release_reload as u16;
        //
        //         false
        //     }
        // } else if old_vol > 0 {
        //     if self.release > 0 {
        //         println!("Releaseing");
        //         dbg!(self.release);
        //         dbg!(self.target);
        //         self.release -= 1;
        //
        //         self.vol = self
        //             .vol
        //             .saturating_sub(self.sustain / self.release_reload as u16);
        //
        //         false
        //     } else {
        //         println!("Release phase is over");
        //
        //         old_vol != 0
        //     }
        // } else {
        //     false
        // }
    }

    // if self.delay > 0 {
    //     self.delay -= 1;
    //
    //     false
    // } else {
    //     self.delay = Self::DELAY;
    //
    //     let old_vol = self.vol;
    //
    //     self.vol = if self.vol > self.target {
    //         self.vol.saturating_sub(self.incr)
    //     } else if self.vol + self.incr < self.target {
    //         self.vol + self.incr
    //     } else {
    //         self.target
    //     };
    //
    //     self.vol == 0 && old_vol != 0
    // }

    pub fn set_volume(&mut self, vol: u16) {
        if vol == 0 {
            self.target = 0;
        } else {
            // self.target = vol * self.gain;
            // self.sustain = vol * 512 / Self::MAX_POLYPHONY;
            self.target = Self::PEAK;
            // self.incr = Self::PEAK / self.attack_reload as u16;
            self.attack = self.attack_reload;
            self.decay = self.decay_reload;
            self.release = self.release_reload;
            // self.set_delay(self.attack);
            self.state = State::Attack;
        }
    }

    // fn set_delay(&mut self, delay: u32) {
    //     self.delay = (Self::MAX_DELAY * delay) / 128 + Self::MIN_DELAY;
    // }

    pub fn set_gain(&mut self, gain: f64) {
        self.gain = gain;
    }

    pub fn volume(&self) -> u16 {
        (self.vol as f64 * self.gain) as u16
    }

    pub fn set_attack(&mut self, value: u8) {
        self.attack_reload = (value + 1) as u32;
    }

    pub fn set_decay(&mut self, value: u8) {
        self.decay_reload = (value + 1) as u32;
    }

    pub fn set_sustain(&mut self, value: u8) {
        self.sustain = value as u16 * 512 / Self::MAX_POLYPHONY;
    }

    pub fn set_release(&mut self, value: u8) {
        self.release_reload = (value + 1) as u32;
    }
}
