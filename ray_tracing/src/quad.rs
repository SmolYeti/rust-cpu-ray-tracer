use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray3;
use nurbs::vector_3::Vec3;
use core::f64;
use std::sync::Arc;

pub struct Quad {
    mat: Arc<dyn Material + Sync + Send>,
    bbox: AABB,
    origin: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    normal: Vec3,
    plane_const: f64,
    area: f64,
}

impl Hittable for Quad {
    fn hit(&self, ray_in: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        let denom = self.normal.dot(&ray_in.direction());

        let mut quad_hit = false;
        if denom.abs() >= 1e-8 {
            let hit_time = (self.plane_const - self.normal.dot(&ray_in.origin())) / denom;

            if time.contains(hit_time) {
                let intersection = ray_in.at(hit_time);

                let [alpha, beta] = self.uv(intersection);

                if Quad::interior(alpha, beta, hit_record) {
                    hit_record.time = hit_time;
                    hit_record.point = intersection;
                    hit_record.mat = Arc::clone(&self.mat);
                    hit_record.set_face_normal(ray_in, self.normal);

                    quad_hit = true;
                }
            }
        }
        quad_hit
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
        let point =
            self.origin + (self.u * rand::random::<f64>()) + (self.v * rand::random::<f64>());
        point - origin.clone()
    }
}

impl Quad {
    pub fn new(origin: Vec3, u: Vec3, v: Vec3, mat: Arc<dyn Material + Sync + Send>) -> Quad {
        let bbox_0 = AABB::from_vec3s(origin, origin + u + v);
        let bbox_1 = AABB::from_vec3s(origin + u, origin + v);
        let bbox = AABB::from_aabbs(&bbox_0, &bbox_1);
        let n = u.cross(&v);
        let normal = n.unit_vector();
        let plane_const = normal.dot(&origin);
        let w = n / n.dot(&n);
        let area = n.length();
        Quad {
            mat,
            bbox,
            origin,
            u,
            v,
            w,
            normal,
            plane_const,
            area,
        }
    }

    pub fn interior(a: f64, b: f64, hit_record: &mut HitRecord) -> bool {
        if a < 0.0 || 1.0 < a || b < 0.0 || 1.0 < b {
            false
        } else {
            hit_record.u = a;
            hit_record.v = b;
            true
        }
    }

    pub fn uv(&self, point: Vec3) -> [f64; 2] {
        let hitpt_vector = point - self.origin;
        let alpha = self.w.dot(&hitpt_vector.cross(&self.v));
        let beta = self.w.dot(&self.u.cross(&hitpt_vector));
        [alpha, beta]
    }
}

pub fn quad_box(
    point_a: Vec3,
    point_b: Vec3,
    mat: Arc<dyn Material + Sync + Send>,
) -> Arc<HittableList> {
    let bbox = AABB::from_vec3s(point_a, point_b);

    let min = Vec3::new(bbox.axis(0).min(), bbox.axis(1).min(), bbox.axis(2).min());
    let max = Vec3::new(bbox.axis(0).max(), bbox.axis(1).max(), bbox.axis(2).max());

    let dx = Vec3::new(bbox.axis(0).size(), 0.0, 0.0);
    let dy = Vec3::new(0.0, bbox.axis(1).size(), 0.0);
    let dz = Vec3::new(0.0, 0.0, bbox.axis(2).size());

    let mut quads = HittableList::new();

    // front
    quads.add(Arc::new(Quad::new(
        Vec3::new(min.x, min.y, max.z),
        dx,
        dy,
        Arc::clone(&mat),
    )));
    // right
    quads.add(Arc::new(Quad::new(
        Vec3::new(max.x, min.y, max.z),
        -dz,
        dy,
        Arc::clone(&mat),
    )));
    // back
    quads.add(Arc::new(Quad::new(
        Vec3::new(max.x, min.y, min.z),
        -dx,
        dy,
        Arc::clone(&mat),
    )));
    // left
    quads.add(Arc::new(Quad::new(
        Vec3::new(min.x, min.y, min.z),
        dz,
        dy,
        Arc::clone(&mat),
    )));
    // top
    quads.add(Arc::new(Quad::new(
        Vec3::new(min.x, max.y, max.z),
        dx,
        -dz,
        Arc::clone(&mat),
    )));
    // bottom
    quads.add(Arc::new(Quad::new(
        Vec3::new(min.x, min.y, min.z),
        dx,
        dz,
        Arc::clone(&mat),
    )));

    Arc::new(quads)
}
