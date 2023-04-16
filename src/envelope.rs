#[derive(Clone, Debug)]
pub struct Envelope {
    target: i16,
    vol: i16,
    incr: i16,
    delay: u8,
}

impl Envelope {
    const DELAY: u8 = 16;
    const INCR: i16 = 64;

    pub fn new(vol: i16) -> Self {
        Self {
            target: vol * 128,
            vol: 0,
            incr: Self::INCR,
            delay: 0,
        }
    }

    pub fn adjust_volume(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        } else {
            self.delay = Self::DELAY;

            // TODO
            self.vol = if self.vol > self.target {
                (self.vol as u16).saturating_sub(self.incr as u16) as i16
            } else if self.vol < self.target {
                self.vol.saturating_add(self.incr)
            } else {
                self.vol
            };
        }
    }

    pub fn set_volume(&mut self, vol: i16) {
        self.target = vol * 128;

        self.incr = Self::INCR;
    }

    pub fn volume(&self) -> f64 {
        self.vol as f64
    }
}
