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
    attack_reload: u8,
    attack: u8,
    decay_reload: u8,
    decay: u8,
    sustain: u16,
    release_reload: u8,
    release: u8,
    state: State,
}

impl Envelope {
    const MAX_POLYPHONY: u16 = 88;
    pub const PEAK: u16 = u16::MAX / Self::MAX_POLYPHONY;

    pub fn new(gain: f64, attack: u8, decay: u8, sustain: u16, release: u8) -> Self {
        Self {
            gain,
            vol: 0,
            incr: 0,
            target: 0,
            attack_reload: attack,
            attack,
            decay_reload: decay,
            decay,
            sustain,
            release_reload: release,
            release,
            state: State::Waiting,
        }
    }

    pub fn adjust_volume(&mut self) -> bool {
        match self.state {
            State::Waiting => false,
            State::Attack => {
                if self.attack > 0 {
                    self.attack -= 1;

                    self.vol = self.vol.saturating_add(self.incr);
                } else {
                    self.target = self.sustain;

                    self.incr = (Self::PEAK - self.sustain) / self.decay_reload as u16;

                    self.state = State::Decay;
                }
                false
            }
            State::Decay => {
                if self.decay > 0 {
                    self.decay -= 1;

                    self.vol = self.vol.saturating_sub(self.incr);
                } else {
                    self.incr = self.sustain / self.release_reload as u16;

                    self.state = State::Sustain;
                }
                false
            }
            State::Sustain => false,
            State::Release => {
                if self.release > 0 {
                    self.release -= 1;

                    self.vol = self
                        .vol
                        .saturating_sub(self.sustain / self.release_reload as u16);

                    false
                } else {
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
            self.state = State::Release
        } else {
            // self.target = vol * self.gain;
            self.sustain = vol * 512 / Self::MAX_POLYPHONY;
            self.target = Self::PEAK;
            self.incr = Self::PEAK / self.attack_reload as u16;
            self.attack = self.attack_reload;
            self.decay = self.decay_reload;
            self.release = self.release_reload;
            self.state = State::Attack;
        }
    }

    pub fn set_gain(&mut self, gain: f64) {
        self.gain = gain;
    }

    pub fn volume(&self) -> u16 {
        (self.vol as f64 * self.gain) as u16
    }
}
