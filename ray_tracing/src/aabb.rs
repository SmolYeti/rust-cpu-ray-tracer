use std::ops;

use crate::interval::Interval;
use crate::ray::Ray3;
use nurbs::vector_3::Vec3;
pub struct AABB {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl AABB {
    pub fn new(x: Interval, y: Interval, z: Interval) -> AABB {
        AABB { x, y, z }
    }

    pub fn copy(&self) -> AABB {
        AABB::new(
            Interval::copy(&self.x),
            Interval::copy(&self.y),
            Interval::copy(&self.z),
        )
    }

    pub fn empty() -> AABB {
        AABB::new(Interval::empty(), Interval::empty(), Interval::empty())
    }

    pub fn from_vec3s(a: Vec3, b: Vec3) -> AABB {
        let x = Interval::new(a.x.min(b.x), a.x.max(b.x));
        let y = Interval::new(a.y.min(b.y), a.y.max(b.y));
        let z = Interval::new(a.z.min(b.z), a.z.max(b.z));
        AABB::new(x, y, z)
    }

    pub fn from_aabbs(a: &AABB, b: &AABB) -> AABB {
        let x = Interval::from_intervals(Interval::copy(&a.x), Interval::copy(&b.x));
        let y = Interval::from_intervals(Interval::copy(&a.y), Interval::copy(&b.y));
        let z = Interval::from_intervals(Interval::copy(&a.z), Interval::copy(&b.z));
        AABB::new(x, y, z)
    }

    pub fn axis(&self, n: u32) -> Interval {
        if n == 0 {
            Interval::copy(&self.x)
        } else if n == 1 {
            Interval::copy(&self.y)
        } else {
            Interval::copy(&self.z)
        }
    }

    pub fn hit(&self, ray_in: &Ray3, ray_interval: Interval) -> bool {
        let mut ray_interval = ray_interval;
        let mut hit = true;
        for a in 0..2 {
            let inv_d = 1.0 / ray_in.direction().at(a);
            let origin = ray_in.origin().at(a);
            let mut t0 = (self.axis(a).min() - origin) * inv_d;
            let mut t1 = (self.axis(a).max() - origin) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            ray_interval.set_min(t0.max(ray_interval.min()));
            ray_interval.set_max(t1.min(ray_interval.max()));

            if ray_interval.max() <= ray_interval.min() {
                hit = false
            }
        }
        hit
    }

    pub fn pad(&self) -> AABB {
        let delta = 0.0001;
        let x = if self.x.size() < delta {
            self.x.expand(delta)
        } else {
            self.x.copy()
        };
        let y = if self.y.size() < delta {
            self.y.expand(delta)
        } else {
            self.y.copy()
        };
        let z = if self.z.size() < delta {
            self.z.expand(delta)
        } else {
            self.z.copy()
        };
        AABB::new(x, y, z)
    }

    pub fn expand(&self, delta: f64) -> AABB {
        let x = self.x.expand(delta);
        let y = self.y.expand(delta);
        let z = self.z.expand(delta);
        AABB::new(x, y, z)
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(self.x.mid(), self.y.mid(), self.z.mid())
    }
}

// Add
impl ops::Add<Vec3> for AABB {
    type Output = AABB;

    fn add(self, rhs: Vec3) -> AABB {
        AABB {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<'a> ops::Add<Vec3> for &'a AABB {
    type Output = AABB;

    fn add(self, rhs: Vec3) -> AABB {
        AABB {
            x: self.x.copy() + rhs.x,
            y: self.y.copy() + rhs.y,
            z: self.z.copy() + rhs.z,
        }
    }
}

impl ops::Add<AABB> for Vec3 {
    type Output = AABB;

    fn add(self, rhs: AABB) -> AABB {
        AABB {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<'a> ops::Add<&'a AABB> for Vec3 {
    type Output = AABB;

    fn add(self, rhs: &'a AABB) -> AABB {
        AABB {
            x: self.x + rhs.x.copy(),
            y: self.y + rhs.y.copy(),
            z: self.z + rhs.z.copy(),
        }
    }
}
