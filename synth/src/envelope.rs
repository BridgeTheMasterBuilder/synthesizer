#[derive(Clone, Debug, Copy)]
enum State {
    Waiting,
    Attack,
    Decay,
    Sustain,
    Release,
}
#[derive(Clone, Debug, Copy)]
pub struct Envelope {
    enabled: bool,
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
    repeat: bool,
}

impl Envelope {
    const MAX_POLYPHONY: u16 = 88;
    pub const PEAK: u16 = u16::MAX / Self::MAX_POLYPHONY;
    pub const INCR: u16 = 1;
    pub const FACTOR: u32 = 1;

    pub fn new(
        gain: f64,
        attack: u32,
        decay: u32,
        sustain: u16,
        release: u32,
        automatic: bool,
    ) -> Self {
        Self {
            enabled: false,
            gain,
            vol: 0,
            incr: Self::INCR,
            target: 0,
            attack: attack * Self::FACTOR,
            attack_reload: attack * Self::FACTOR,
            decay: decay * Self::FACTOR,
            decay_reload: decay * Self::FACTOR,
            sustain: sustain * 512 / Self::MAX_POLYPHONY,
            release: release * Self::FACTOR,
            release_reload: release * Self::FACTOR,
            state: State::Waiting,
            repeat: automatic,
        }
    }

    // TODO No need for this bool return value if we have self.enabled?
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

                    if self.vol > self.target {
                        self.decay = self.decay_reload;

                        return false;
                    }

                    self.state = State::Sustain;
                }
                false
            }
            State::Sustain => {
                if !self.enabled || self.repeat {
                    self.target = 0;
                    self.state = State::Release
                }

                false
            }
            State::Release => {
                if self.release > 0 {
                    self.release -= 1;

                    false
                } else {
                    self.vol = if self.vol > self.target {
                        self.vol.saturating_sub(self.incr)
                    } else {
                        self.target
                    };

                    if self.vol > self.target {
                        self.release = self.release_reload;

                        return false;
                    }

                    if self.repeat {
                        self.set_volume(255);
                    } else {
                        self.state = State::Waiting;
                    }

                    true
                }
            }
        }
    }

    pub fn set_volume(&mut self, vol: u16) {
        if vol == 0 {
            self.enabled = false;
        } else {
            self.enabled = true;
            self.target = Self::PEAK;
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

    pub fn normalized_volume(&self) -> f64 {
        self.vol as f64 * self.gain / Self::PEAK as f64
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

    pub fn toggle_repeat(&mut self) {
        self.repeat = !self.repeat;
    }
}
