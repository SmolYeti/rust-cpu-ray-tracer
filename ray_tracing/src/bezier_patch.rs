use core::f64;
use std::sync::Arc;

use crate::aabb::AABB;
use crate::bvh_node::BVHNode;
use crate::hittable::HitRecord;
use crate::hittable::Hittable;
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray3;
use crate::triangle::Triangle;
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
}

impl Hittable for BezierPatch {
    fn hit(&self, ray_in: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        let hit_surf = self
            .tris
            .hit(ray_in, Interval::new(time.min(), time.max()), hit_record);
        if self.debug || !hit_surf {
            hit_surf
        } else {
            // Intersection code based on Ray Tracing Bezier Surfaces on the GPU by Joakim Low
            let o = ray_in.origin();
            let d = ray_in.direction().unit_vector();
            let n1 = if d.x.abs() > d.y.abs() && d.x.abs() > d.z.abs() {
                Vec3::new(d.y, -d.x, 0.0)
            } else {
                Vec3::new(0.0, d.z, -d.y)
            };
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
                if det_j.abs() < f64::EPSILON as f64 {
                    return None;
                } else {
                    Some(Point4D::new([j.w(), -j.y(), -j.z(), j.x()]) / det_j)
                }
            };

            let interval = Interval::new(0.0, 1.0);

            // let u0 = inital_guess;
            let mut uv = Point2D::new([hit_record.u, hit_record.v]);
            let mut n: u32 = 0;
            let max_iter: u32 = 50;
            let mut error: f64 = 10000.0;
            let tolerance: f64 = 0.001;
            let mut last_error = 10001.0;
            while n < max_iter && error > tolerance && error < last_error {
                let inv = match j_inv(uv) {
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
                error = fun1.dot(fun1).abs();
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
                true
            } else {
                false
            }
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
        let tri_samples = 5;
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

        let tris = BezierPatch::build_tris(&surface, &mat, tri_samples);

        BezierPatch {
            surface,
            mat,
            bbox,
            debug: true,
            tris,
        }
    }

    fn build_tris(
        surface: &BezierSurface,
        mat: &Arc<dyn Material + Sync + Send>,
        samples: usize,
    ) -> TriHitList {
        let mut tri_surface = HittableList::new();

        let points = surface.evaluate_points(samples, samples);

        let u_div = 1.0 / (samples - 1) as f64;
        let v_div = 1.0 / (samples - 1) as f64;

        for u in 0..samples - 1 {
            for v in 0..samples - 1 {
                let v_start = u * samples;
                let point_a = Vec3::from_point(points[v_start + v]);
                let point_b = Vec3::from_point(points[v_start + v + 1]);
                let point_c = Vec3::from_point(points[v_start + v + samples]);
                let point_d = Vec3::from_point(points[v_start + v + 1 + samples]);
                let uv_a = [u as f64 * u_div, v as f64 * v_div];
                let uv_b = [u as f64 * u_div, (v + 1) as f64 * v_div];
                let uv_c = [(u + 1) as f64 * u_div, v as f64 * v_div];
                let uv_d = [(u + 1) as f64 * u_div, (v + 1) as f64 * v_div];

                tri_surface.add(Arc::new(Triangle::new(
                    point_a,
                    point_b,
                    point_c,
                    [uv_a[0], uv_a[1], uv_b[0], uv_b[1], uv_c[0], uv_c[1]],
                    Arc::clone(&mat),
                )));
                tri_surface.add(Arc::new(Triangle::new(
                    point_d,
                    point_c,
                    point_b,
                    [uv_d[0], uv_d[1], uv_c[0], uv_c[1], uv_b[0], uv_b[1]],
                    Arc::clone(&mat),
                )));
            }
        }
        let tris = BVHNode::from_list(&tri_surface);

        TriHitList { tris }
    }
}

struct TriHitList {
    tris: BVHNode,
}

impl TriHitList {
    pub fn hit(&self, r: &Ray3, time: Interval, hit_record: &mut HitRecord) -> bool {
        self.tris.hit(r, time, hit_record)
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f64 {
        self.tris.pdf_value(origin, direction)
    }
}
// TODO - Tests
// Add tests for BezierPatch::new
