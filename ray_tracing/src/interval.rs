use std::ops;

pub struct Interval {
    min: f64,
    max: f64,
}

impl Interval {
    pub fn empty() -> Interval {
        Interval {
            min: f64::INFINITY,
            max: -f64::INFINITY,
        }
    }

    pub fn univeral() -> Interval {
        Interval {
            min: -f64::INFINITY,
            max: f64::INFINITY,
        }
    }

    pub fn new(min: f64, max: f64) -> Interval {
        Interval { min, max }
    }

    pub fn from_intervals(inter_0: Interval, inter_1: Interval) -> Interval {
        Interval::new(inter_0.min.min(inter_1.min), inter_0.max.max(inter_1.max))
    }

    pub fn copy(&self) -> Interval {
        Interval::new(self.min(), self.max())
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn mid(&self) -> f64 {
        (self.max + self.min) * 0.5
    }

    pub fn set_min(&mut self, val: f64) {
        self.min = val;
    }

    pub fn set_max(&mut self, val: f64) {
        self.max = val;
    }

    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    pub fn expand(&self, delta: f64) -> Interval {
        let padding = delta * 0.5;
        Interval::new(self.min - padding, self.max + padding)
    }
}

// Add
impl ops::Add<f64> for Interval {
    type Output = Interval;

    fn add(self, rhs: f64) -> Interval {
        Interval {
            min: self.min() + rhs,
            max: self.max() + rhs,
        }
    }
}

impl<'a> ops::Add<f64> for &'a Interval {
    type Output = Interval;

    fn add(self, rhs: f64) -> Interval {
        Interval {
            min: self.min() + rhs,
            max: self.max() + rhs,
        }
    }
}

impl ops::Add<Interval> for f64 {
    type Output = Interval;

    fn add(self, rhs: Interval) -> Interval {
        Interval {
            min: self + rhs.min(),
            max: self + rhs.max(),
        }
    }
}

impl<'a> ops::Add<&'a Interval> for f64 {
    type Output = Interval;

    fn add(self, rhs: &'a Interval) -> Interval {
        Interval {
            min: self + rhs.min(),
            max: self + rhs.max(),
        }
    }
}
