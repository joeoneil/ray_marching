use crate::geom::points::vec3::Vec3;

use crate::geom::matrix::matrix2x2::Matrix2x2;
use std::ops;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Matrix3x3 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl Matrix3x3 {
    pub fn new(x: Vec3, y: Vec3, z: Vec3) -> Matrix3x3 {
        Matrix3x3 { x, y, z }
    }

    pub fn identity() -> Matrix3x3 {
        Matrix3x3 {
            x: Vec3::new(1.0, 0.0, 0.0),
            y: Vec3::new(0.0, 1.0, 0.0),
            z: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn from_alpha(alpha: f32) -> Matrix3x3 {
        let (s, c) = alpha.sin_cos();
        Matrix3x3 {
            x: Vec3::new(1.0, 0.0, 0.0),
            y: Vec3::new(0.0, c, s),
            z: Vec3::new(0.0, -s, c),
        }
    }

    pub fn from_beta(beta: f32) -> Matrix3x3 {
        let (s, c) = beta.sin_cos();
        Matrix3x3 {
            x: Vec3::new(c, 0.0, -s),
            y: Vec3::new(0.0, 1.0, 0.0),
            z: Vec3::new(s, 0.0, c),
        }
    }

    pub fn from_gamma(gamma: f32) -> Matrix3x3 {
        let (s, c) = gamma.sin_cos();
        Matrix3x3 {
            x: Vec3::new(c, s, 0.0),
            y: Vec3::new(-s, c, 0.0),
            z: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn sub_matrix(&self, row: usize, col: usize) -> Matrix2x2 {
        assert!(row < 3);
        assert!(col < 3);
        let mut m = Matrix2x2::identity();
        let mut r = 0;
        for i in 0..3 {
            if i == row {
                continue;
            }
            let mut c = 0;
            for j in 0..3 {
                if j == col {
                    continue;
                }
                m[r][c] = self[i][j];
                c += 1;
            }
            r += 1;
        }
        m
    }

    pub fn cofactor_matrix(&self) -> Matrix3x3 {
        let mut m = Matrix3x3::identity();
        for i in 0..3 {
            for j in 0..3 {
                let sub = self.sub_matrix(i, j);
                let cofactor = sub.determinant();
                m[i][j] = if (i + j) % 2 == 0 {
                    cofactor
                } else {
                    -cofactor
                };
            }
        }
        m
    }

    pub fn adjugate(&self) -> Matrix3x3 {
        self.cofactor_matrix().transpose()
    }

    pub fn determinant(&self) -> f32 {
        self.x.x * self.sub_matrix(0, 0).determinant()
            - self.x.y * self.sub_matrix(0, 1).determinant()
            + self.x.z * self.sub_matrix(0, 2).determinant()
    }

    pub fn inverse(&self) -> Matrix3x3 {
        let det = self.determinant();
        if det == 0.0 {
            panic!("Matrix is not invertible");
        }
        self.adjugate() * (1.0 / det)
    }

    pub fn transpose(&self) -> Matrix3x3 {
        Matrix3x3 {
            x: Vec3::new(self.x.x, self.y.x, self.z.x),
            y: Vec3::new(self.x.y, self.y.y, self.z.y),
            z: Vec3::new(self.x.z, self.y.z, self.z.z),
        }
    }
}

impl Default for Matrix3x3 {
    fn default() -> Self {
        Matrix3x3::identity()
    }
}

impl ops::Add for Matrix3x3 {
    type Output = Matrix3x3;

    fn add(self, other: Matrix3x3) -> Matrix3x3 {
        Matrix3x3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ops::AddAssign for Matrix3x3 {
    fn add_assign(&mut self, other: Matrix3x3) {
        *self = *self + other;
    }
}

impl ops::Sub for Matrix3x3 {
    type Output = Matrix3x3;

    fn sub(self, other: Matrix3x3) -> Matrix3x3 {
        Matrix3x3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl ops::SubAssign for Matrix3x3 {
    fn sub_assign(&mut self, other: Matrix3x3) {
        *self = *self - other;
    }
}

impl ops::Mul for Matrix3x3 {
    type Output = Self;

    fn mul(self, other: Matrix3x3) -> Matrix3x3 {
        Matrix3x3 {
            x: Vec3::new(
                self.x.x * other.x.x + self.x.y * other.y.x + self.x.z * other.z.x,
                self.x.x * other.x.y + self.x.y * other.y.y + self.x.z * other.z.y,
                self.x.x * other.x.z + self.x.y * other.y.z + self.x.z * other.z.z,
            ),
            y: Vec3::new(
                self.y.x * other.x.x + self.y.y * other.y.x + self.y.z * other.z.x,
                self.y.x * other.x.y + self.y.y * other.y.y + self.y.z * other.z.y,
                self.y.x * other.x.z + self.y.y * other.y.z + self.y.z * other.z.z,
            ),
            z: Vec3::new(
                self.z.x * other.x.x + self.z.y * other.y.x + self.z.z * other.z.x,
                self.z.x * other.x.y + self.z.y * other.y.y + self.z.z * other.z.y,
                self.z.x * other.x.z + self.z.y * other.y.z + self.z.z * other.z.z,
            ),
        }
    }
}

impl ops::Mul<Vec3> for Matrix3x3 {
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.x * other.x + self.x.y * other.y + self.x.z * other.z,
            self.y.x * other.x + self.y.y * other.y + self.y.z * other.z,
            self.z.x * other.x + self.z.y * other.y + self.z.z * other.z,
        )
    }
}

impl ops::Mul<f32> for Matrix3x3 {
    type Output = Matrix3x3;

    fn mul(self, other: f32) -> Matrix3x3 {
        Matrix3x3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl ops::MulAssign for Matrix3x3 {
    fn mul_assign(&mut self, other: Matrix3x3) {
        *self = *self * other;
    }
}

impl ops::MulAssign<f32> for Matrix3x3 {
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}

impl ops::Div<f32> for Matrix3x3 {
    type Output = Matrix3x3;

    fn div(self, other: f32) -> Matrix3x3 {
        Matrix3x3 {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl ops::DivAssign<f32> for Matrix3x3 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other;
    }
}

impl ops::Neg for Matrix3x3 {
    type Output = Matrix3x3;

    fn neg(self) -> Matrix3x3 {
        Matrix3x3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Index<usize> for Matrix3x3 {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Vec3 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl ops::IndexMut<usize> for Matrix3x3 {
    fn index_mut(&mut self, index: usize) -> &mut Vec3 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds"),
        }
    }
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_matrix3x3_determinant() {
        let m = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );

        assert_eq!(m.determinant(), 0.0);
    }

    #[test]
    fn test_matrix3x3_inverse() {
        let m: Matrix3x3 = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.0, 1.0, 4.0),
            Vec3::new(5.0, 6.0, 0.0),
        );

        let m_inv = m.inverse();
        let m_inv_expected = Matrix3x3::new(
            Vec3::new(-24.0, 18.0, 5.0),
            Vec3::new(20.0, -15.0, -4.0),
            Vec3::new(-5.0, 4.0, 1.0),
        );

        assert_eq!(m_inv, m_inv_expected);
    }

    #[test]
    #[should_panic]
    fn test_matrix3x3_inverse_panic() {
        let m: Matrix3x3 = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        m.inverse();
    }

    #[test]
    fn test_matrix3x3_transpose() {
        let m: Matrix3x3 = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );

        let m_t = m.transpose();
        let m_t_expected = Matrix3x3::new(
            Vec3::new(1.0, 4.0, 7.0),
            Vec3::new(2.0, 5.0, 8.0),
            Vec3::new(3.0, 6.0, 9.0),
        );

        assert_eq!(m_t, m_t_expected);
    }

    #[test]
    fn test_matrix3x3_add() {
        let m1 = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let m2 = Matrix3x3::new(
            Vec3::new(9.0, 8.0, 7.0),
            Vec3::new(6.0, 5.0, 4.0),
            Vec3::new(3.0, 2.0, 1.0),
        );
        let m3 = m1 + m2;
        let m3_expected = Matrix3x3::new(
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(10.0, 10.0, 10.0),
        );
        assert_eq!(m3, m3_expected);
    }

    #[test]
    fn test_matrix3x3_sub() {
        let m1 = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let m2 = Matrix3x3::new(
            Vec3::new(9.0, 8.0, 7.0),
            Vec3::new(6.0, 5.0, 4.0),
            Vec3::new(3.0, 2.0, 1.0),
        );
        let m3 = m1 - m2;
        let m3_expected = Matrix3x3::new(
            Vec3::new(-8.0, -6.0, -4.0),
            Vec3::new(-2.0, 0.0, 2.0),
            Vec3::new(4.0, 6.0, 8.0),
        );
        assert_eq!(m3, m3_expected);
    }

    #[test]
    fn test_matrix3x3_mul() {
        let m1 = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let m2 = Matrix3x3::new(
            Vec3::new(9.0, 8.0, 7.0),
            Vec3::new(6.0, 5.0, 4.0),
            Vec3::new(3.0, 2.0, 1.0),
        );
        let m3 = m1 * m2;
        let m3_expected = Matrix3x3::new(
            Vec3::new(30.0, 24.0, 18.0),
            Vec3::new(84.0, 69.0, 54.0),
            Vec3::new(138.0, 114.0, 90.0),
        );
        assert_eq!(m3, m3_expected);
    }

    #[test]
    fn test_matrix3x3_mul_vec3() {
        let m = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let v = Vec3::new(1.0, 2.0, 3.0);
        let v2 = m * v;
        let v2_expected = Vec3::new(14.0, 32.0, 50.0);
        assert_eq!(v2, v2_expected);
    }

    #[test]
    fn test_matrix3x3_mul_scalar() {
        let m = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let m2 = m * 2.0;
        let m2_expected = Matrix3x3::new(
            Vec3::new(2.0, 4.0, 6.0),
            Vec3::new(8.0, 10.0, 12.0),
            Vec3::new(14.0, 16.0, 18.0),
        );
        assert_eq!(m2, m2_expected);
    }

    #[test]
    fn test_matrix3x3_div_scalar() {
        let m = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let m2 = m / 2.0;
        let m2_expected = Matrix3x3::new(
            Vec3::new(0.5, 1.0, 1.5),
            Vec3::new(2.0, 2.5, 3.0),
            Vec3::new(3.5, 4.0, 4.5),
        );
        assert_eq!(m2, m2_expected);
    }

    #[test]
    fn test_matrix3x3_neg() {
        let m = Matrix3x3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let m2 = -m;
        let m2_expected = Matrix3x3::new(
            Vec3::new(-1.0, -2.0, -3.0),
            Vec3::new(-4.0, -5.0, -6.0),
            Vec3::new(-7.0, -8.0, -9.0),
        );
        assert_eq!(m2, m2_expected);
    }
}
