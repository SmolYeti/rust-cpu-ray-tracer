use core::f64;
use std::sync::Arc;

use crate::aabb::AABB;
use crate::hittable::HitRecord;
use crate::hittable::Hittable;
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray3;
use crate::triangle::Triangle;
use nurbs::bezier_curve::BezierCurve3D;
use nurbs::bezier_surface::BezierSurface;
use nurbs::point_types::Point2D;
use nurbs::point_types::Point4D;
use nurbs::surface::Surface;
use nurbs::vector_3::Vec3;

pub struct BezierPatch {
    surface: BezierSurface,
    mat: Arc<dyn Material + Sync + Send>,
    bbox: AABB,
    debug: bool,
    tris: TriHitList,
    aabbs: AABBHitList,
}

impl Hittable for BezierPatch {
    fn hit(&self, ray_in: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        if self.debug {
            self.tris.hit(ray_in, time, hit_record)
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
            let j_inv = |uv: Point2D| -> Option<Point4D> {
                let su = Vec3::from_point(self.surface.partial_derivative_u(uv));
                let sv = Vec3::from_point(self.surface.partial_derivative_v(uv));
                let j = Point4D::new([n1.dot(&su), n1.dot(&sv), n2.dot(&su), n2.dot(&sv)]);
                let det_j = (j.x() * j.w()) - (j.y() * j.z());
                //if det_j < f64::EPSILON as f64 {
                //    return None;
                //} else {
                Some(Point4D::new([j.w(), -j.y(), -j.z(), j.z()]) / det_j)
                //}
            };

            let interval = Interval::new(0.0, 1.0);

            // let u0 = inital_guess;
            let mut uv = self.aabbs.test_hit(&ray_in, &time);
            let mut n: u32 = 0;
            let max_iter: u32 = 50;
            let mut error: f64 = 10000.0;
            let tolerance: f64 = 0.001;
            let mut last_error = 10001.0;
            while n < max_iter && error > tolerance && error < last_error {
                let j_inv_result = j_inv(uv);
                let inv = match j_inv_result {
                    Some(matrix) => matrix,
                    None => break,
                };

                let fun = f(uv);
                uv = uv
                    - Point2D::new([
                        inv.x() * fun.x() + inv.y() * fun.y(),
                        inv.z() * fun.x() + inv.w() * fun.y(),
                    ]);
                uv = Point2D::new([interval.clamp(uv.u()), interval.clamp(uv.v())]);
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
        self.tris.pdf_value(origin, direction)
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

        let tris = BezierPatch::build_tris(&surface, &mat);
        let aabbs = BezierPatch::build_aabbs(&surface);

        BezierPatch {
            surface,
            mat,
            bbox,
            debug: false,
            tris,
            aabbs,
        }
    }

    fn build_tris(surface: &BezierSurface, mat: &Arc<dyn Material + Sync + Send>) -> TriHitList {
        let mut tris: Vec<Triangle> = vec![];

        let mut last_curve: Option<&BezierCurve3D> = None;
        for curve_iter in 0..surface.get_curves().len() {
            let curve = &surface.get_curves()[curve_iter];
            if last_curve.is_some() {
                for iter in 1..curve.points().len() {
                    let point_0 = Vec3::from_point(curve.points()[iter - 1]);
                    let point_1 = Vec3::from_point(curve.points()[iter]);
                    let last_point_0 = Vec3::from_point(last_curve.unwrap().points()[iter - 1]);
                    let last_point_1 = Vec3::from_point(last_curve.unwrap().points()[iter]);
                    tris.push(Triangle::new(
                        last_point_0,
                        last_point_1,
                        point_0,
                        Arc::clone(&mat),
                    ));
                    tris.push(Triangle::new(
                        point_1,
                        point_0,
                        last_point_1,
                        Arc::clone(&mat),
                    ));
                }
            }
            last_curve = Some(&curve);
        }

        TriHitList { tris }
    }

    fn build_aabbs(surface: &BezierSurface) -> AABBHitList {
        let mut aabbs: Vec<AABB> = vec![];
        let mut uvs: Vec<Point2D> = vec![];

        let samples = 5;
        let u_div = 1.0 / samples as f64;
        let v_div = 1.0 / samples as f64;

        let points = surface.evaluate_points(samples, samples);

        for u in 0..samples - 1 {
            for v in 0..samples - 1 {
                let v_start = u * samples;
                let aabb_0 = AABB::from_vec3s(
                    Vec3::from_point(points[v_start + v]),
                    Vec3::from_point(points[v_start + v + 1]),
                );
                let aabb_1 = AABB::from_vec3s(
                    Vec3::from_point(points[v_start + v + samples]),
                    Vec3::from_point(points[v_start + v + 1 + samples]),
                );
                let aabb = AABB::from_aabbs(&aabb_0, &aabb_1);

                let center_uv = Point2D::new([(u as f64 + 0.5) * u_div, (v as f64 + 0.5) * v_div]);
                let center_point = surface.evaluate(center_uv);
                let aabb_center = AABB::from_vec3s(
                    Vec3::from_point(center_point),
                    Vec3::from_point(center_point),
                );
                let aabb = AABB::from_aabbs(&aabb, &aabb_center).expand(0.0001);

                aabbs.push(aabb);
                uvs.push(center_uv);
            }
        }

        AABBHitList { aabbs, uvs }
    }
}

struct TriHitList {
    tris: Vec<Triangle>,
}

impl TriHitList {
    pub fn hit(&self, r: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        let mut temp_record = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = time.max();

        for tri in &self.tris {
            if tri.hit(
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

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f64 {
        let weight = 1.0 / (self.tris.len() as f64);
        let mut sum = 0.0;
        for tri in &self.tris {
            sum += weight * tri.pdf_value(origin, direction);
        }
        sum
    }
}

struct AABBHitList {
    aabbs: Vec<AABB>,
    uvs: Vec<Point2D>,
}

impl AABBHitList {
    pub fn test_hit(&self, r: &Ray3, time: &Interval) -> Point2D {
        let mut closest_uv = Point2D::new([0.5, 0.5]);
        let mut closest_so_far = time.max();

        for iter in 0..self.aabbs.len() {
            let bbox = &self.aabbs[iter];
            if bbox.hit(r, Interval::new(time.min(), closest_so_far)) {
                let bbox_dir = bbox.center() - r.origin();
                closest_so_far = bbox_dir.dot(&bbox_dir);
                closest_uv = self.uvs[iter];
            }
        }

        closest_uv
    }
}

// TODO - Tests
// Add tests for BezierPatch::new
