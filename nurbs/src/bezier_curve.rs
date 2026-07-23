use crate::curve::Curve2D;
use crate::curve::Curve3D;
use crate::interval::Interval;
use crate::point_types::Point;
use crate::point_types::Point2D;
use crate::point_types::Point3D;

#[derive(Clone)]
pub struct BezierCurveND<const N: usize> {
    control_points: Vec<Point<N>>,
    curve_interval: Interval,
}

pub type BezierCurve2D = BezierCurveND<2>;
pub type BezierCurve3D = BezierCurveND<3>;

// Returns:
// The value of the Bernstein polynomials of i, n - 1 at the value of u
// B(i,n - 1) (u)
// Parameters:
// i - index of berstein polynomial
// n - degree of berstein polynomial + 1
// u - input value to berstain polynomial
pub fn bernstein(index: usize, n: usize, u: f64) -> f64 {
    let mut temp = vec![0.0; n + 1];
    temp[n - index] = 1.0;
    let u_inv = 1.0 - u;
    for i in 1..(n + 1) {
        let mut j = n;
        while j >= i {
            temp[j] = (u_inv * temp[j]) + (u * temp[j - 1]);
            j -= 1;
        }
    }
    temp[n]
}

// Returns:
// A vector with the values of the Bernstein polynomials of degree n at the
// fixed value of u B(0,n - 1) (u) to B(n - 1,n - 1)(u) Parameters: n - degree
// of berstein polynomial + 1 u - input value to berstain polynomial
pub fn all_bernstein(n: usize, u: f64) -> Vec<f64> {
    let mut bernstein = vec![0.0; n + 1];
    bernstein[0] = 1.0;
    let u_inv = 1.0 - u;

    for i in 1..(n + 1) {
        let mut saved = 0.0;
        for j in 0..i {
            let temp = bernstein[j];
            bernstein[j] = saved + (u_inv * temp);
            saved = u * temp;
        }
        bernstein[i] = saved;
    }

    bernstein
}

impl<const N: usize> BezierCurveND<N> {
    pub fn new(control_points: Vec<Point<N>>, curve_interval: Interval) -> BezierCurveND<N> {
        BezierCurveND {
            control_points,
            curve_interval,
        }
    }

    pub fn points(&self) -> &Vec<Point<N>> {
        &self.control_points
    }

    pub fn from_points(control_points: Vec<Point<N>>) -> BezierCurveND<N> {
        BezierCurveND::new(control_points, Interval::new(Point2D::new([0.0, 1.0])))
    }

    // NURBS Algorithm A1.5 DeCasteljau1, p24
    fn de_casteljau(&self, u: f64) -> Point<N> {
        let mut points = self.control_points.clone();
        let u_inv = 1.0 - u;

        let n = points.len();
        for i in 1..n {
            for j in 0..(n - i) {
                points[j] = (u_inv * points[j]) + (u * points[j + 1]);
            }
        }

        points[0]
    }

    // NURBS Algorithm A1.4 PointOnBezierCurv, p22
    fn _point_on_curve(&self, param: f64) -> Point<N> {
        let u = self.curve_interval.localize_clamp_value(param);
        let mut point = Point::<N>::empty();
        let bernstein = all_bernstein(self.control_points.len() - 1, u);

        for i in 0..self.control_points.len() {
            point += bernstein[i] * self.control_points[i];
        }

        point
    }

    // Chaper 1, Equation 1.9 derivative of a Bezier curve, p22
    // This is probably wrong
    pub fn derivative(&self, param: f64) -> Point<N> {
        let u = self.curve_interval.localize_clamp_value(param);
        let bernstein = all_bernstein(self.control_points.len() - 2, u);
        let mut deriv = Point::<N>::empty();

        for i in 0..bernstein.len() {
            deriv += bernstein[i] * (self.control_points[i + 1] - self.control_points[i]);
        }

        deriv *= bernstein.len() as f64;

        deriv
    }
}

impl Curve2D for BezierCurve2D {
    fn interval(&self) -> &Interval {
        &self.curve_interval
    }

    fn evaluate(&self, parameter: f64) -> Point2D {
        let u = self.interval().localize_clamp_value(parameter);
        self.de_casteljau(u)
    }
}

impl Curve3D for BezierCurve3D {
    fn interval(&self) -> &Interval {
        &self.curve_interval
    }

    fn evaluate(&self, parameter: f64) -> Point3D {
        let u = self.interval().localize_clamp_value(parameter);
        self.de_casteljau(u)
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::bezier_curve::BezierCurve2D;
    use crate::bezier_curve::BezierCurve3D;
    use crate::bezier_curve::all_bernstein;
    use crate::bezier_curve::bernstein;
    use crate::curve::Curve2D;
    use crate::curve::Curve3D;
    use crate::interval::Interval;
    use crate::point_types::Point2D;
    use crate::point_types::Point3D;
    use crate::utility::f64_equal;
    use crate::utility::f64_near;

    #[test]
    fn test_berstein_vs_all() {
        let div = 1.0 / 99.0;
        for i in 0..100 {
            let u = i as f64 * div;
            let all_bern = all_bernstein(2, u);
            let bern_0 = bernstein(0, 2, u);
            let bern_1 = bernstein(1, 2, u);
            let bern_2 = bernstein(2, 2, u);
            assert!(
                f64_equal(all_bern[0], bern_0),
                "Actual {} vs {}",
                all_bern[0],
                bern_0
            );
            assert!(
                f64_equal(all_bern[1], bern_1),
                "Actual {} vs {}",
                all_bern[1],
                bern_1
            );
            assert!(
                f64_equal(all_bern[2], bern_2),
                "Actual {} vs {}",
                all_bern[2],
                bern_2
            );
        }
    }

    #[test]
    fn test_berstein_parition_of_unity() {
        let div = 1.0 / 99.0;
        for n in 1..5 {
            for i in 0..100 {
                let u = i as f64 * div;
                let all_bern = all_bernstein(n, u);
                let mut sum = 0.0;
                for bern in all_bern {
                    sum += bern;
                }
                assert!(f64_near(sum, 1.0, f64::EPSILON * 10.0), "Actual {}", sum);
            }
        }
    }

    #[test]
    fn test_berstein_start_end() {
        let start = 0.0;
        let end = 1.0;
        for n in 1..10 {
            let bern_start = bernstein(0, n, start);
            let bern_end = bernstein(n, n, end);
            assert!(f64_equal(bern_start, 1.0), "Actual {}", bern_start);
            assert!(f64_equal(bern_end, 1.0), "Actual {}", bern_end);
        }
    }

    #[test]
    fn test_berstein_max() {
        let div = 1.0 / 99.0;
        for n in 1..5 {
            let mut maxes = Vec::with_capacity(n);
            let mut u_vals = Vec::with_capacity(n);

            // Build the max bernstein list
            let n_1 = 1.0 / (n as f64);
            for i in 0..n {
                u_vals.push(i as f64 * n_1);
                maxes.push(bernstein(i, n, u_vals[i]));
            }

            // Check values against max
            for i in 0..100 {
                let u = i as f64 * div;
                let all_bern = all_bernstein(n, u);
                for j in 0..n {
                    if (u_vals[j] - u).abs() < f64::EPSILON {
                        assert!(f64_equal(all_bern[j], maxes[j]));
                    } else {
                        assert!(all_bern[j] < maxes[j]);
                    }
                }
            }
        }
    }

    #[test]
    fn test_2d_construct() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([0.0, 1.0]));

        let bezier = BezierCurve2D::from_points(control_points);

        assert!(f64_equal(bezier.interval().min(), 0.0));
        assert!(f64_equal(bezier.interval().max(), 1.0));
    }

    #[test]
    fn test_2d_point() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([0.0, 1.0]));

        let bezier = BezierCurve2D::from_points(control_points.clone());

        let point = bezier.evaluate(0.5);

        let mut test_points = control_points;
        while test_points.len() > 1 {
            let mut temp_points = Vec::new();

            for i in 1..test_points.len() {
                temp_points.push((test_points[i - 1] + test_points[i]) * 0.5);
            }

            test_points = temp_points;
        }

        assert!(
            f64_equal(test_points[0].x(), point.x()),
            "Actual {} vs {}",
            test_points[0].x(),
            point.x()
        );
        assert!(
            f64_equal(test_points[0].y(), point.y()),
            "Actual {} vs {}",
            test_points[0].y(),
            point.y()
        );
    }

    #[test]
    fn test_2d_point_interval() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([0.0, 1.0]));

        let bezier = BezierCurve2D::new(
            control_points.clone(),
            Interval::from_values(0.0, 10.0),
        );

        let point = bezier.evaluate(5.0);

        let mut test_points = control_points;
        while test_points.len() > 1 {
            let mut temp_points = Vec::new();

            for i in 1..test_points.len() {
                temp_points.push((test_points[i - 1] + test_points[i]) * 0.5);
            }

            test_points = temp_points;
        }

        assert!(
            f64_equal(test_points[0].x(), point.x()),
            "Actual {} vs {}",
            test_points[0].x(),
            point.x()
        );
        assert!(
            f64_equal(test_points[0].y(), point.y()),
            "Actual {} vs {}",
            test_points[0].y(),
            point.y()
        );
    }

    #[test]
    fn test_2d_points() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([0.0, 1.0]));

        let bezier = BezierCurve2D::from_points(control_points.clone());

        let points = bezier.evaluate_points(100);

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let u = i as f64 * div;
            let u_inv = 1.0 - u;
            let mut test_points = control_points.clone();
            while test_points.len() > 1 {
                let mut temp_points = Vec::new();

                for i in 1..test_points.len() {
                    temp_points.push((u_inv * test_points[i - 1]) + (u * test_points[i]));
                }

                test_points = temp_points;
            }

            assert!(
                f64_equal(test_points[0].x(), points[i].x()),
                "Actual {} vs {}",
                test_points[0].x(),
                points[i].x()
            );
            assert!(
                f64_equal(test_points[0].y(), points[i].y()),
                "Actual {} vs {}",
                test_points[0].y(),
                points[i].y()
            );
        }
    }

    #[test]
    fn test_2d_points_interval() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([0.0, 1.0]));

        let bezier = BezierCurve2D::new(control_points.clone(), Interval::from_values(0.0, 10.0));

        let points = bezier.evaluate_points(100);

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let u = i as f64 * div;
            let u_inv = 1.0 - u;
            let mut test_points = control_points.clone();
            while test_points.len() > 1 {
                let mut temp_points = Vec::new();

                for i in 1..test_points.len() {
                    temp_points.push((u_inv * test_points[i - 1]) + (u * test_points[i]));
                }

                test_points = temp_points;
            }

            assert!(
                f64_near(test_points[0].x(), points[i].x(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                test_points[0].x(),
                points[i].x()
            );
            assert!(
                f64_near(test_points[0].y(), points[i].y(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                test_points[0].y(),
                points[i].y()
            );
        }
    }

    #[test]
    fn test_2d_end_derivatives() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([2.0, 1.0]));
        control_points.push(Point2D::new([3.0, 0.0]));

        let bezier = BezierCurve2D::from_points(control_points.clone());

        let start = bezier.derivative(0.0);
        let end = bezier.derivative(1.0);

        let n = (control_points.len() - 1) as f64;

        // The start derivative should be n(P1 - P0)
        let start_calc = n * (control_points[1] - control_points[0]);
        assert!(
            f64_equal(start_calc.x(), start.x()),
            "Actual: {} vs {}",
            start_calc.x(),
            start.x()
        );
        assert!(
            f64_equal(start_calc.y(), start.y()),
            "Actual: {} vs {}",
            start_calc.y(),
            start.y()
        );

        // The end derivative should be n(Pn - Pn-1)
        let end_calc = n * (control_points[3] - control_points[2]);
        assert!(
            f64_equal(end_calc.x(), end.x()),
            "Actual: {} vs {}",
            end_calc.x(),
            end.x()
        );
        assert!(
            f64_equal(end_calc.y(), end.y()),
            "Actual: {} vs {}",
            end_calc.y(),
            end.y()
        );
    }

    #[test]
    fn test_2d_end_derivatives_interval() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([2.0, 1.0]));
        control_points.push(Point2D::new([3.0, 0.0]));

        let bezier = BezierCurve2D::new(control_points.clone(), Interval::from_values(0.0, 17.5));

        let start = bezier.derivative(0.0);
        let end = bezier.derivative(17.5);

        let n = (control_points.len() - 1) as f64;

        // The start derivative should be n(P1 - P0)
        let start_calc = n * (control_points[1] - control_points[0]);
        assert!(
            f64_equal(start_calc.x(), start.x()),
            "Actual: {} vs {}",
            start_calc.x(),
            start.x()
        );
        assert!(
            f64_equal(start_calc.y(), start.y()),
            "Actual: {} vs {}",
            start_calc.y(),
            start.y()
        );

        // The end derivative should be n(Pn - Pn-1)
        let end_calc = n * (control_points[3] - control_points[2]);
        assert!(
            f64_equal(end_calc.x(), end.x()),
            "Actual: {} vs {}",
            end_calc.x(),
            end.x()
        );
        assert!(
            f64_equal(end_calc.y(), end.y()),
            "Actual: {} vs {}",
            end_calc.y(),
            end.y()
        );
    }

    #[test]
    fn test_2d_bernstein_vs_decasteljau() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([0.0, 1.0]));

        let bezier = BezierCurve2D::from_points(control_points.clone());

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let param = i as f64 * div;
            let bernstein = bezier._point_on_curve(param);
            let decasteljau = bezier.de_casteljau(param);
            assert!(
                f64_near(bernstein.x(), decasteljau.x(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                bernstein.x(),
                decasteljau.x()
            );
            assert!(
                f64_near(bernstein.y(), decasteljau.y(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                bernstein.y(),
                decasteljau.y()
            );
        }
    }

    #[test]
    fn test_2d_polynomial_compare() {
        let mut control_points = Vec::with_capacity(4);
        control_points.push(Point2D::new([0.0, 0.0]));
        control_points.push(Point2D::new([1.0, 0.0]));
        control_points.push(Point2D::new([1.0, 1.0]));
        control_points.push(Point2D::new([0.0, 1.0]));

        let bezier = BezierCurve2D::from_points(control_points.clone());

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let param = i as f64 * div;

            // Bezier
            let point_b = bezier.evaluate(param);

            // Cubic Bezier Polynomial
            let param_inverse = 1.0 - param;
            let mut point_p = param_inverse.powi(3) * control_points[0];
            point_p += 3.0 * param * param_inverse.powi(2) * control_points[1];
            point_p += 3.0 * param.powi(2) * param_inverse * control_points[2];
            point_p += param.powi(3) * control_points[3];

            // Compare
            assert!(
                f64_near(point_b.x(), point_p.x(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                point_b.x(),
                point_p.x()
            );
            assert!(
                f64_near(point_b.y(), point_p.y(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                point_b.y(),
                point_p.y()
            );
        }
    }

    #[test]
    fn test_2d_polynomial_compare_deriv_ex1_6() {
        let control_points = vec![
            Point2D::new([0.0, 0.0]),
            Point2D::new([0.0, 1.0]),
            Point2D::new([1.0, 1.0]),
            Point2D::new([1.0, 0.0]),
        ];
        let bezier = BezierCurve2D::from_points(control_points.clone());
        let div = 1.0 / 99.0;
        for i in 0..100 {
            // Bezier
            let param = i as f64 * div;
            let deriv_b = bezier.derivative(param);

            // Cubic Bezier Polynomial
            let param_inv = 1.0 - param;
            let mut deriv_p = param.powi(2) * (control_points[3] - control_points[2]);
            deriv_p += 2.0 * param_inv * param * (control_points[2] - control_points[1]);
            deriv_p += param_inv.powi(2) * (control_points[1] - control_points[0]);
            deriv_p *= 3.0;

            assert!(f64_equal(deriv_b.x(), deriv_p.x()));
            assert!(f64_equal(deriv_b.y(), deriv_p.y()));
        }
    }

    #[test]
    fn test_3d_construct() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::from_points(control_points);

        assert!(f64_equal(bezier.interval().min(), 0.0));
        assert!(f64_equal(bezier.interval().max(), 1.0));
    }

    #[test]
    fn test_3d_point() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::from_points(control_points.clone());

        let point = bezier.evaluate(0.5);

        let mut test_points = control_points;
        while test_points.len() > 1 {
            let mut temp_points = Vec::new();

            for i in 1..test_points.len() {
                temp_points.push((test_points[i - 1] + test_points[i]) * 0.5);
            }

            test_points = temp_points;
        }

        assert!(
            f64_equal(test_points[0].x(), point.x()),
            "Actual {} vs {}",
            test_points[0].x(),
            point.x()
        );
        assert!(
            f64_equal(test_points[0].y(), point.y()),
            "Actual {} vs {}",
            test_points[0].y(),
            point.y()
        );
        assert!(
            f64_equal(test_points[0].z(), point.z()),
            "Actual {} vs {}",
            test_points[0].z(),
            point.z()
        );
    }

    #[test]
    fn test_3d_point_interval() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::new(
            control_points.clone(),
            Interval::new(Point2D::new([0.0, 10.0])),
        );

        let point = bezier.evaluate(5.0);

        let mut test_points = control_points;
        while test_points.len() > 1 {
            let mut temp_points = Vec::new();

            for i in 1..test_points.len() {
                temp_points.push((test_points[i - 1] + test_points[i]) * 0.5);
            }

            test_points = temp_points;
        }

        assert!(
            f64_equal(test_points[0].x(), point.x()),
            "Actual {} vs {}",
            test_points[0].x(),
            point.x()
        );
        assert!(
            f64_equal(test_points[0].y(), point.y()),
            "Actual {} vs {}",
            test_points[0].y(),
            point.y()
        );
        assert!(
            f64_equal(test_points[0].z(), point.z()),
            "Actual {} vs {}",
            test_points[0].z(),
            point.z()
        );
    }

    #[test]
    fn test_3d_points() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::from_points(control_points.clone());

        let points = bezier.evaluate_points(100);

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let u = i as f64 * div;
            let u_inv = 1.0 - u;
            let mut test_points = control_points.clone();
            while test_points.len() > 1 {
                let mut temp_points = Vec::new();

                for i in 1..test_points.len() {
                    temp_points.push((u_inv * test_points[i - 1]) + (u * test_points[i]));
                }

                test_points = temp_points;
            }

            assert!(
                f64_equal(test_points[0].x(), points[i].x()),
                "Actual {} vs {}",
                test_points[0].x(),
                points[i].x()
            );
            assert!(
                f64_equal(test_points[0].y(), points[i].y()),
                "Actual {} vs {}",
                test_points[0].y(),
                points[i].y()
            );
            assert!(
                f64_equal(test_points[0].z(), points[i].z()),
                "Actual {} vs {}",
                test_points[0].z(),
                points[i].z()
            );
        }
    }
    
    #[test]
    fn test_3d_points_interval() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::new(control_points.clone(), Interval::from_values(0.0, 10.0));

        let points = bezier.evaluate_points(100);

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let u = i as f64 * div;
            let u_inv = 1.0 - u;
            let mut test_points = control_points.clone();
            while test_points.len() > 1 {
                let mut temp_points = Vec::new();

                for i in 1..test_points.len() {
                    temp_points.push((u_inv * test_points[i - 1]) + (u * test_points[i]));
                }

                test_points = temp_points;
            }

            assert!(
                f64_near(test_points[0].x(), points[i].x(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                test_points[0].x(),
                points[i].x()
            );
            assert!(
                f64_near(test_points[0].y(), points[i].y(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                test_points[0].y(),
                points[i].y()
            );
            assert!(
                f64_near(test_points[0].z(), points[i].z(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                test_points[0].z(),
                points[i].z()
            );
        }
    }

    #[test]
    fn test_3d_end_derivatives() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::from_points(control_points.clone());

        let start = bezier.derivative(0.0);
        let end = bezier.derivative(1.0);

        let n = (control_points.len() - 1) as f64;

        // The start derivative should be n(P1 - P0)
        let start_calc = n * (control_points[1] - control_points[0]);
        assert!(
            f64_equal(start_calc.x(), start.x()),
            "Actual: {} vs {}",
            start_calc.x(),
            start.x()
        );
        assert!(
            f64_equal(start_calc.y(), start.y()),
            "Actual: {} vs {}",
            start_calc.y(),
            start.y()
        );
        assert!(
            f64_equal(start_calc.z(), start.z()),
            "Actual: {} vs {}",
            start_calc.z(),
            start.z()
        );

        // The end derivative should be n(Pn - Pn-1)
        let end_calc = n * (control_points[3] - control_points[2]);
        assert!(
            f64_equal(end_calc.x(), end.x()),
            "Actual: {} vs {}",
            end_calc.x(),
            end.x()
        );
        assert!(
            f64_equal(end_calc.y(), end.y()),
            "Actual: {} vs {}",
            end_calc.y(),
            end.y()
        );
        assert!(
            f64_equal(end_calc.y(), end.y()),
            "Actual: {} vs {}",
            end_calc.z(),
            end.z()
        );
    }

    #[test]
    fn test_3d_end_derivatives_interval() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::new(control_points.clone(), Interval::from_values(0.0, 17.5));

        let start = bezier.derivative(0.0);
        let end = bezier.derivative(17.5);

        let n = (control_points.len() - 1) as f64;

        // The start derivative should be n(P1 - P0)
        let start_calc = n * (control_points[1] - control_points[0]);
        assert!(
            f64_equal(start_calc.x(), start.x()),
            "Actual: {} vs {}",
            start_calc.x(),
            start.x()
        );
        assert!(
            f64_equal(start_calc.y(), start.y()),
            "Actual: {} vs {}",
            start_calc.y(),
            start.y()
        );

        // The end derivative should be n(Pn - Pn-1)
        let end_calc = n * (control_points[3] - control_points[2]);
        assert!(
            f64_equal(end_calc.x(), end.x()),
            "Actual: {} vs {}",
            end_calc.x(),
            end.x()
        );
        assert!(
            f64_equal(end_calc.y(), end.y()),
            "Actual: {} vs {}",
            end_calc.y(),
            end.y()
        );
        assert!(
            f64_equal(end_calc.z(), end.z()),
            "Actual: {} vs {}",
            end_calc.z(),
            end.z()
        );
    }

    #[test]
    fn test_3d_bernstein_vs_decasteljau() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::from_points(control_points.clone());

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let param = i as f64 * div;
            let bernstein = bezier._point_on_curve(param);
            let decasteljau = bezier.de_casteljau(param);
            assert!(
                f64_near(bernstein.x(), decasteljau.x(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                bernstein.x(),
                decasteljau.x()
            );
            assert!(
                f64_near(bernstein.y(), decasteljau.y(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                bernstein.y(),
                decasteljau.y()
            );
            assert!(
                f64_near(bernstein.z(), decasteljau.z(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                bernstein.z(),
                decasteljau.z()
            );
        }
    }

    #[test]
    fn test_3d_polynomial_compare() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];

        let bezier = BezierCurve3D::from_points(control_points.clone());

        let div = 1.0 / 99.0;
        for i in 0..100 {
            let param = i as f64 * div;

            // Bezier
            let point_b = bezier.evaluate(param);

            // Cubic Bezier Polynomial
            let param_inverse = 1.0 - param;
            let mut point_p = param_inverse.powi(3) * control_points[0];
            point_p += 3.0 * param * param_inverse.powi(2) * control_points[1];
            point_p += 3.0 * param.powi(2) * param_inverse * control_points[2];
            point_p += param.powi(3) * control_points[3];

            // Compare
            assert!(
                f64_near(point_b.x(), point_p.x(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                point_b.x(),
                point_p.x()
            );
            assert!(
                f64_near(point_b.y(), point_p.y(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                point_b.y(),
                point_p.y()
            );
            assert!(
                f64_near(point_b.z(), point_p.z(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                point_b.z(),
                point_p.z()
            );
        }
    }

    #[test]
    fn test_3d_polynomial_compare_deriv_ex1_6() {
        let control_points = vec![
            Point3D::new([0.0, 0.0, 0.0]),
            Point3D::new([1.0, 0.0, 1.0]),
            Point3D::new([1.0, 1.0, 2.0]),
            Point3D::new([0.0, 1.0, 3.0]),
        ];
        let bezier = BezierCurve3D::from_points(control_points.clone());
        let div = 1.0 / 99.0;
        for i in 0..100 {
            // Bezier
            let param = i as f64 * div;
            let deriv_b = bezier.derivative(param);

            // Cubic Bezier Polynomial
            let param_inv = 1.0 - param;
            let mut deriv_p = param.powi(2) * (control_points[3] - control_points[2]);
            deriv_p += 2.0 * param_inv * param * (control_points[2] - control_points[1]);
            deriv_p += param_inv.powi(2) * (control_points[1] - control_points[0]);
            deriv_p *= 3.0;

            assert!(f64_equal(deriv_b.x(), deriv_p.x()));
            assert!(f64_equal(deriv_b.y(), deriv_p.y()));
            assert!(f64_near(deriv_b.z(), deriv_p.z(), f64::EPSILON * 10.0),
                "Actual {} vs {}",
                deriv_b.z(),
                deriv_p.z());
        
        }
    }

}