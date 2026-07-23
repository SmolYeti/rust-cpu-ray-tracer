use crate::bezier_curve::BezierCurve3D;
use crate::bezier_curve::all_bernstein;
use crate::curve::Curve3D;
use crate::interval::Interval;
use crate::point_types::Point2D;
use crate::point_types::Point3D;
use crate::surface::Surface;

pub struct BezierSurface {
    curves: Vec<BezierCurve3D>,
    u_interval: Interval,
    v_interval: Interval,
}

impl BezierSurface {
    pub fn new(
        curves: Vec<BezierCurve3D>,
        u_interval: Interval,
        v_interval: Interval,
    ) -> BezierSurface {
        BezierSurface {
            curves,
            u_interval,
            v_interval,
        }
    }

    pub fn from_curves(curves: Vec<BezierCurve3D>) -> BezierSurface {
        let interval = Interval::new(Point2D::new([0.0, 1.0]));
        BezierSurface::new(curves, interval.clone(), interval)
    }

    pub fn from_points(
        points: Vec<Point3D>,
        curve_length: usize,
        u_interval: Interval,
        v_interval: Interval,
    ) -> BezierSurface {
        let curve_count = points.len() / curve_length;
        let mut curves: Vec<BezierCurve3D> = Vec::with_capacity(curve_count);

        for curve in 0..curve_count {
            let mut control_points = Vec::with_capacity(curve_length);
            for point in 0..curve_length {
                let index = curve * curve_length + point;
                control_points.push(points[index]);
            }
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }

        BezierSurface {
            curves,
            u_interval,
            v_interval,
        }
    }

    pub fn get_curves(&self) -> &Vec<BezierCurve3D> {
        &self.curves
    }

    pub fn partial_derivative_u(&self, uv: Point2D) -> Point3D {
        let m = self.curves[0].points().len() - 1;
        let n = self.curves.len() - 1;

        let u_bern = all_bernstein(m - 1, uv.u());
        let v_bern = all_bernstein(n, uv.v());

        let mut sum = Point3D::empty();
        for j in 0..(n + 1) {
            let curve = &self.curves[j];
            for i in 0..m {
                sum += (curve.points()[i + 1] - curve.points()[i]) * u_bern[i] * v_bern[j];
            }
        }

        m as f64 * sum
    }

    pub fn partial_derivative_v(&self, uv: Point2D) -> Point3D {
        let m = self.curves[0].points().len() - 1;
        let n = self.curves.len() - 1;

        let u_bern = all_bernstein(m, uv.u());
        let v_bern = all_bernstein(n - 1, uv.v());

        let mut sum = Point3D::empty();
        for j in 0..n {
            let curve_0 = &self.curves[j];
            let curve_1 = &self.curves[j + 1];
            for i in 0..(m + 1) {
                sum += (curve_1.points()[i] - curve_0.points()[i]) * u_bern[i] * v_bern[j];
            }
        }

        n as f64 * sum
    }
}

impl Surface for BezierSurface {
    fn interval_u(&self) -> &Interval {
        &self.u_interval
    }

    fn interval_v(&self) -> &Interval {
        &self.v_interval
    }

    fn evaluate(&self, uv: Point2D) -> Point3D {
        let mut control_points = Vec::with_capacity(self.curves.len());
        for curve in 0..self.curves.len() {
            control_points.push(self.curves[curve].evaluate(uv.x()));
        }
        let bezier_curve = BezierCurve3D::new(control_points, self.interval_v().clone());
        bezier_curve.evaluate(uv.y())
    }

    fn evaluate_points(&self, u_sample_count: usize, v_sample_count: usize) -> Vec<Point3D> {
        let mut points = Vec::<Point3D>::with_capacity(u_sample_count * v_sample_count);
        let u_div =
            (self.interval_u().max() - self.interval_u().min()) / ((u_sample_count as f64) - 1.0);
        let v_div =
            (self.interval_v().max() - self.interval_v().min()) / ((v_sample_count as f64) - 1.0);

        for i in 0..u_sample_count {
            let u = self.interval_u().min() + (i as f64 * u_div);
            let mut control_points = Vec::with_capacity(self.curves.len());
            for curve in 0..self.curves.len() {
                control_points.push(self.curves[curve].evaluate(u));
            }
            let bezier_curve = BezierCurve3D::new(control_points, self.interval_v().clone());

            for j in 0..v_sample_count {
                let v = self.interval_v().min() + (j as f64 * v_div);
                points.push(bezier_curve.evaluate(v));
            }
        }

        points
    }
}

#[cfg(test)]
mod tests {
    use crate::bezier_curve::BezierCurve3D;
    use crate::bezier_surface::BezierSurface;
    use crate::curve::Curve3D;
    use crate::interval::Interval;
    use crate::point_types::Point2D;
    use crate::point_types::Point3D;
    use crate::surface::Surface;
    use crate::utility::f64_equal;

    #[test]
    fn test_construct() {
        let curves: Vec<BezierCurve3D> = Vec::new();

        let bezier = BezierSurface::from_curves(curves);

        assert!(f64_equal(bezier.interval_u().min(), 0.0));
        assert!(f64_equal(bezier.interval_u().max(), 1.0));
        assert!(f64_equal(bezier.interval_v().min(), 0.0));
        assert!(f64_equal(bezier.interval_v().max(), 1.0));
    }

    #[test]
    fn test_construct_from_points() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
            Point3D::new([0.0, 0.0, 2.0]),
            Point3D::new([1.0, 0.0, 3.0]),
            Point3D::new([1.0, 1.0, 4.0]),
            Point3D::new([0.0, 1.0, 5.0]),
            Point3D::new([0.0, 0.0, 3.0]),
            Point3D::new([1.0, 0.0, 4.0]),
            Point3D::new([1.0, 1.0, 5.0]),
            Point3D::new([0.0, 1.0, 6.0]),
        ];

        let interval = Interval::new(Point2D::new([0.0, 1.0]));

        let bezier = BezierSurface::from_points(control_points, 4, interval.clone(), interval);

        assert!(f64_equal(bezier.interval_u().min(), 0.0));
        assert!(f64_equal(bezier.interval_u().max(), 1.0));
        assert!(f64_equal(bezier.interval_v().min(), 0.0));
        assert!(f64_equal(bezier.interval_v().max(), 1.0));

        assert_eq!(bezier.get_curves().len(), 3);
    }

    #[test]
    fn test_point() {
        let mut curves: Vec<BezierCurve3D> = Vec::with_capacity(4);
        {
            let control_points = vec![
                Point3D::new([0.0, -1.0, 0.0]),
                Point3D::new([0.0, 2.0, 1.0]),
                Point3D::new([0.0, 2.0, 2.0]),
                Point3D::new([0.0, 1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }
        {
            let control_points = vec![
                Point3D::new([1.0, 0.0, 0.0]),
                Point3D::new([1.0, 4.0, 1.0]),
                Point3D::new([1.0, 3.0, 2.0]),
                Point3D::new([1.0, 2.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }
        {
            let control_points = vec![
                Point3D::new([2.0, 2.0, 0.0]),
                Point3D::new([2.0, 1.0, 1.0]),
                Point3D::new([2.0, 0.0, 2.0]),
                Point3D::new([2.0, -1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }
        {
            let control_points = vec![
                Point3D::new([3.0, 3.0, 0.0]),
                Point3D::new([3.0, -2.0, 1.0]),
                Point3D::new([3.0, -4.0, 2.0]),
                Point3D::new([3.0, 0.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }

        let bezier = BezierSurface::from_curves(curves.clone());
        let point = bezier.evaluate(Point2D::new([0.5, 0.5]));

        let temp_cps = vec![
            curves[0].evaluate(0.5),
            curves[1].evaluate(0.5),
            curves[2].evaluate(0.5),
            curves[3].evaluate(0.5),
        ];
        let temp_curve = BezierCurve3D::from_points(temp_cps);
        let test_pt = temp_curve.evaluate(0.5);

        assert!(f64_equal(point.x(), test_pt.x()));
        assert!(f64_equal(point.y(), test_pt.y()));
        assert!(f64_equal(point.z(), test_pt.z()));
    }

    #[test]
    fn test_point_interval() {
        let u_interval = Interval::new(Point2D::new([0.0, 10.0]));
        let v_interval = Interval::new(Point2D::new([-12.0, -10.0]));
        let u_mid = u_interval.mid();
        let v_mid = v_interval.mid();
        let mut curves: Vec<BezierCurve3D> = Vec::with_capacity(4);
        {
            let control_points = vec![
                Point3D::new([0.0, -1.0, 0.0]),
                Point3D::new([0.0, 2.0, 1.0]),
                Point3D::new([0.0, 2.0, 2.0]),
                Point3D::new([0.0, 1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }
        {
            let control_points = vec![
                Point3D::new([1.0, 0.0, 0.0]),
                Point3D::new([1.0, 4.0, 1.0]),
                Point3D::new([1.0, 3.0, 2.0]),
                Point3D::new([1.0, 2.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }
        {
            let control_points = vec![
                Point3D::new([2.0, 2.0, 0.0]),
                Point3D::new([2.0, 1.0, 1.0]),
                Point3D::new([2.0, 0.0, 2.0]),
                Point3D::new([2.0, -1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }
        {
            let control_points = vec![
                Point3D::new([3.0, 3.0, 0.0]),
                Point3D::new([3.0, -2.0, 1.0]),
                Point3D::new([3.0, -4.0, 2.0]),
                Point3D::new([3.0, 0.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }

        let bezier = BezierSurface::new(curves.clone(), u_interval, v_interval);
        let point = bezier.evaluate(Point2D::new([u_mid, v_mid]));

        let temp_cps = vec![
            curves[0].evaluate(u_mid),
            curves[1].evaluate(u_mid),
            curves[2].evaluate(u_mid),
            curves[3].evaluate(u_mid),
        ];
        let temp_curve = BezierCurve3D::new(temp_cps, v_interval);
        let test_pt = temp_curve.evaluate(v_mid);

        assert!(f64_equal(point.x(), test_pt.x()));
        assert!(f64_equal(point.y(), test_pt.y()));
        assert!(f64_equal(point.z(), test_pt.z()));
    }

    #[test]
    fn test_points() {
        let point_count = 100;
        let div = 1.0 / ((point_count - 1) as f64);
        let mut curves: Vec<BezierCurve3D> = Vec::with_capacity(4);
        {
            let control_points = vec![
                Point3D::new([0.0, -1.0, 0.0]),
                Point3D::new([0.0, 2.0, 1.0]),
                Point3D::new([0.0, 2.0, 2.0]),
                Point3D::new([0.0, 1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }
        {
            let control_points = vec![
                Point3D::new([1.0, 0.0, 0.0]),
                Point3D::new([1.0, 4.0, 1.0]),
                Point3D::new([1.0, 3.0, 2.0]),
                Point3D::new([1.0, 2.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }
        {
            let control_points = vec![
                Point3D::new([2.0, 2.0, 0.0]),
                Point3D::new([2.0, 1.0, 1.0]),
                Point3D::new([2.0, 0.0, 2.0]),
                Point3D::new([2.0, -1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }
        {
            let control_points = vec![
                Point3D::new([3.0, 3.0, 0.0]),
                Point3D::new([3.0, -2.0, 1.0]),
                Point3D::new([3.0, -4.0, 2.0]),
                Point3D::new([3.0, 0.0, 3.0]),
            ];
            curves.push(BezierCurve3D::from_points(control_points));
        }

        let bezier = BezierSurface::from_curves(curves.clone());
        let points = bezier.evaluate_points(point_count, point_count);

        for i in 0..point_count {
            let u = i as f64 * div;
            let temp_cps = vec![
                curves[0].evaluate(u),
                curves[1].evaluate(u),
                curves[2].evaluate(u),
                curves[3].evaluate(u),
            ];
            let temp_curve = BezierCurve3D::from_points(temp_cps);
            for j in 0..point_count {
                let v = j as f64 * div;
                let test_pt = temp_curve.evaluate(v);
                let point = points[i * point_count + j];

                assert!(f64_equal(point.x(), test_pt.x()));
                assert!(f64_equal(point.y(), test_pt.y()));
                assert!(f64_equal(point.z(), test_pt.z()));
            }
        }
    }

    #[test]
    fn test_points_interval() {
        let u_interval = Interval::new(Point2D::new([7.0, 10.0]));
        let v_interval = Interval::new(Point2D::new([-12.0, 20.0]));
        let point_count = 100;
        let div = 1.0 / ((point_count - 1) as f64);
        let u_div = div * (u_interval.max() - u_interval.min());
        let v_div = div * (v_interval.max() - v_interval.min());
        let mut curves: Vec<BezierCurve3D> = Vec::with_capacity(4);
        {
            let control_points = vec![
                Point3D::new([0.0, -1.0, 0.0]),
                Point3D::new([0.0, 2.0, 1.0]),
                Point3D::new([0.0, 2.0, 2.0]),
                Point3D::new([0.0, 1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }
        {
            let control_points = vec![
                Point3D::new([1.0, 0.0, 0.0]),
                Point3D::new([1.0, 4.0, 1.0]),
                Point3D::new([1.0, 3.0, 2.0]),
                Point3D::new([1.0, 2.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }
        {
            let control_points = vec![
                Point3D::new([2.0, 2.0, 0.0]),
                Point3D::new([2.0, 1.0, 1.0]),
                Point3D::new([2.0, 0.0, 2.0]),
                Point3D::new([2.0, -1.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }
        {
            let control_points = vec![
                Point3D::new([3.0, 3.0, 0.0]),
                Point3D::new([3.0, -2.0, 1.0]),
                Point3D::new([3.0, -4.0, 2.0]),
                Point3D::new([3.0, 0.0, 3.0]),
            ];
            curves.push(BezierCurve3D::new(control_points, u_interval));
        }

        let bezier = BezierSurface::new(curves.clone(), u_interval, v_interval);
        let points = bezier.evaluate_points(point_count, point_count);

        for i in 0..point_count {
            let u = (i as f64 * u_div) + u_interval.min();
            let temp_cps = vec![
                curves[0].evaluate(u),
                curves[1].evaluate(u),
                curves[2].evaluate(u),
                curves[3].evaluate(u),
            ];
            let temp_curve = BezierCurve3D::new(temp_cps, v_interval);
            for j in 0..point_count {
                let v = (j as f64 * v_div) + v_interval.min();
                let test_pt = temp_curve.evaluate(v);
                let point = points[i * point_count + j];

                assert!(
                    f64_equal(point.x(), test_pt.x()),
                    "<{}, {}> Actual {} vs {}",
                    i,
                    j,
                    point.x(),
                    test_pt.x()
                );
                assert!(
                    f64_equal(point.y(), test_pt.y()),
                    "<{}, {}> Actual {} vs {}",
                    i,
                    j,
                    point.y(),
                    test_pt.y()
                );
                assert!(
                    f64_equal(point.z(), test_pt.z()),
                    "<{}, {}> Actual {} vs {}",
                    i,
                    j,
                    point.z(),
                    test_pt.z()
                );
            }
        }
    }
}
