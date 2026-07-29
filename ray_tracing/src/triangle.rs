use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray3;
use core::f64;
use nurbs::vector_3::Vec3;
use std::sync::Arc;

// TODOs:
// - UVs are wrong for proper texturing (uv coords need to be passed by vertex)
// - PDF value is likely wrong

pub struct Triangle {
    mat: Arc<dyn Material + Sync + Send>,
    bbox: AABB,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    normal: Vec3,
    area: f64,
    uvs: [f64; 6],
}

impl Hittable for Triangle {
    fn hit(&self, ray_in: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        // Implemented from (aka copied)
        // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm#Rust_implementation
        let e1 = self.b - self.a;
        let e2 = self.c - self.a;

        let ray_cross_e2 = ray_in.direction().cross(&e2);
        let det = e1.dot(&ray_cross_e2);

        if det.abs() < f64::EPSILON {
            return false;
        }

        let inv_det = 1.0 / det;
        let s = ray_in.origin() - self.a;
        let u = inv_det * s.dot(&ray_cross_e2);

        if u < 0.0 || u > 1.0 {
            return false;
        }

        let s_cross_e1 = s.cross(&e1);
        let v = inv_det * ray_in.direction().dot(&s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return false;
        }

        let t = inv_det * e2.dot(&s_cross_e1);
        if time.contains(t) {
            hit_record.time = t;
            hit_record.point = ray_in.at(t);
            hit_record.mat = Arc::clone(&self.mat);
            hit_record.set_face_normal(ray_in, self.normal);
            hit_record.u = u * self.uvs[4] + v * self.uvs[2] + (1.0 - u - v) * self.uvs[0];
            hit_record.v = u * self.uvs[5] + v * self.uvs[3] + (1.0 - u - v) * self.uvs[1];

            return true;
        } else {
            return false;
        }
    }

    fn bounding_box(&self) -> crate::aabb::AABB {
        AABB::copy(&self.bbox)
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f64 {
        let mut rec = HitRecord::new();
        let ray = Ray3::new(*origin, *direction, 0.0);
        if self.hit(&ray, Interval::new(0.001, f64::INFINITY), &mut rec) {
            let dist_sq = rec.time * rec.time * direction.length_squared();
            let cosine = f64::abs(direction.dot(&rec.normal)) / direction.length();

            dist_sq / (cosine * self.area)
        } else {
            0.0
        }
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let mut rand_uv = Vec3::random_unit_vector();
        if (rand_uv.x + rand_uv.y + rand_uv.z - 1.0).abs() > f64::EPSILON {
            let div = 1.0 / (rand_uv.x + rand_uv.y + rand_uv.z);
            rand_uv.x *= div;
            rand_uv.y *= div;
            rand_uv.z *= div;
        }
        let point = (self.a * rand_uv.x) + (self.b * rand_uv.y) + (self.c * rand_uv.z);
        point - origin.clone()
    }
}

impl Triangle {
    pub fn new(
        a: Vec3,
        b: Vec3,
        c: Vec3,
        uvs: [f64; 6],
        mat: Arc<dyn Material + Sync + Send>,
    ) -> Triangle {
        let bbox_0 = AABB::from_vec3s(a, b);
        let bbox_1 = AABB::from_vec3s(a, c);
        let bbox = AABB::from_aabbs(&bbox_0, &bbox_1);
        let ac = c - a;
        let n = (b - a).cross(&ac);
        let normal = n.unit_vector();
        let area = n.length() * 0.5;
        Triangle {
            mat,
            bbox,
            a,
            b,
            c,
            normal,
            area,
            uvs,
        }
    }
}
