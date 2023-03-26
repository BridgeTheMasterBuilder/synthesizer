#[derive(Debug)]
pub struct Envelope {
    target: i16,
    vol: i16,
    incr: i16,
    delay: u8,
}

impl Envelope {
    const DELAY: u8 = 16;

    pub fn new(vol: i16) -> Self {
        Self {
            target: vol,
            vol: 1,
            incr: 1,
            delay: Self::DELAY,
        }
    }

    pub fn adjust_volume(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
            return;
        } else {
            self.delay = Self::DELAY;

            self.vol += if self.target == 0 {
                -self.incr
            } else if self.vol < self.target {
                self.incr
            } else {
                0
            };
        }
    }

    pub fn set_volume(&mut self, vol: i16) {
        self.target = vol;
        if self.vol == 0 {
            self.vol = 1;
        }
        self.incr = 1;
    }

    pub fn volume(&self) -> f64 {
        self.vol as f64
    }
}
