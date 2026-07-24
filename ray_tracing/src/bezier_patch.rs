use core::f64;
use std::f64::consts::PI;
use std::sync::Arc;

use crate::aabb::AABB;
use crate::hittable::HitRecord;
use crate::hittable::Hittable;
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::material::Material;
use crate::quad::Quad;
use crate::ray::Ray3;
use nurbs::bezier_curve::BezierCurve3D;
use nurbs::bezier_surface::BezierSurface;
use nurbs::point_types::Point2D;
use nurbs::point_types::Point3D;
use nurbs::point_types::Point4D;
use nurbs::surface::Surface;
use nurbs::vector_3::Vec3;

pub struct BezierPatch {
    surface: BezierSurface,
    mat: Arc<dyn Material + Sync + Send>,
    bbox: AABB,
    debug: bool,
    quads: QuadHitList,
}

impl Hittable for BezierPatch {
    fn hit(&self, ray_in: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        if self.debug {
            self.quads.hit(ray_in, time, hit_record)
        } else {
            // Intersection code based on Ray Tracing Bezier Surfaces on the GPU by Joakim Low
            let o = ray_in.origin();
            let d = ray_in.direction().unit_vector();
            let mut n1 = Vec3::new(0.0, d.z, -d.y);
            if d.x.abs() > d.y.abs() && d.x.abs() > d.z.abs() {
                n1 = Vec3::new(d.y, -d.x, 0.0);
            }
            let n2 = n1.cross(&d);
            let f = |uv: Point2D| -> Point2D {
                let surf_point = Vec3::from_point(self.surface.evaluate(uv));
                Point2D::new([
                    n1.dot(&surf_point) - n1.dot(&o),
                    n2.dot(&surf_point) - n2.dot(&o),
                ])
            };
            let j_inv = |uv: Point2D| -> Point4D {
                let su = Vec3::from_point(self.surface.partial_derivative_u(uv));
                let sv = Vec3::from_point(self.surface.partial_derivative_v(uv));
                let j = Point4D::new([n1.dot(&su), n1.dot(&sv), n2.dot(&su), n2.dot(&sv)]);
                let det_j = (j.x() * j.w()) - (j.y() * j.z());
                Point4D::new([j.w(), -j.y(), -j.z(), j.z()]) / det_j
            };

            // let u0 = inital_guess;
            let mut uv = self.quads.test_hit(&ray_in, &time);
            let mut n: u32 = 0;
            let max_iter: u32 = 50;
            let mut error: f64 = 1000.0;
            let tolerance: f64 = 0.001;
            let mut last_error = 1001.0;
            while n < max_iter && error > tolerance && error < last_error {
                let inv = j_inv(uv);
                let fun = f(uv);
                uv = uv
                    - Point2D::new([
                        inv.x() * fun.x() + inv.y() * fun.y(),
                        inv.z() * fun.x() + inv.w() * fun.y(),
                    ]);
                last_error = error;
                let fun1 = f(uv);
                error = fun1.dot(fun1);
                n += 1;
            }

            let intersection = Vec3::from_point(self.surface.evaluate(uv));
            let hit_time = (intersection - ray_in.origin()).length() / ray_in.direction().length();

            if error < tolerance && time.contains(hit_time) {
                let su = Vec3::from_point(self.surface.partial_derivative_u(uv));
                let sv = Vec3::from_point(self.surface.partial_derivative_v(uv));
                let normal = su.cross(&sv).unit_vector();
                hit_record.time = hit_time;
                hit_record.point = intersection;
                hit_record.mat = Arc::clone(&self.mat);
                hit_record.set_face_normal(ray_in, normal);
            }

            error < tolerance
        }
    }

    fn bounding_box(&self) -> crate::aabb::AABB {
        AABB::copy(&self.bbox)
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f64 {
        self.quads.pdf_value(origin, direction)
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let point = Vec3::from_point(
            self.surface
                .evaluate(Point2D::new([rand::random::<f64>(), rand::random::<f64>()])),
        );
        point - origin.clone()
    }
}

impl BezierPatch {
    pub fn new(surface: BezierSurface, mat: Arc<dyn Material + Sync + Send>) -> BezierPatch {
        // Calculate the bbox
        let mut vec_min = Vec3::new(f64::MAX, f64::MAX, f64::MAX);
        let mut vec_max = Vec3::new(f64::MIN, f64::MIN, f64::MIN);

        for curve in surface.get_curves() {
            for iter in 0..curve.points().len() {
                let point = curve.points()[iter];
                // Bbox calc
                if vec_min.x > point.x() {
                    vec_min.x = point.x();
                }
                if vec_min.y > point.y() {
                    vec_min.y = point.y();
                }
                if vec_min.z > point.z() {
                    vec_min.z = point.z();
                }
                if vec_max.x < point.x() {
                    vec_max.x = point.x();
                }
                if vec_max.y < point.y() {
                    vec_max.y = point.y();
                }
                if vec_max.z < point.z() {
                    vec_max.z = point.z();
                }
            }
        }

        let bbox = AABB::from_vec3s(vec_min, vec_max);

        let quads = BezierPatch::build_quads(&surface, &mat);

        BezierPatch {
            surface,
            mat,
            bbox,
            debug: false,
            quads,
        }
    }

    fn build_quads(surface: &BezierSurface, mat: &Arc<dyn Material + Sync + Send>) -> QuadHitList {
        let mut quads: Vec<Quad> = vec![];

        let mut last_curve: Option<&BezierCurve3D> = None;
        for curve in surface.get_curves() {
            if last_curve.is_some() {
                for iter in 1..curve.points().len() {
                    let point_0 = curve.points()[iter - 1];
                    let _point_1 = curve.points()[iter];
                    let last_point_0 = last_curve.unwrap().points()[iter - 1];
                    let last_point_1 = last_curve.unwrap().points()[iter];
                    let u = Vec3::from_point(point_0 - last_point_0);
                    let v = Vec3::from_point(last_point_1 - last_point_0);
                    //area += u.cross(&v).length();
                    quads.push(Quad::new(
                        Vec3::from_point(last_point_0),
                        u,
                        v,
                        Arc::clone(&mat),
                    ));
                }
            }
            last_curve = Some(curve);
        }

        QuadHitList {
            quads,
            count: [
                surface.get_curves().len() - 1,
                surface.get_curves()[0].points().len() - 1,
            ],
        }
    }
}

struct QuadHitList {
    quads: Vec<Quad>,
    count: [usize; 2],
}

impl QuadHitList {
    pub fn hit(&self, r: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        let mut temp_record = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = time.max();

        for quad in &self.quads {
            if quad.hit(
                r,
                Interval::new(time.min(), closest_so_far),
                &mut temp_record,
            ) {
                hit_anything = true;
                closest_so_far = temp_record.time;
                hit_record.from(&temp_record);
            }
        }

        hit_anything
    }

    pub fn test_hit(&self, r: &Ray3, time: &Interval) -> Point2D {
        let mut temp_record = HitRecord::new();
        let mut closest_uv = [0.5, 0.5];
        let mut closest_so_far = time.max();

        let x_mult = 1.0 / self.count[0] as f64;
        let y_mult = 1.0 / self.count[1] as f64;

        for iter in 0..self.quads.len() {
            let quad = &self.quads[iter];
            if quad.hit(
                r,
                Interval::new(time.min(), closest_so_far),
                &mut temp_record,
            ) {
                closest_so_far = temp_record.time;
                closest_uv = quad.uv(temp_record.point);
                let x = (iter % self.count[0]) as f64;
                let y = (iter / self.count[0]) as f64;
                closest_uv = [(closest_uv[0] + x) * x_mult, (closest_uv[1] + y) * y_mult];
            }
        }

        Point2D::new(closest_uv)
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f64 {
        let weight = 1.0 / (self.quads.len() as f64);
        let mut sum = 0.0;
        for quad in &self.quads {
            sum += weight * quad.pdf_value(origin, direction);
        }
        sum
    }
}

// TODO - Tests
// Add tests for BezierPatch::new
