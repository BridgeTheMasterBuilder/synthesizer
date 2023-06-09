#[derive(Clone, Debug)]
pub struct Envelope {
    target: u16,
    vol: u16,
    incr: u16,
    delay: u8,
}

impl Envelope {
    const DELAY: u8 = 16;
    const INCR: u16 = 64;

    pub fn new(vol: u16) -> Self {
        Self {
            target: vol * 128,
            vol: 0,
            incr: Self::INCR,
            delay: 0,
        }
    }

    pub fn adjust_volume(&mut self) -> bool {
        if self.delay > 0 {
            self.delay -= 1;

            false
        } else {
            self.delay = Self::DELAY;

            let old_vol = self.vol;

            self.vol = if self.vol > self.target {
                self.vol.saturating_sub(self.incr)
            } else if self.vol + self.incr < self.target {
                self.vol + self.incr
            } else {
                self.target
            };

            self.vol == 0 && old_vol != 0
        }
    }

    pub fn set_volume(&mut self, vol: u16) {
        self.target = vol * 128;

        self.incr = Self::INCR;
    }

    pub fn volume(&self) -> f64 {
        self.vol as f64
    }
}
