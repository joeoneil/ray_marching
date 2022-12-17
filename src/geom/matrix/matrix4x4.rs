use crate::geom::points::vec4::Vec4;

use crate::geom::matrix::matrix3x3::Matrix3x3;
use std::ops;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Matrix4x4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Matrix4x4 {
    pub fn new(x: Vec4, y: Vec4, z: Vec4, w: Vec4) -> Matrix4x4 {
        Matrix4x4 { x, y, z, w }
    }

    pub fn identity() -> Matrix4x4 {
        Matrix4x4 {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn sub_matrix(&self, row: usize, col: usize) -> Matrix3x3 {
        assert!(row < 4);
        assert!(col < 4);
        let mut m = Matrix3x3::identity();
        let mut r = 0;
        for i in 0..4 {
            if i == row {
                continue;
            }
            let mut c = 0;
            for j in 0..4 {
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

    pub fn cofactor_matrix(&self) -> Matrix4x4 {
        let mut m = Matrix4x4::identity();
        for i in 0..4 {
            for j in 0..4 {
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

    pub fn adjugate(&self) -> Matrix4x4 {
        self.cofactor_matrix().transpose()
    }

    pub fn determinant(&self) -> f32 {
        self.x.x * self.sub_matrix(0, 0).determinant()
            - self.x.y * self.sub_matrix(0, 1).determinant()
            + self.x.z * self.sub_matrix(0, 2).determinant()
            - self.x.w * self.sub_matrix(0, 3).determinant()
    }

    pub fn inverse(&self) -> Matrix4x4 {
        let det = self.determinant();
        if det == 0.0 {
            panic!("Matrix is not invertible");
        }
        self.adjugate() * (1.0 / det)
    }

    pub fn transpose(&self) -> Matrix4x4 {
        Matrix4x4 {
            x: Vec4::new(self.x.x, self.y.x, self.z.x, self.w.x),
            y: Vec4::new(self.x.y, self.y.y, self.z.y, self.w.y),
            z: Vec4::new(self.x.z, self.y.z, self.z.z, self.w.z),
            w: Vec4::new(self.x.w, self.y.w, self.z.w, self.w.w),
        }
    }
}

impl Default for Matrix4x4 {
    fn default() -> Self {
        Matrix4x4::identity()
    }
}

impl ops::Add for Matrix4x4 {
    type Output = Matrix4x4;

    fn add(self, rhs: Matrix4x4) -> Matrix4x4 {
        Matrix4x4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl ops::AddAssign for Matrix4x4 {
    fn add_assign(&mut self, rhs: Matrix4x4) {
        *self = *self + rhs;
    }
}

impl ops::Sub for Matrix4x4 {
    type Output = Matrix4x4;

    fn sub(self, rhs: Matrix4x4) -> Matrix4x4 {
        Matrix4x4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl ops::SubAssign for Matrix4x4 {
    fn sub_assign(&mut self, rhs: Matrix4x4) {
        *self = *self - rhs;
    }
}

impl ops::Mul for Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, rhs: Matrix4x4) -> Matrix4x4 {
        Matrix4x4 {
            x: Vec4::new(
                self.x.x * rhs.x.x + self.x.y * rhs.y.x + self.x.z * rhs.z.x + self.x.w * rhs.w.x,
                self.x.x * rhs.x.y + self.x.y * rhs.y.y + self.x.z * rhs.z.y + self.x.w * rhs.w.y,
                self.x.x * rhs.x.z + self.x.y * rhs.y.z + self.x.z * rhs.z.z + self.x.w * rhs.w.z,
                self.x.x * rhs.x.w + self.x.y * rhs.y.w + self.x.z * rhs.z.w + self.x.w * rhs.w.w,
            ),
            y: Vec4::new(
                self.y.x * rhs.x.x + self.y.y * rhs.y.x + self.y.z * rhs.z.x + self.y.w * rhs.w.x,
                self.y.x * rhs.x.y + self.y.y * rhs.y.y + self.y.z * rhs.z.y + self.y.w * rhs.w.y,
                self.y.x * rhs.x.z + self.y.y * rhs.y.z + self.y.z * rhs.z.z + self.y.w * rhs.w.z,
                self.y.x * rhs.x.w + self.y.y * rhs.y.w + self.y.z * rhs.z.w + self.y.w * rhs.w.w,
            ),
            z: Vec4::new(
                self.z.x * rhs.x.x + self.z.y * rhs.y.x + self.z.z * rhs.z.x + self.z.w * rhs.w.x,
                self.z.x * rhs.x.y + self.z.y * rhs.y.y + self.z.z * rhs.z.y + self.z.w * rhs.w.y,
                self.z.x * rhs.x.z + self.z.y * rhs.y.z + self.z.z * rhs.z.z + self.z.w * rhs.w.z,
                self.z.x * rhs.x.w + self.z.y * rhs.y.w + self.z.z * rhs.z.w + self.z.w * rhs.w.w,
            ),
            w: Vec4::new(
                self.w.x * rhs.x.x + self.w.y * rhs.y.x + self.w.z * rhs.z.x + self.w.w * rhs.w.x,
                self.w.x * rhs.x.y + self.w.y * rhs.y.y + self.w.z * rhs.z.y + self.w.w * rhs.w.y,
                self.w.x * rhs.x.z + self.w.y * rhs.y.z + self.w.z * rhs.z.z + self.w.w * rhs.w.z,
                self.w.x * rhs.x.w + self.w.y * rhs.y.w + self.w.z * rhs.z.w + self.w.w * rhs.w.w,
            ),
        }
    }
}

impl ops::Mul<Vec4> for Matrix4x4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.x.x * rhs.x + self.x.y * rhs.y + self.x.z * rhs.z + self.x.w * rhs.w,
            self.y.x * rhs.x + self.y.y * rhs.y + self.y.z * rhs.z + self.y.w * rhs.w,
            self.z.x * rhs.x + self.z.y * rhs.y + self.z.z * rhs.z + self.z.w * rhs.w,
            self.w.x * rhs.x + self.w.y * rhs.y + self.w.z * rhs.z + self.w.w * rhs.w,
        )
    }
}

impl ops::Mul<f32> for Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, rhs: f32) -> Matrix4x4 {
        Matrix4x4 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl ops::MulAssign for Matrix4x4 {
    fn mul_assign(&mut self, rhs: Matrix4x4) {
        *self = *self * rhs;
    }
}

impl ops::MulAssign<f32> for Matrix4x4 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl ops::Div<f32> for Matrix4x4 {
    type Output = Matrix4x4;

    fn div(self, rhs: f32) -> Matrix4x4 {
        Matrix4x4 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl ops::DivAssign<f32> for Matrix4x4 {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl ops::Neg for Matrix4x4 {
    type Output = Matrix4x4;

    fn neg(self) -> Matrix4x4 {
        Matrix4x4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl ops::Index<usize> for Matrix4x4 {
    type Output = Vec4;

    fn index(&self, index: usize) -> &Vec4 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl ops::IndexMut<usize> for Matrix4x4 {
    fn index_mut(&mut self, index: usize) -> &mut Vec4 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Index out of bounds"),
        }
    }
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_matrix4x4_determinant() {
        let m = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(m.determinant(), 0.0);
    }

    #[test]
    fn test_matrix4x4_inverse() {
        // Construct a matrix with non-zero determinant
        let m = Matrix4x4::new(
            Vec4::new(1.0, 8.0, 14.0, 3.0),
            Vec4::new(9.0, 5.0, 6.0, 13.0),
            Vec4::new(15.0, 2.0, 4.0, 11.0),
            Vec4::new(7.0, 12.0, 16.0, 10.0),
        );
        assert_eq!(m.determinant(), 2646.0);
        let m_inv = m.inverse();
        assert_eq!(m * m_inv, Matrix4x4::identity());
    }

    #[test]
    #[should_panic]
    fn test_matrix4x4_inverse_panic() {
        let m = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        m.inverse();
    }

    #[test]
    fn test_matrix4x4_transpose() {
        let m = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(
            m.transpose(),
            Matrix4x4::new(
                Vec4::new(1.0, 5.0, 9.0, 13.0),
                Vec4::new(2.0, 6.0, 10.0, 14.0),
                Vec4::new(3.0, 7.0, 11.0, 15.0),
                Vec4::new(4.0, 8.0, 12.0, 16.0),
            )
        );
    }

    #[test]
    fn test_matrix4x4_add() {
        let m1 = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        let m2 = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(
            m1 + m2,
            Matrix4x4::new(
                Vec4::new(2.0, 4.0, 6.0, 8.0),
                Vec4::new(10.0, 12.0, 14.0, 16.0),
                Vec4::new(18.0, 20.0, 22.0, 24.0),
                Vec4::new(26.0, 28.0, 30.0, 32.0),
            )
        );
    }

    #[test]
    fn test_matrix4x4_sub() {
        let m1 = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        let m2 = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(
            m1 - m2,
            Matrix4x4::new(
                Vec4::new(0.0, 0.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 0.0),
            )
        );
    }

    #[test]
    fn test_matrix4x4_mul() {
        let m1 = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        let m2 = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(
            m1 * m2,
            Matrix4x4::new(
                Vec4::new(90.0, 100.0, 110.0, 120.0),
                Vec4::new(202.0, 228.0, 254.0, 280.0),
                Vec4::new(314.0, 356.0, 398.0, 440.0),
                Vec4::new(426.0, 484.0, 542.0, 600.0),
            )
        );
    }

    #[test]
    fn test_matrix4x4_mul_vec4() {
        let m = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m * v, Vec4::new(30.0, 70.0, 110.0, 150.0));
    }

    #[test]
    fn test_matrix4x4_mul_scalar() {
        let m = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(
            m * 2.0,
            Matrix4x4::new(
                Vec4::new(2.0, 4.0, 6.0, 8.0),
                Vec4::new(10.0, 12.0, 14.0, 16.0),
                Vec4::new(18.0, 20.0, 22.0, 24.0),
                Vec4::new(26.0, 28.0, 30.0, 32.0),
            )
        );
    }

    #[test]
    fn test_matrix4x4_div_scalar() {
        let m = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(
            m / 2.0,
            Matrix4x4::new(
                Vec4::new(0.5, 1.0, 1.5, 2.0),
                Vec4::new(2.5, 3.0, 3.5, 4.0),
                Vec4::new(4.5, 5.0, 5.5, 6.0),
                Vec4::new(6.5, 7.0, 7.5, 8.0),
            )
        );
    }

    #[test]
    fn test_matrix4x4_neg() {
        let m = Matrix4x4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(
            -m,
            Matrix4x4::new(
                Vec4::new(-1.0, -2.0, -3.0, -4.0),
                Vec4::new(-5.0, -6.0, -7.0, -8.0),
                Vec4::new(-9.0, -10.0, -11.0, -12.0),
                Vec4::new(-13.0, -14.0, -15.0, -16.0),
            )
        );
    }
}
