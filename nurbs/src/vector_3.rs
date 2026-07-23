use core::ops;
use crate::point_types::Point3D;
use crate::utility::random_f64_range;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn from_point(point: Point3D) -> Vec3 {
        Vec3{x: point.x(), y: point.y(), z: point.z()}
    }

    pub fn empty() -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    pub fn set_vals(&mut self, x: f64, y: f64, z: f64) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn set_vec(&mut self, xyz: Vec3) {
        self.set_vals(xyz.x, xyz.y, xyz.z);
    }

    pub fn at(&self, index: u32) -> f64 {
        if index == 0 {
            self.x
        } else if index == 1 {
            self.y
        } else {
            self.z
        }
    }

    pub fn set(&mut self, index: u32, value: f64) {
        if index == 0 {
            self.x = value;
        } else if index == 1 {
            self.y = value;
        } else {
            self.z = value;
        }
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.dot(self)
    }

    pub fn near_zero(&self) -> bool {
        self.x.abs() < f64::EPSILON && self.y.abs() < f64::EPSILON && self.z.abs() < f64::EPSILON
    }

    pub fn dot(&self, _rhs: &Vec3) -> f64 {
        (self.x * _rhs.x) + (self.y * _rhs.y) + (self.z * _rhs.z)
    }

    pub fn cross(&self, _rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * _rhs.z - self.z * _rhs.y,
            y: self.z * _rhs.x - self.x * _rhs.z,
            z: self.x * _rhs.y - self.y * _rhs.x,
        }
    }

    pub fn unit_vector(&self) -> Vec3 {
        self.clone() / self.length()
    }

    pub fn random() -> Vec3 {
        Vec3 {
            x: rand::random::<f64>(),
            y: rand::random::<f64>(),
            z: rand::random::<f64>(),
        }
    }

    pub fn random_range(min: f64, max: f64) -> Vec3 {
        Vec3 {
            x: random_f64_range(min, max),
            y: random_f64_range(min, max),
            z: random_f64_range(min, max),
        }
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        let mut sphere = Vec3::random_range(-1.0, 1.0);
        while sphere.length_squared() > 1.0 {
            sphere = Vec3::random_range(-1.0, 1.0);
        }
        sphere
    }

    pub fn random_unit_vector() -> Vec3 {
        Vec3::random_in_unit_sphere().unit_vector()
    }

    pub fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
        let on_sphere = Vec3::random_unit_vector();
        if on_sphere.dot(normal) > 0.0 {
            on_sphere
        } else {
            -on_sphere
        }
    }

    pub fn random_in_unit_disk() -> Vec3 {
        let mut disk = Vec3::random_range(-1.0, 1.0);
        disk.z = 0.0;
        while disk.length_squared() > 1.0 {
            disk = Vec3::random_range(-1.0, 1.0);
        }
        disk
    }

    pub fn random_cosine_direction() -> Vec3 {
        let r1 = rand::random::<f64>();
        let r2 = rand::random::<f64>();
        let sqrt_r2 = r2.sqrt();

        let phi = 2.0 * PI * r1;
        let x = f64::cos(phi) * sqrt_r2;
        let y = f64::sin(phi) * sqrt_r2;
        let z = f64::sqrt(1.0 - r2);

        Vec3::new(x, y, z)
    }

    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        self.clone() - (2.0 * self.dot(normal) * normal)
    }

    pub fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta = (-uv).dot(n).min(1.0);
        let ray_out_perp = etai_over_etat * (uv.clone() + (cos_theta * n));
        let ray_out_parallel = -(1.0 - ray_out_perp.length_squared()).abs().sqrt() * n;
        ray_out_perp + ray_out_parallel
    }
}

// Negate
impl ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<'a> ops::Neg for &'a Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// Add
impl ops::Add<Self> for Vec3 {
    type Output = Vec3;

    fn add(self, _rhs: Self) -> Vec3 {
        Vec3 {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
            z: self.z + _rhs.z,
        }
    }
}

impl<'a, 'b> ops::Add<&'a Self> for &'b Vec3 {
    type Output = Vec3;

    fn add(self, _rhs: &Self) -> Vec3 {
        Vec3 {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
            z: self.z + _rhs.z,
        }
    }
}

impl ops::Add<f64> for Vec3 {
    type Output = Vec3;

    fn add(self, _rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x + _rhs,
            y: self.y + _rhs,
            z: self.z + _rhs,
        }
    }
}

impl<'a> ops::Add<f64> for &'a Vec3 {
    type Output = Vec3;

    fn add(self, _rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x + _rhs,
            y: self.y + _rhs,
            z: self.z + _rhs,
        }
    }
}

impl ops::Add<Vec3> for f64 {
    type Output = Vec3;

    fn add(self, _rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self + _rhs.x,
            y: self + _rhs.y,
            z: self + _rhs.z,
        }
    }
}

impl<'a> ops::Add<&'a Vec3> for f64 {
    type Output = Vec3;

    fn add(self, _rhs: &'a Vec3) -> Vec3 {
        Vec3 {
            x: self + _rhs.x,
            y: self + _rhs.y,
            z: self + _rhs.z,
        }
    }
}

// Add Assign
impl ops::AddAssign<Self> for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::AddAssign<f64> for Vec3 {
    fn add_assign(&mut self, rhs: f64) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

// Subtract
impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, _rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
            z: self.z - _rhs.z,
        }
    }
}

impl<'a, 'b> ops::Sub<&'a Vec3> for &'b Vec3 {
    type Output = Vec3;

    fn sub(self, _rhs: &'a Vec3) -> Vec3 {
        Vec3 {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
            z: self.z - _rhs.z,
        }
    }
}

impl ops::Sub<f64> for Vec3 {
    type Output = Vec3;

    fn sub(self, _rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x - _rhs,
            y: self.y - _rhs,
            z: self.z - _rhs,
        }
    }
}

impl<'a> ops::Sub<f64> for &'a Vec3 {
    type Output = Vec3;

    fn sub(self, _rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x - _rhs,
            y: self.y - _rhs,
            z: self.z - _rhs,
        }
    }
}

impl ops::Sub<Vec3> for f64 {
    type Output = Vec3;

    fn sub(self, _rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self - _rhs.x,
            y: self - _rhs.y,
            z: self - _rhs.z,
        }
    }
}

impl<'a> ops::Sub<&'a Vec3> for f64 {
    type Output = Vec3;

    fn sub(self, _rhs: &'a Vec3) -> Vec3 {
        Vec3 {
            x: self - _rhs.x,
            y: self - _rhs.y,
            z: self - _rhs.z,
        }
    }
}

// Multiply
impl ops::Mul<Self> for Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs: Self) -> Vec3 {
        Vec3 {
            x: self.x * _rhs.x,
            y: self.y * _rhs.y,
            z: self.z * _rhs.z,
        }
    }
}

impl<'a, 'b> ops::Mul<&'a Self> for &'b Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs: &Self) -> Vec3 {
        Vec3 {
            x: self.x * _rhs.x,
            y: self.y * _rhs.y,
            z: self.z * _rhs.z,
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x * _rhs,
            y: self.y * _rhs,
            z: self.z * _rhs,
        }
    }
}

impl<'a> ops::Mul<f64> for &'a Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x * _rhs,
            y: self.y * _rhs,
            z: self.z * _rhs,
        }
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, _rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self * _rhs.x,
            y: self * _rhs.y,
            z: self * _rhs.z,
        }
    }
}

impl<'a> ops::Mul<&'a Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, _rhs: &'a Vec3) -> Vec3 {
        Vec3 {
            x: self * _rhs.x,
            y: self * _rhs.y,
            z: self * _rhs.z,
        }
    }
}

// Divide
impl ops::Div<Vec3> for Vec3 {
    type Output = Self;

    fn div(self, _rhs: Self) -> Self {
        Self {
            x: self.x / _rhs.x,
            y: self.y / _rhs.y,
            z: self.z / _rhs.z,
        }
    }
}

impl<'a, 'b> ops::Div<&'a Vec3> for &'b Vec3 {
    type Output = Vec3;

    fn div(self, _rhs: &'a Vec3) -> Vec3 {
        Vec3 {
            x: self.x / _rhs.x,
            y: self.y / _rhs.y,
            z: self.z / _rhs.z,
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, _rhs: f64) -> Self {
        Self {
            x: self.x / _rhs,
            y: self.y / _rhs,
            z: self.z / _rhs,
        }
    }
}

impl<'a> ops::Div<f64> for &'a Vec3 {
    type Output = Vec3;

    fn div(self, _rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x / _rhs,
            y: self.y / _rhs,
            z: self.z / _rhs,
        }
    }
}

impl ops::Div<Vec3> for f64 {
    type Output = Vec3;

    fn div(self, _rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self / _rhs.x,
            y: self / _rhs.y,
            z: self / _rhs.z,
        }
    }
}

impl<'a> ops::Div<&'a Vec3> for f64 {
    type Output = Vec3;

    fn div(self, _rhs: &'a Vec3) -> Vec3 {
        Vec3 {
            x: self / _rhs.x,
            y: self / _rhs.y,
            z: self / _rhs.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::point_types::Point3D;
    use crate::vector_3::Vec3;

    #[test]
    fn test_new() {
        let vec = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);
    }

    #[test]
    fn test_point() {
        let point = Point3D::new([1.0, 2.0, 3.0]);
        let vec = Vec3::from_point(point);
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);
    }

    #[test]
    fn test_empty() {
        let vec = Vec3::empty();
        assert_eq!(vec.x, 0.0);
        assert_eq!(vec.y, 0.0);
        assert_eq!(vec.z, 0.0);
    }

    #[test]
    fn test_set_vals() {
        let mut vec = Vec3::empty();
        vec.set_vals(1.0, 2.0, 3.0);
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);
    }

    #[test]
    fn test_set_vec() {
        let vec_set = Vec3::new(1.0, 2.0, 3.0);
        let mut vec = Vec3::empty();
        vec.set_vec(vec_set);
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);
    }
}
