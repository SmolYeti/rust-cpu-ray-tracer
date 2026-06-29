use std::mem::swap;

// ALGORITHM A.2.1 int FindSpan(n,p,u,U) p.
// n - set as Max index of U
// p - degree
// u - input value along the curve/knot vector
// U - Knot vector
// Returns - span index of u
pub fn find_span_knot(degree: u32, knots: &Vec<u32>, knot: u32) -> u32 {
    let n = knots.len() as u32 - degree - 2;
    if knot >= knots[n as usize + 1] {
        n
    } else {
        if knot <= knots[degree as usize] {
            degree
        } else {
            let mut low = degree;
            let mut high = n + 1;
            let mut mid = (low + high) / 2;
            while (knot < knots[mid as usize] || knot >= knots[mid as usize + 1]) && low < high {
                if knot < knots[mid as usize] {
                    high = mid;
                } else {
                    low = mid;
                }
                mid = (low + high) / 2;
            }
            mid
        }
    }
}

pub fn find_span_param(degree: u32, knots: &Vec<f64>, param: f64, tolerance: f64) -> u32 {
    let n = knots.len() as u32 - degree - 2;
    if param >= knots[n as usize + 1] - tolerance {
        n
    } else {
        if param <= knots[degree as usize] + tolerance {
            degree
        } else {
            let mut low = degree;
            let mut high = n + 1;
            let mut mid = (low + high) / 2;
            while (param < knots[mid as usize] - tolerance
                || param >= knots[mid as usize + 1] - tolerance)
                && low < high
            {
                if param < knots[mid as usize] - tolerance {
                    high = mid;
                } else {
                    low = mid;
                }
                mid = (low + high) / 2;
            }
            mid
        }
    }
}

pub fn find_start_knot(degree: u32, knots: &Vec<u32>, knot: u32) -> u32 {
    let n = knots.len() as u32 - degree - 1;
    if knot >= knots[n as usize + 1] {
        n
    } else {
        if knot <= knots[degree as usize] {
            0
        } else {
            let mut low = degree + 1;
            let mut high = n - 1;
            let mut mid = (low + high) / 2;
            while !(knot > knots[(mid - 1) as usize] && knot <= knots[mid as usize]) {
                if knot <= knots[mid as usize] {
                    high = mid;
                } else if knot >= knots[mid as usize + 1] {
                    low = mid + 1;
                } else {
                    low = mid;
                }
                mid = (low + high) / 2;
            }
            mid
        }
    }
}

pub fn find_start_param(degree: u32, knots: &Vec<f64>, param: f64, tolerance: f64) -> u32 {
    let n = knots.len() as u32 - degree - 1;
    if param >= knots[n as usize + 1] - tolerance {
        n
    } else {
        if param <= knots[degree as usize] + tolerance {
            0
        } else {
            let mut low = degree + 1;
            let mut high = n - 1;
            let mut mid = (low + high) / 2;
            while !(param > knots[(mid - 1) as usize] + tolerance
                && param <= knots[mid as usize] + tolerance)
                && low < high
            {
                if param <= knots[mid as usize] + tolerance {
                    high = mid;
                } else if param >= knots[mid as usize + 1] - tolerance {
                    low = mid + 1;
                } else {
                    low = mid;
                }
                mid = (low + high) / 2;
            }
            mid
        }
    }
}

pub fn multiplicity_knot_u(degree: u32, knots: &Vec<u32>, knot: u32) -> i32 {
    let mut mult = -1;
    let index = find_start_knot(degree, knots, knot) as i32;
    while index + mult + 1 < knots.len() as i32 && knots[(index + mult + 1) as usize] == knot {
        mult += 1;
    }
    mult
}

pub fn multiplicity_knot_i(degree: u32, knots: &Vec<u32>, knot: i32) -> i32 {
    if knot < 0 {
        -1
    } else {
        multiplicity_knot_u(degree, knots, knot as u32)
    }
}

pub fn multiplicity_param(degree: u32, knots: &Vec<f64>, param: f64, tolerance: f64) -> i32 {
    let mut mult = -1;
    let index = find_start_param(degree, knots, param, tolerance) as i32;
    while index + mult + 1 < knots.len() as i32
        && (param - knots[(index + mult + 1) as usize]).abs() <= tolerance
    {
        mult += 1;
    }
    mult
}

// ALGORITHM A2.2  BasisFuns(i,u,p,U,N)
// i - Span index (From find span)
// u - input value along the curve/knot vector
// p - degree
// U - Knot vector
// N - Returned basis vector
pub fn basis_fun(span: u32, u: f64, degree: u32, knots: &Vec<f64>, tolerance: f64) -> Vec<f64> {
    let u = u.min(knots[knots.len() - 1]);
    let mut bases = vec![0.0; degree as usize + 1];
    if span >= knots.len() as u32 - degree - 2
        || (u >= knots[span as usize] - tolerance && u < knots[span as usize + 1] - tolerance)
    {
        bases[0] = 1.0;
    }
    let mut left = vec![0.0; degree as usize + 1];
    let mut right = vec![0.0; degree as usize + 1];
    for j in 1..(degree + 1) as usize {
        left[j] = u - knots[span as usize + 1 - j];
        right[j] = knots[span as usize + j] - u;
        let mut saved = 0.0;
        for r in 0..j {
            let temp = bases[r] / (right[r + 1] + left[j - r]);
            bases[r] = saved + (right[r + 1] * temp);
            saved = left[j - r] * temp;
        }
        bases[j] = saved;
    }
    bases
}

// ALGORITHM A2.3  DersBasisFuns(i,u,p,n,U,ders)
// Same as above
// n - nth derivative calculated (max)
// ders - Returned basis derivative vector
pub fn ders_basis_fun(
    i: usize,
    u: f64,
    degree: usize,
    n: usize,
    knots: &Vec<f64>,
) -> Vec<Vec<f64>> {
    if knots.is_empty() {
        Vec::new()
    } else if i + degree == knots.len() - 1 {
        vec![vec![0.0; degree + 1]; n + 1]
    } else {
        let u = u.min(knots[knots.len() - 1]);
        let mut ndu = vec![vec![0.0; degree + 1]; degree + 1];
        ndu[0][0] = 1.0;

        let mut left = vec![0.0; degree + 1];
        let mut right = vec![0.0; degree + 1];

        for j in 1..(degree + 1) {
            left[j] = u - knots[i + 1 - j];
            right[j] = knots[i + j] - u;

            let mut saved = 0.0;
            for r in 0..j {
                ndu[j][r] = right[r + 1] + left[j - r];
                let temp = ndu[r][j - 1] / ndu[j][r];

                ndu[r][j] = saved + (right[r + 1] * temp);
                saved = left[j - r] * temp;
            }
            ndu[j][j] = saved;
        }

        let mut derivatives = vec![vec![0.0; degree + 1]; n + 1];
        for j in 0..degree + 1 {
            derivatives[0][j] = ndu[j][degree];
        }

        // This section computes the derivatives
        let mut a = [vec![0.0; degree + 1], vec![0.0; degree + 1]];

        for r in 0..degree as i32 + 1 {
            let mut s1 = 0;
            let mut s2 = 1;
            a[0][0] = 1.0;
            for k in 1..n as i32 + 1 {
                let mut d = 0.0;
                let rk = r - k;
                let pk = degree as i32 - k;
                if r >= k {
                    a[s2][0] = a[s1][0] / ndu[(pk + 1) as usize][rk as usize];
                    d = a[s2][0] * ndu[rk as usize][pk as usize];
                }
                let j1 = if rk >= -1 { 1 } else { -rk };
                let j2 = if r - 1 <= pk {
                    k - 1
                } else {
                    degree as i32 - r
                };
                for j in j1..j2 + 1 {
                    a[s2][j as usize] = (a[s1][j as usize] - a[s1][(j - 1) as usize])
                        / ndu[(pk + 1) as usize][(rk + j) as usize];
                    d += a[s2][j as usize] * ndu[(rk + j) as usize][pk as usize];
                }
                if r <= pk {
                    a[s2][k as usize] =
                        -a[s1][(k - 1) as usize] / ndu[(pk + 1) as usize][r as usize];
                    d += a[s2][k as usize] * ndu[r as usize][pk as usize];
                }
                derivatives[k as usize][r as usize] = d;
                swap::<usize>(&mut s1, &mut s2);
            }
        }

        // Multiply through the correct factors
        let mut r = degree as f64;
        for k in 1..n + 1 {
            for j in 0..degree + 1 {
                derivatives[k][j] *= r;
            }
            r *= (degree - k) as f64;
        }

        derivatives
    }
}

// ALGORITHM A2.4  OneBasisFun(p,m,U,i,u,Nip)
// p - degree
// m - max index of U
// U - Knot vector
// i - ith basis value
// u - input value along the curve/knot vector
/*double OneBasisFun(uint32_t degree, const std::vector<double> &knots,
                   uint32_t i, double u) {
  u = std::min(u, static_cast<double>(knots[knots.size() - 1]));
  if ((i == 0 && u <= knots[0] + std::numeric_limits<double>::epsilon()) ||
      (i == (static_cast<uint32_t>(knots.size()) - degree - 2) &&
       u >= knots[knots.size() - 1] - std::numeric_limits<double>::epsilon())) {
    return 1.0;
  }
  if (u < knots[i] || u >= knots[i + degree + 1]) {
    return 0.0;
  }
  std::vector<double> N(degree + 1);
  for (uint32_t j = 0; j <= degree; ++j) {
    if (u >= knots[i + j] - std::numeric_limits<double>::epsilon() &&
        u < knots[i + j + 1] - std::numeric_limits<double>::epsilon()) {
      N[j] = 1.0;
    } else {
      N[j] = 0.0;
    }
  }
  for (uint32_t k = 1; k <= degree; ++k) {
    double saved = 0.0;
    if (std::abs(N[0]) > std::numeric_limits<double>::epsilon()) {
      saved = ((u - knots[i]) * N[0]) / (knots[i + k] - knots[i]);
    }
    for (uint32_t j = 0; j < degree + 1 - k; ++j) {
      double u_left = knots[i + j + 1];
      double u_right = knots[i + j + k + 1];
      if (std::abs(N[j + 1]) <= std::numeric_limits<double>::epsilon()) {
        N[j] = saved;
        saved = 0.0;
      } else {
        double temp = N[j + 1] / (u_right - u_left);
        N[j] = saved + ((u_right - u) * temp);
        saved = (u - u_left) * temp;
      }
    }
  }
  return N[0];
}

// ALGORITHM A2.5 - DersOneBasisFun(p,m,U,i,u,n,ders)
// p - degree
// m - max index of U
// U - Knot vector
// i - ith basis value
// u - input value along the curve / knot vector
// n - nth derivative calculated(max)
// ders - Returned basis derivative vector for only the ith basis
std::vector<double> DersOneBasisFun(uint32_t degree,
                                    const std::vector<double> &knots,
                                    uint32_t i, double u, uint32_t n) {
  u = std::min(u, knots[knots.size() - 1]);
  // Local property
  if (u < knots[i] - std::numeric_limits<double>::epsilon() ||
                static_cast<size_t>(i + degree + 1) >= knots.size() /*||
                u >= static_cast<double>(knots[i + degree + 1]) - std::numeric_limits<double>::epsilon()*/) {
    return std::vector<double>(n + 1, 0.0);
  }
  std::vector<std::vector<double>> N(degree + 1);
  for (auto &vect : N) {
    vect.resize(degree + 1);
  }

  // Initialize zero-degree functs
  for (uint32_t j = 0; j <= degree; ++j) {
    if (u >= knots[i + j] - std::numeric_limits<double>::epsilon() &&
        u < knots[i + j + 1] - std::numeric_limits<double>::epsilon()) {
      N[j][0] = 1.0;
    } else {
      N[j][0] = 0.0;
    }
  }

  // Compute full trianglar table
  for (uint32_t k = 1; k <= degree; ++k) {
    double saved = 0.0;
    if (std::abs(N[0][k - 1]) > std::numeric_limits<double>::epsilon()) {
      saved = ((u - knots[i]) * N[0][k - 1]) / (knots[i + k] - knots[i]);
    }
    for (uint32_t j = 0; j < degree + 1 - k; ++j) {
      double left = knots[i + j + 1];
      double right = knots[i + j + k + 1];
      if (std::abs(N[j + 1][k - 1]) <= std::numeric_limits<double>::epsilon()) {
        N[j][k] = saved;
        saved = 0.0;
      } else {
        double temp = N[j + 1][k - 1] / (right - left);
        N[j][k] = saved + ((right - u) * temp);
        saved = (u - left) * temp;
      }
    }
  }
  std::vector<double> derivatives(n + 1);
  // The function value
  derivatives[0] = N[0][degree];
  // Compute the derivatives
  for (uint32_t k = 1; k <= n; ++k) {
    // Load the appropriate column
    std::vector<double> ND(k + 1);
    for (uint32_t j = 0; j <= k; ++j) {
      ND[j] = N[j][degree - k];
    }
    // Compute the table of width k
    for (uint32_t jj = 1; jj <= k; ++jj) {
      double saved = 0.0;
      if (std::abs(ND[0]) > std::numeric_limits<double>::epsilon()) {
        saved = ND[0] / (knots[i + degree + jj - k] - knots[i]);
      }
      for (uint32_t j = 0; j < k + 1 - jj; ++j) {
        double left = knots[i + j + 1];
        double right = knots[knots.size() - 1];
        if (static_cast<size_t>(i + j + degree + jj + 1) < knots.size()) {
          right = knots[i + j + degree + jj + 1];
        }
        if (std::abs(ND[j + 1]) <= std::numeric_limits<double>::epsilon()) {
          ND[j] = static_cast<double>(degree + jj - k) * saved;
          saved = 0.0;
        } else {
          double temp = ND[j + 1] / (right - left);
          ND[j] = static_cast<double>(degree + jj - k) * (saved - temp);
          saved = temp;
        }
      }
    }
    derivatives[k] = ND[0];
  }
  return derivatives;
}

// ALGORITHM  ALLBasisFuns(i,u,p,U,N) p99
// i - Span index (From find span)
// u - input value along the curve/knot vector
// p - degree
// U - Knot vector
// N - Returned basis vector
// Blurb:
/* We assume a routine, AllBasisFuns, which is a simple
   modification of Basis Funs (Algorithm A2. 2), to return all nonzero basis
   functions of all degrees from 0 up top. In particular, N[j] [i] is the value
   of the ith-degree basis function, Nspan-i+j,i(u), where 0<=i<=p and 0<=j<=i.
 */
// This has not been tested or used, Though it likely doesn't work
std::vector<std::vector<double>>
AllBasisFuns(uint32_t span, double u, uint32_t degree,
             const std::vector<double> &knots) {
  u = std::min(u, knots[knots.size() - 1]);
  std::vector<std::vector<double>> bases(degree + 1);
  for (auto &vect : bases) {
    vect.resize(degree + 1, 0);
  }
  // Initalize basis values
  // Initialize zero-degree functs
  for (uint32_t i = 0; i <= degree; ++i) {
    if (u >= knots[span + i] - std::numeric_limits<double>::epsilon() &&
        u < knots[span + i + 1] - std::numeric_limits<double>::epsilon()) {
      bases[i][0] = 1.0;
    } else {
      bases[i][0] = 0.0;
    }
  }
  std::vector<double> left(degree * 2), right(degree + 2);
  for (uint8_t i = 0; i < degree * 2; ++i) {
    left[i] = u - knots[span + degree - 1 - i];
  }
  for (uint8_t j = 1; j <= degree; ++j) {
    right[j] = knots[span + j] - u;
    bases[0][j] = 0.0;
    for (uint8_t r = 0; r < degree; ++r) {
      double temp = bases[r][j - 1] / (right[r + 1] + left[j - r + degree - 2]);
      bases[r][j] = bases[r][j] + (right[r + 1] * temp);
      bases[r + 1][j] = left[j - r + degree - 2] * temp;
    }
  }
  return bases;
}

// - https://en.wikipedia.org/wiki/Binomial_coefficient
// - ->bin(n, k) = (n * (n - 1) * ... * (n - k + 1) / (k * (k - 1) * ...
// * 1)
std::vector<std::vector<double>> BinomialCoefficients(uint32_t n, uint32_t k) {
  std::vector<std::vector<double>> bin(n + 1, std::vector<double>(k + 1, 1));
  for (uint32_t i = 1; i <= n; ++i) {
    for (uint32_t j = 1; j <= k; ++j) {
      bin[i][j] = bin[i - 1][j - 1] + bin[i - 1][j];
    }
  }
  return bin;
}
 */

#[cfg(test)]
mod tests {

    mod find_span {
        use crate::knot_utility_functions::find_span_knot;
        use crate::knot_utility_functions::find_span_param;
        static K_TOLERANCE: f64 = f64::EPSILON;

        #[test]
        fn test_knot_min() {
            let degree = 2;
            let knots: Vec<u32> = vec![0, 0, 0, 1, 2, 2, 2];
            let ret = find_span_knot(degree, &knots, 0);
            assert_eq!(ret, 2);
        }

        #[test]
        fn test_knot_mid() {
            let degree = 2;
            let knots: Vec<u32> = vec![0, 0, 0, 1, 2, 2, 2];
            let ret = find_span_knot(degree, &knots, 1);
            assert_eq!(ret, 3);
        }

        #[test]
        fn testknot_max() {
            let degree = 2;
            let knots: Vec<u32> = vec![0, 0, 0, 1, 2, 2, 2];
            let ret = find_span_knot(degree, &knots, 2);
            assert_eq!(ret, 3);
            let ret = find_span_knot(degree, &knots, 3);
            assert_eq!(ret, 3);
        }

        #[test]
        fn test_param_min() {
            let degree = 2;
            let knots: Vec<f64> = vec![0., 0., 0., 1., 2., 2., 2.];
            let ret = find_span_param(degree, &knots, -1., K_TOLERANCE);
            assert_eq!(ret, 2);
            let ret = find_span_param(degree, &knots, 0., K_TOLERANCE);
            assert_eq!(ret, 2);
        }

        #[test]
        fn test_param_mid() {
            let degree = 2;
            let knots: Vec<f64> = vec![0., 0., 0., 1., 2., 2., 2.];
            let ret = find_span_param(degree, &knots, 0.5, K_TOLERANCE);
            assert_eq!(ret, 2);
            let ret = find_span_param(degree, &knots, 1., K_TOLERANCE);
            assert_eq!(ret, 3);
            let ret = find_span_param(degree, &knots, 1.5, K_TOLERANCE);
            assert_eq!(ret, 3);
        }

        #[test]
        fn test_param_max() {
            let degree = 2;
            let knots: Vec<f64> = vec![0., 0., 0., 1., 2., 2., 2.];
            let ret = find_span_param(degree, &knots, 2., K_TOLERANCE);
            assert_eq!(ret, 3);
            let ret = find_span_param(degree, &knots, 3., K_TOLERANCE);
            assert_eq!(ret, 3);
        }
    }

    mod find_start {
        use crate::knot_utility_functions::find_start_knot;
        use crate::knot_utility_functions::find_start_param;
        static K_TOLERANCE: f64 = f64::EPSILON;

        #[test]
        fn test_knot() {
            let degree = 3;
            let knots: Vec<u32> = vec![0, 0, 0, 0, 1, 2, 2, 2, 3, 3, 4, 4, 4, 4];
            let ret = find_start_knot(degree, &knots, 0);
            assert_eq!(ret, 0);
            let ret = find_start_knot(degree, &knots, 1);
            assert_eq!(ret, 4);
            let ret = find_start_knot(degree, &knots, 2);
            assert_eq!(ret, 5);
            let ret = find_start_knot(degree, &knots, 3);
            assert_eq!(ret, 8);
            let ret = find_start_knot(degree, &knots, 4);
            assert_eq!(ret, 10);
            let ret = find_start_knot(degree, &knots, 5);
            assert_eq!(ret, 10);
        }

        #[test]
        fn test_param() {
            let degree = 3;
            let knots: Vec<f64> = vec![0., 0., 0., 0., 1., 2., 2., 2., 3., 3., 4., 4., 4., 4.];
            let ret = find_start_param(degree, &knots, -1., K_TOLERANCE);
            assert_eq!(ret, 0);
            let ret = find_start_param(degree, &knots, 0., K_TOLERANCE);
            assert_eq!(ret, 0);
            let ret = find_start_param(degree, &knots, 1., K_TOLERANCE);
            assert_eq!(ret, 4);
            let ret = find_start_param(degree, &knots, 2., K_TOLERANCE);
            assert_eq!(ret, 5);
            let ret = find_start_param(degree, &knots, 3., K_TOLERANCE);
            assert_eq!(ret, 8);
            let ret = find_start_param(degree, &knots, 4., K_TOLERANCE);
            assert_eq!(ret, 10);
            let ret = find_start_param(degree, &knots, 5., K_TOLERANCE);
            assert_eq!(ret, 10);
        }
    }

    mod multiplicity {
        use crate::knot_utility_functions::multiplicity_knot_i;
        use crate::knot_utility_functions::multiplicity_knot_u;
        use crate::knot_utility_functions::multiplicity_param;
        static K_TOLERANCE: f64 = f64::EPSILON;

        #[test]
        fn test_knot_u() {
            let degree = 3;
            let knots: Vec<u32> = vec![0, 0, 0, 0, 1, 2, 2, 2, 3, 3, 4, 4, 4, 4];
            let ret = multiplicity_knot_u(degree, &knots, 0);
            assert_eq!(ret, 3);
            let ret = multiplicity_knot_u(degree, &knots, 1);
            assert_eq!(ret, 0);
            let ret = multiplicity_knot_u(degree, &knots, 2);
            assert_eq!(ret, 2);
            let ret = multiplicity_knot_u(degree, &knots, 3);
            assert_eq!(ret, 1);
            let ret = multiplicity_knot_u(degree, &knots, 4);
            assert_eq!(ret, 3);
            let ret = multiplicity_knot_u(degree, &knots, 5);
            assert_eq!(ret, -1);
        }

        #[test]
        fn test_knot_i() {
            let degree = 3;
            let knots: Vec<u32> = vec![0, 0, 0, 0, 1, 2, 2, 2, 3, 3, 4, 4, 4, 4];
            let ret = multiplicity_knot_i(degree, &knots, -1);
            assert_eq!(ret, -1);
            let ret = multiplicity_knot_i(degree, &knots, 0);
            assert_eq!(ret, 3);
            let ret = multiplicity_knot_i(degree, &knots, 1);
            assert_eq!(ret, 0);
            let ret = multiplicity_knot_i(degree, &knots, 2);
            assert_eq!(ret, 2);
            let ret = multiplicity_knot_i(degree, &knots, 3);
            assert_eq!(ret, 1);
            let ret = multiplicity_knot_i(degree, &knots, 4);
            assert_eq!(ret, 3);
            let ret = multiplicity_knot_i(degree, &knots, 5);
            assert_eq!(ret, -1);
        }

        #[test]
        fn test_param() {
            let degree = 3;
            let knots: Vec<f64> = vec![
                0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0,
            ];
            let ret = multiplicity_param(degree, &knots, -1.0, K_TOLERANCE);
            assert_eq!(ret, -1);
            let ret = multiplicity_param(degree, &knots, 0.0, K_TOLERANCE);
            assert_eq!(ret, 3);
            let ret = multiplicity_param(degree, &knots, 0.5, K_TOLERANCE);
            assert_eq!(ret, -1);
            let ret = multiplicity_param(degree, &knots, 1.0, K_TOLERANCE);
            assert_eq!(ret, 0);
            let ret = multiplicity_param(degree, &knots, 1.5, K_TOLERANCE);
            assert_eq!(ret, -1);
            let ret = multiplicity_param(degree, &knots, 2.0, K_TOLERANCE);
            assert_eq!(ret, 2);
            let ret = multiplicity_param(degree, &knots, 2.99, K_TOLERANCE);
            assert_eq!(ret, -1);
            let ret = multiplicity_param(degree, &knots, 3.0, K_TOLERANCE);
            assert_eq!(ret, 1);
            let ret = multiplicity_param(degree, &knots, 3.00001, K_TOLERANCE);
            assert_eq!(ret, -1);
            let ret = multiplicity_param(degree, &knots, 4.0, K_TOLERANCE);
            assert_eq!(ret, 3);
            let ret = multiplicity_param(degree, &knots, 5.0, K_TOLERANCE);
            assert_eq!(ret, -1);
        }
    }

    mod basis {
        use crate::{
            knot_utility_functions::{basis_fun, find_span_param},
            utility::f64_equal,
        };
        use core::f64;
        static K_TOLERANCE: f64 = f64::EPSILON;

        #[test]
        fn test_ex2_3() {
            let u_value = 2.5;
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 5.0];
            let degree = 2;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 4);

            let bases = basis_fun(span_index, u_value, degree, &knots, K_TOLERANCE);
            assert_eq!(bases.len(), 3);
            assert!(f64_equal(bases[0], 1.0 / 8.0));
            assert!(f64_equal(bases[1], 6.0 / 8.0));
            assert!(f64_equal(bases[2], 1.0 / 8.0));
        }

        #[test]
        fn test_min() {
            let u_value = 0.0;
            let degree = 2;
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0];
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            let bases = basis_fun(span_index, u_value, degree, &knots, K_TOLERANCE);
            assert_eq!(bases.len(), 3);
            assert!(f64_equal(bases[0], 1.0));
            assert!(f64_equal(bases[1], 0.0));
            assert!(f64_equal(bases[2], 0.0));
        }

        #[test]
        fn test_mid() {
            let u_value_0 = 1.0;
            let u_value_1 = 1.5;
            let degree = 2;
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0];
            let span_index = find_span_param(degree, &knots, u_value_0, K_TOLERANCE);
            let bases = basis_fun(span_index, u_value_0, degree, &knots, K_TOLERANCE);
            assert_eq!(bases.len(), 3);
            assert!(f64_equal(bases[0], 0.5));
            assert!(f64_equal(bases[1], 0.5));
            assert!(f64_equal(bases[2], 0.0));

            let span_index = find_span_param(degree, &knots, u_value_1, K_TOLERANCE);
            let bases = basis_fun(span_index, u_value_1, degree, &knots, K_TOLERANCE);
            assert_eq!(bases.len(), 3);
            assert!(f64_equal(bases[0], 0.125));
            assert!(f64_equal(bases[1], 0.625));
            assert!(f64_equal(bases[2], 0.25));
        }

        #[test]
        fn test_max() {
            let u_value_0 = 2.0;
            let u_value_1 = 2.5;
            let degree = 2;
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0];
            let span_index = find_span_param(degree, &knots, u_value_0, K_TOLERANCE);
            let bases = basis_fun(span_index, u_value_0, degree, &knots, K_TOLERANCE);
            assert_eq!(bases.len(), 3);
            assert!(f64_equal(bases[0], 0.0));
            assert!(f64_equal(bases[1], 0.0));
            assert!(f64_equal(bases[2], 1.0));

            let span_index = find_span_param(degree, &knots, u_value_1, K_TOLERANCE);
            let bases = basis_fun(span_index, u_value_1, degree, &knots, K_TOLERANCE);
            assert_eq!(bases.len(), 3);
            assert!(f64_equal(bases[0], 0.0));
            assert!(f64_equal(bases[1], 0.0));
            assert!(f64_equal(bases[2], 1.0));
        }
    }

    mod basis_derivative {
        use crate::{
            knot_utility_functions::{ders_basis_fun, find_span_param},
            utility::f64_equal,
        };
        static K_TOLERANCE: f64 = core::f64::EPSILON;

        #[test]
        fn test_ex2_4() {
            let u_val = 2.5;
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 5.0];
            let degree = 2;
            let span_index = find_span_param(degree, &knots, u_val, K_TOLERANCE);
            assert_eq!(span_index, 4);

            let basis_ders = ders_basis_fun(span_index as usize, u_val, degree as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[2].len(), 3);
            assert!(
                f64_equal(basis_ders[1][2], 0.5),
                "Actual {} vs 0.5",
                basis_ders[1][2]
            );
            assert!(
                f64_equal(basis_ders[2][2], 1.0),
                "Actual {} vs 1.0",
                basis_ders[2][2]
            );
        }

        #[test]
        fn test_ex_0_0() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 0.0;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 3);

            // Degree 1, Derivative 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));
            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[1][0], -2.0));
            assert!(f64_equal(basis_ders[1][1], 2.0));
            assert!(f64_equal(basis_ders[1][2], 0.0));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));
            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[0][3], 0.0));
            assert!(f64_equal(basis_ders[1][0], -3.0));
            assert!(f64_equal(basis_ders[1][1], 3.0));
            assert!(f64_equal(basis_ders[1][2], 0.0));
            assert!(f64_equal(basis_ders[1][3], 0.0));
            assert!(f64_equal(basis_ders[2][0], 6.0));
            assert!(f64_equal(basis_ders[2][1], -9.0));
            assert!(f64_equal(basis_ders[2][2], 3.0));
            assert!(f64_equal(basis_ders[2][3], 0.0));
        }

        #[test]
        fn test_ex_0_5() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 0.5;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 3);

            // Degree 1, Derivative 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.25));
            assert!(f64_equal(basis_ders[0][1], 0.625));
            assert!(f64_equal(basis_ders[0][2], 0.125));
            assert!(f64_equal(basis_ders[1][0], -1.0));
            assert!(f64_equal(basis_ders[1][1], 0.5));
            assert!(f64_equal(basis_ders[1][2], 0.5));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.125));
            assert!(f64_equal(basis_ders[0][1], 0.59375));
            assert!(f64_equal(basis_ders[0][2], 0.26041666666666663));
            assert!(f64_equal(basis_ders[0][3], 0.020833333333333332));
            assert!(f64_equal(basis_ders[1][0], -0.75));
            assert!(f64_equal(basis_ders[1][1], -0.1875));
            assert!(f64_equal(basis_ders[1][2], 0.8125));
            assert!(f64_equal(basis_ders[1][3], 0.125));
            assert!(f64_equal(basis_ders[2][0], 3.0));
            assert!(f64_equal(basis_ders[2][1], -3.75));
            assert!(f64_equal(basis_ders[2][2], 0.25));
            assert!(f64_equal(basis_ders[2][3], 0.5));
        }

        #[test]
        fn test_ex_1_0() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 1.0;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 4);

            // Degree 1, Derivitive 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));
            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[1][0], -1.0));
            assert!(f64_equal(basis_ders[1][1], 1.0));
            assert!(f64_equal(basis_ders[1][2], 0.0));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.25));
            assert!(f64_equal(basis_ders[0][1], 0.58333333333333326));
            assert!(f64_equal(basis_ders[0][2], 0.16666666666666666));
            assert!(f64_equal(basis_ders[0][3], 0.0));
            assert!(f64_equal(basis_ders[1][0], -0.75));
            assert!(f64_equal(basis_ders[1][1], 0.25));
            assert!(f64_equal(basis_ders[1][2], 0.5));
            assert!(f64_equal(basis_ders[1][3], 0.0));
            assert!(f64_equal(basis_ders[2][0], 1.5));
            assert!(f64_equal(basis_ders[2][1], -2.5));
            assert!(f64_equal(basis_ders[2][2], 1.0));
            assert!(f64_equal(basis_ders[2][3], 0.0));
        }

        #[test]
        fn test_ex_1_5() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 1.5;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 4);

            // Degree 1, Derivitive 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.125));
            assert!(f64_equal(basis_ders[0][1], 0.75));
            assert!(f64_equal(basis_ders[0][2], 0.125));
            assert!(f64_equal(basis_ders[1][0], -0.5));
            assert!(f64_equal(basis_ders[1][1], 0.0));
            assert!(f64_equal(basis_ders[1][2], 0.5));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.03125));
            assert!(f64_equal(basis_ders[0][1], 0.46875));
            assert!(f64_equal(basis_ders[0][2], 0.46875));
            assert!(f64_equal(basis_ders[0][3], 0.03125));
            assert!(f64_equal(basis_ders[1][0], -0.1875));
            assert!(f64_equal(basis_ders[1][1], -0.5625));
            assert!(f64_equal(basis_ders[1][2], 0.5625));
            assert!(f64_equal(basis_ders[1][3], 0.1875));
            assert!(f64_equal(basis_ders[2][0], 0.75));
            assert!(f64_equal(basis_ders[2][1], -0.75));
            assert!(f64_equal(basis_ders[2][2], -0.75));
            assert!(f64_equal(basis_ders[2][3], 0.75));
        }

        #[test]
        fn test_ex_2_0() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 2.0;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 5);

            // Degree 1, Derivitive 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));
            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[1][0], -1.0));
            assert!(f64_equal(basis_ders[1][1], 1.0));
            assert!(f64_equal(basis_ders[1][2], 0.0));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.16666666666666666));
            assert!(f64_equal(basis_ders[0][1], 0.58333333333333326));
            assert!(f64_equal(basis_ders[0][2], 0.25));
            assert!(f64_equal(basis_ders[0][3], 0.0));
            assert!(f64_equal(basis_ders[1][0], -0.5));
            assert!(f64_equal(basis_ders[1][1], -0.25));
            assert!(f64_equal(basis_ders[1][2], 0.75));
            assert!(f64_equal(basis_ders[1][3], 0.0));
            assert!(f64_equal(basis_ders[2][0], 1.0));
            assert!(f64_equal(basis_ders[2][1], -2.5));
            assert!(f64_equal(basis_ders[2][2], 1.5));
            assert!(f64_equal(basis_ders[2][3], 0.0));
        }

        #[test]
        fn test_ex_2_5() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 2.5;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 5);

            // Degree 1, Derivitive 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.125));
            assert!(f64_equal(basis_ders[0][1], 0.625));
            assert!(f64_equal(basis_ders[0][2], 0.25));
            assert!(f64_equal(basis_ders[1][0], -0.5));
            assert!(f64_equal(basis_ders[1][1], -0.5));
            assert!(f64_equal(basis_ders[1][2], 1.0));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.020833333333333332));
            assert!(f64_equal(basis_ders[0][1], 0.26041666666666663));
            assert!(f64_equal(basis_ders[0][2], 0.65625));
            assert!(f64_equal(basis_ders[0][3], 0.0625));
            assert!(f64_equal(basis_ders[1][0], -0.125));
            assert!(f64_equal(basis_ders[1][1], -0.8125));
            assert!(f64_equal(basis_ders[1][2], 0.5625));
            assert!(f64_equal(basis_ders[1][3], 0.375));
            assert!(f64_equal(basis_ders[2][0], 0.5));
            assert!(f64_equal(basis_ders[2][1], 0.25));
            assert!(f64_equal(basis_ders[2][2], -2.25));
            assert!(f64_equal(basis_ders[2][3], 1.5));
        }

        #[test]
        fn test_ex_3_0() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 3.0;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 7);

            // Degree 1, Derivitive 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));
            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[1][0], -2.0));
            assert!(f64_equal(basis_ders[1][1], 2.0));
            assert!(f64_equal(basis_ders[1][2], 0.0));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));
            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[0][3], 0.0));
            assert!(f64_equal(basis_ders[1][0], -1.5));
            assert!(f64_equal(basis_ders[1][1], 1.5));
            assert!(f64_equal(basis_ders[1][2], 0.0));
            assert!(f64_equal(basis_ders[1][3], 0.0));
            assert!(f64_equal(basis_ders[2][0], 3.0));
            assert!(f64_equal(basis_ders[2][1], -9.0));
            assert!(f64_equal(basis_ders[2][2], 6.0));
            assert!(f64_equal(basis_ders[2][3], 0.0));
        }
    
        #[test]
        fn test_ex_3_5() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 3.5;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 7);

            // Degree 1, Derivitive 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.25));
            assert!(f64_equal(basis_ders[0][1], 0.5));
            assert!(f64_equal(basis_ders[0][2], 0.25));
            assert!(f64_equal(basis_ders[1][0], -1.0));
            assert!(f64_equal(basis_ders[1][1], 0.0));
            assert!(f64_equal(basis_ders[1][2], 1.0));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.0625));
            assert!(f64_equal(basis_ders[0][1], 0.4375));
            assert!(f64_equal(basis_ders[0][2], 0.375));
            assert!(f64_equal(basis_ders[0][3], 0.125));
            assert!(f64_equal(basis_ders[1][0], -0.375));
            assert!(f64_equal(basis_ders[1][1], -1.125));
            assert!(f64_equal(basis_ders[1][2], 0.75));
            assert!(f64_equal(basis_ders[1][3], 0.75));
            assert!(f64_equal(basis_ders[2][0], 1.5));
            assert!(f64_equal(basis_ders[2][1], -1.5));
            assert!(f64_equal(basis_ders[2][2], -3.0));
            assert!(f64_equal(basis_ders[2][3], 3.0));
        }
   
        #[test]
        fn test_ex_4_0() {
            let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
            let degree = 3;
            let degree_1 = 1;
            let degree_2 = 2;
            let degree_3 = 3;

            let u_value = 4.0;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 7);

            // Degree 1, Derivitive 0
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_1 as usize, 0, &knots);
            assert_eq!(basis_ders.len(), 1);
            assert_eq!(basis_ders[0].len(), 2);
            assert!(f64_equal(basis_ders[0][0], 0.0));
            assert!(f64_equal(basis_ders[0][1], 1.0));

            // Degree 2, Derivative 1
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_2 as usize, 1, &knots);
            assert_eq!(basis_ders.len(), 2);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));
            assert!(f64_equal(basis_ders[0][2], 1.0));
            assert!(f64_equal(basis_ders[1][0], 0.0));
            assert!(f64_equal(basis_ders[1][1], -2.0));
            assert!(f64_equal(basis_ders[1][2], 2.0));

            // Degree 3, Derivitive 2
            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree_3 as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 4);
            assert!(f64_equal(basis_ders[0][0], 0.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));
            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[0][3], 1.0));
            assert!(f64_equal(basis_ders[1][0], 0.0));
            assert!(f64_equal(basis_ders[1][1], 0.0));
            assert!(f64_equal(basis_ders[1][2], -3.0));
            assert!(f64_equal(basis_ders[1][3], 3.0));
            assert!(f64_equal(basis_ders[2][0], 0.0));
            assert!(f64_equal(basis_ders[2][1], 6.0));
            assert!(f64_equal(basis_ders[2][2], -12.0));
            assert!(f64_equal(basis_ders[2][3], 6.0));
        }
   
        #[test]
        fn test_min() {
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 5.0];
            let degree = 2;
            let u_value = 0.0;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 2);

            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 1.0));
            assert!(f64_equal(basis_ders[1][0], -2.0));
            assert!(f64_equal(basis_ders[2][0], 2.0));

            assert!(f64_equal(basis_ders[0][1], 0.0));
            assert!(f64_equal(basis_ders[1][1], 2.0));
            assert!(f64_equal(basis_ders[2][1], -3.0));

            assert!(f64_equal(basis_ders[0][2], 0.0));
            assert!(f64_equal(basis_ders[1][2], 0.0));
            assert!(f64_equal(basis_ders[2][2], 1.0));
        }
   
        #[test]
        fn test_mid() {
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 5.0];
            let degree = 2;
            let u_value_0 = 2.5;
            let u_value_1 = 3.0;

            // u: 2.5
            let span_index = find_span_param(degree, &knots, u_value_0, K_TOLERANCE);
            assert_eq!(span_index, 4);

            let basis_ders =
                ders_basis_fun(span_index as usize, u_value_0, degree as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.125));
            assert!(f64_equal(basis_ders[0][1], 0.75));
            assert!(f64_equal(basis_ders[0][2], 0.125));

            assert!(f64_equal(basis_ders[1][0], -0.5));
            assert!(f64_equal(basis_ders[1][1], 0.0));
            assert!(f64_equal(basis_ders[1][2], 0.5));

            assert!(f64_equal(basis_ders[2][0], 1.0));
            assert!(f64_equal(basis_ders[2][1], -2.0));
            assert!(f64_equal(basis_ders[2][2], 1.0));

            // u: 3.0
            let span_index = find_span_param(degree, &knots, u_value_1, K_TOLERANCE);
            assert_eq!(span_index, 5);

            let basis_ders =
                ders_basis_fun(span_index as usize, u_value_1, degree as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.5));
            assert!(f64_equal(basis_ders[0][1], 0.5));
            assert!(f64_equal(basis_ders[0][2], 0.0));

            assert!(f64_equal(basis_ders[1][0], -1.0));
            assert!(f64_equal(basis_ders[1][1], 1.0));
            assert!(f64_equal(basis_ders[1][2], 0.0));

            assert!(f64_equal(basis_ders[2][0], 1.0));
            assert!(f64_equal(basis_ders[2][1], -3.0));
            assert!(f64_equal(basis_ders[2][2], 2.0));
        }

        #[test]
        fn test_max() {
            let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 5.0, 5.0, 5.0];
            let degree = 2;
            let u_value = 5.0;
            let span_index = find_span_param(degree, &knots, u_value, K_TOLERANCE);
            assert_eq!(span_index, 7);

            let basis_ders =
                ders_basis_fun(span_index as usize, u_value, degree as usize, 2, &knots);
            assert_eq!(basis_ders.len(), 3);
            assert_eq!(basis_ders[0].len(), 3);
            assert!(f64_equal(basis_ders[0][0], 0.0));
            assert!(f64_equal(basis_ders[0][1], 0.0));
            assert!(f64_equal(basis_ders[0][2], 1.0));

            assert!(f64_equal(basis_ders[1][0], 0.0));
            assert!(f64_equal(basis_ders[1][1], -2.0));
            assert!(f64_equal(basis_ders[1][2], 2.0));

            assert!(f64_equal(basis_ders[2][0], 2.0));
            assert!(f64_equal(basis_ders[2][1], -4.0));
            assert!(f64_equal(basis_ders[2][2], 2.0));
        }
    }

    mod single_basis {
        use crate::{
            knot_utility_functions::{ders_basis_fun, find_span_param},
            utility::f64_equal,
        };
        static K_TOLERANCE: f64 = core::f64::EPSILON;

        #[test]
        fn test_runs() {
            assert!(true)
        }

    }
}

/*

TEST(NURBS_Chapter2, BasisDerivativeMid) {
  constexpr double u_value_0 = 2.5;
  constexpr double u_value_1 = 3.0;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 3, 4, 4, 5, 5, 5};
  constexpr uint32_t degree = 2;
  uint32_t span_index = FindSpanParam(degree, knots, u_value_0, kTolerance);
  EXPECT_EQ(span_index, 4);
  std::vector<std::vector<double>> basis_ders =
      DersBasisFuns(span_index, u_value_0, degree, 2, knots);
  EXPECT_EQ(basis_ders.size(), 3);
  EXPECT_EQ(basis_ders[2].size(), 3);
  // N^0 i, 2
  EXPECT_DOUBLE_EQ(basis_ders[0][0], 0.125);
  EXPECT_DOUBLE_EQ(basis_ders[0][1], 0.75);
  EXPECT_DOUBLE_EQ(basis_ders[0][2], 0.125);
  // N^1 i, 2
  EXPECT_DOUBLE_EQ(basis_ders[1][0], -0.5);
  EXPECT_DOUBLE_EQ(basis_ders[1][1], 0.0);
  EXPECT_DOUBLE_EQ(basis_ders[1][2], 0.5);
  // N^2 i, 2
  EXPECT_DOUBLE_EQ(basis_ders[2][0], 1.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][1], -2.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][2], 1.0);

  span_index = FindSpanParam(degree, knots, u_value_1, kTolerance);
  EXPECT_EQ(span_index, 5);
  basis_ders = DersBasisFuns(span_index, u_value_1, degree, 2, knots);
  EXPECT_EQ(basis_ders.size(), 3);
  EXPECT_EQ(basis_ders[2].size(), 3);
  EXPECT_DOUBLE_EQ(basis_ders[0][0], 0.5);
  EXPECT_DOUBLE_EQ(basis_ders[1][0], -1.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][0], 1.0);

  EXPECT_DOUBLE_EQ(basis_ders[0][1], 0.5);
  EXPECT_DOUBLE_EQ(basis_ders[1][1], 1.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][1], -3.0);

  EXPECT_DOUBLE_EQ(basis_ders[0][2], 0.0);
  EXPECT_DOUBLE_EQ(basis_ders[1][2], 0.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][2], 2.0);
}

TEST(NURBS_Chapter2, BasisDerivativeMax) {
  constexpr double u_value = 5.0;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 3, 4, 4, 5, 5, 5};
  constexpr uint32_t degree = 2;
  uint32_t span_index = FindSpanParam(degree, knots, u_value, kTolerance);
  EXPECT_EQ(span_index, 7);
  std::vector<std::vector<double>> basis_ders =
      DersBasisFuns(span_index, u_value, degree, 2, knots);
  EXPECT_EQ(basis_ders.size(), 3);
  EXPECT_EQ(basis_ders[2].size(), 3);
  EXPECT_DOUBLE_EQ(basis_ders[0][0], 0.0);
  EXPECT_DOUBLE_EQ(basis_ders[1][0], 0.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][0], 2.0);

  EXPECT_DOUBLE_EQ(basis_ders[0][1], 0.0);
  EXPECT_DOUBLE_EQ(basis_ders[1][1], -2.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][1], -4.0);

  EXPECT_DOUBLE_EQ(basis_ders[0][2], 1.0);
  EXPECT_DOUBLE_EQ(basis_ders[1][2], 2.0);
  EXPECT_DOUBLE_EQ(basis_ders[2][2], 2.0);
}


// Test the single function against the multi function
TEST(NURBS_Chapter2, SingleBasisMin) {
  constexpr double u_value = 0.0;
  constexpr uint32_t degree = 2;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 2, 2};
  const uint32_t span_index = FindSpanParam(degree, knots, u_value, kTolerance);
  const std::vector<double> bases =
      BasisFuns(span_index, u_value, degree, knots, kTolerance);

  double basis = OneBasisFun(degree, knots, 0, u_value);
  EXPECT_DOUBLE_EQ(bases[0], basis);

  basis = OneBasisFun(degree, knots, 1, u_value);
  EXPECT_DOUBLE_EQ(bases[1], basis);

  basis = OneBasisFun(degree, knots, 2, u_value);
  EXPECT_DOUBLE_EQ(bases[2], basis);
}

TEST(NURBS_Chapter2, SingleBasisMid) {
  constexpr double u_value_0 = 1.0;
  constexpr double u_value_1 = 1.5;
  constexpr uint32_t degree = 2;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 2, 2};

  uint32_t span_index = FindSpanParam(degree, knots, u_value_0, kTolerance);
  std::vector<double> bases =
      BasisFuns(span_index, u_value_0, degree, knots, kTolerance);
  double basis = OneBasisFun(degree, knots, 1, u_value_0);
  EXPECT_DOUBLE_EQ(bases[0], basis);
  basis = OneBasisFun(degree, knots, 2, u_value_0);
  EXPECT_DOUBLE_EQ(bases[1], basis);
  basis = OneBasisFun(degree, knots, 3, u_value_0);
  EXPECT_DOUBLE_EQ(bases[2], basis);

  span_index = FindSpanParam(degree, knots, u_value_1, kTolerance);
  bases = BasisFuns(span_index, u_value_1, degree, knots, kTolerance);
  basis = OneBasisFun(degree, knots, 1, u_value_1);
  EXPECT_DOUBLE_EQ(bases[0], basis);
  basis = OneBasisFun(degree, knots, 2, u_value_1);
  EXPECT_DOUBLE_EQ(bases[1], basis);
  basis = OneBasisFun(degree, knots, 3, u_value_1);
  EXPECT_DOUBLE_EQ(bases[2], basis);
}

TEST(NURBS_Chapter2, SingleBasisMax) {
  constexpr double u_value_0 = 2.0;
  constexpr double u_value_1 = 2.5;
  constexpr uint32_t degree = 2;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 2, 2};
  uint32_t span_index = FindSpanParam(degree, knots, u_value_0, kTolerance);
  std::vector<double> bases =
      BasisFuns(span_index, u_value_0, degree, knots, kTolerance);
  double basis = OneBasisFun(degree, knots, 1, u_value_0);
  EXPECT_DOUBLE_EQ(bases[0], basis);
  basis = OneBasisFun(degree, knots, 2, u_value_0);
  EXPECT_DOUBLE_EQ(bases[1], basis);
  basis = OneBasisFun(degree, knots, 3, u_value_0);
  EXPECT_DOUBLE_EQ(bases[2], basis);

  span_index = FindSpanParam(degree, knots, u_value_1, kTolerance);
  bases = BasisFuns(span_index, u_value_1, degree, knots, kTolerance);
  basis = OneBasisFun(degree, knots, 1, u_value_1);
  EXPECT_DOUBLE_EQ(bases[0], basis);
  basis = OneBasisFun(degree, knots, 2, u_value_1);
  EXPECT_DOUBLE_EQ(bases[1], basis);
  basis = OneBasisFun(degree, knots, 3, u_value_1);
  EXPECT_DOUBLE_EQ(bases[2], basis);
}



// TODO - Test the single function against the multi function
TEST(NURBS_Chapter2, SingleBasisDerivativeMin) {
  constexpr double u_value = 0.0;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 3, 4, 4, 5, 5, 5};
  constexpr uint32_t degree = 2;
  uint32_t span_index = FindSpanParam(degree, knots, u_value, kTolerance);
  EXPECT_EQ(span_index, 2);
  std::vector<std::vector<double>> bases_der =
      DersBasisFuns(span_index, u_value, degree, 2, knots);

  std::vector<double> bases =
      DersOneBasisFun(degree, knots, span_index, u_value, 2);

  EXPECT_DOUBLE_EQ(bases_der[0][2], bases[0]);
  EXPECT_DOUBLE_EQ(bases_der[1][2], bases[1]);
  EXPECT_DOUBLE_EQ(bases_der[2][2], bases[2]);
}

TEST(NURBS_Chapter2, SingleBasisDerivativeMid) {
  constexpr double u_value_0 = 2.5;
  constexpr double u_value_1 = 3.0;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 3, 4, 4, 5, 5, 5};
  constexpr uint32_t degree = 2;
  uint32_t span_index = FindSpanParam(degree, knots, u_value_0, kTolerance);
  EXPECT_EQ(span_index, 4);
  std::vector<std::vector<double>> bases_der =
      DersBasisFuns(span_index, u_value_0, degree, 2, knots);

  std::vector<double> bases =
      DersOneBasisFun(degree, knots, span_index, u_value_0, 2);

  EXPECT_DOUBLE_EQ(bases_der[0][2], bases[0]);
  EXPECT_DOUBLE_EQ(bases_der[1][2], bases[1]);
  EXPECT_DOUBLE_EQ(bases_der[2][2], bases[2]);

  span_index = FindSpanParam(degree, knots, u_value_1, kTolerance);
  EXPECT_EQ(span_index, 5);
  bases_der = DersBasisFuns(span_index, u_value_1, degree, 2, knots);

  bases = DersOneBasisFun(degree, knots, span_index, u_value_1, 2);

  EXPECT_DOUBLE_EQ(bases_der[0][2], bases[0]);
  EXPECT_DOUBLE_EQ(bases_der[1][2], bases[1]);
  EXPECT_DOUBLE_EQ(bases_der[2][2], bases[2]);
}

// Disabling because the algorithms provided for these 2 methods disagree on the
// output for the max derivative
TEST(NURBS_Chapter2, DISABLED_SingleBasisDerivativeMax) {
  constexpr double u_value_0 = 4.9;
  constexpr double u_value_1 = 5.0;
  const std::vector<double> knots = {0, 0, 0, 1, 2, 3, 4, 4, 5, 5, 5};
  constexpr uint32_t degree = 2;
  uint32_t span_index = FindSpanParam(degree, knots, u_value_0, kTolerance);
  EXPECT_EQ(span_index, 7);
  std::vector<std::vector<double>> bases_der =
      DersBasisFuns(span_index, u_value_0, degree, 2, knots);

  std::vector<double> bases =
      DersOneBasisFun(degree, knots, span_index, u_value_0, 2);

  EXPECT_DOUBLE_EQ(bases_der[0][2], bases[0]);
  EXPECT_DOUBLE_EQ(bases_der[1][2], bases[1]);
  EXPECT_DOUBLE_EQ(bases_der[2][2], bases[2]);

  span_index = FindSpanParam(degree, knots, u_value_1, kTolerance);
  EXPECT_EQ(span_index, 7);
  bases_der = DersBasisFuns(span_index, u_value_1, degree, 2, knots);

  bases = DersOneBasisFun(degree, knots, span_index, u_value_1, 2);

  EXPECT_DOUBLE_EQ(bases_der[2][0], bases[0]);
  EXPECT_DOUBLE_EQ(bases_der[2][1], bases[1]);
  EXPECT_DOUBLE_EQ(bases_der[2][2], bases[2]);
}
*/
