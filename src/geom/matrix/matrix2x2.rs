use crate::geom::points::vec2::Vec2;

use std::ops;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Matrix2x2 {
    pub x: Vec2,
    pub y: Vec2,
}

impl Matrix2x2 {
    pub fn new(x: Vec2, y: Vec2) -> Matrix2x2 {
        Matrix2x2 { x, y }
    }

    pub fn identity() -> Matrix2x2 {
        Matrix2x2 {
            x: Vec2::new(1.0, 0.0),
            y: Vec2::new(0.0, 1.0),
        }
    }

    pub fn from_angle(theta: f32) -> Matrix2x2 {
        let (s, c) = theta.sin_cos();
        Matrix2x2 {
            x: Vec2::new(c, s),
            y: Vec2::new(-s, c),
        }
    }

    pub fn determinant(&self) -> f32 {
        self.x.x * self.y.y - self.x.y * self.y.x
    }

    pub fn inverse(&self) -> Matrix2x2 {
        let det = self.determinant();
        if det == 0.0 {
            panic!("Matrix is not invertible");
        }
        Matrix2x2 {
            x: Vec2::new(self.y.y / det, -self.x.y / det),
            y: Vec2::new(-self.y.x / det, self.x.x / det),
        }
    }

    pub fn transpose(&self) -> Matrix2x2 {
        Matrix2x2 {
            x: Vec2::new(self.x.x, self.y.x),
            y: Vec2::new(self.x.y, self.y.y),
        }
    }
}

impl Default for Matrix2x2 {
    fn default() -> Self {
        Matrix2x2::identity()
    }
}

impl ops::Add for Matrix2x2 {
    type Output = Matrix2x2;

    fn add(self, other: Matrix2x2) -> Matrix2x2 {
        Matrix2x2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::AddAssign for Matrix2x2 {
    fn add_assign(&mut self, other: Matrix2x2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl ops::Sub for Matrix2x2 {
    type Output = Matrix2x2;

    fn sub(self, other: Matrix2x2) -> Matrix2x2 {
        Matrix2x2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::SubAssign for Matrix2x2 {
    fn sub_assign(&mut self, other: Matrix2x2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl ops::Mul for Matrix2x2 {
    type Output = Matrix2x2;

    fn mul(self, other: Matrix2x2) -> Matrix2x2 {
        Matrix2x2 {
            x: Vec2::new(
                self.x.x * other.x.x + self.x.y * other.y.x,
                self.x.x * other.x.y + self.x.y * other.y.y,
            ),
            y: Vec2::new(
                self.y.x * other.x.x + self.y.y * other.y.x,
                self.y.x * other.x.y + self.y.y * other.y.y,
            ),
        }
    }
}

impl ops::Mul<Vec2> for Matrix2x2 {
    type Output = Vec2;

    fn mul(self, other: Vec2) -> Vec2 {
        Vec2::new(
            self.x.x * other.x + self.x.y * other.y,
            self.y.x * other.x + self.y.y * other.y,
        )
    }
}

impl ops::Mul<f32> for Matrix2x2 {
    type Output = Matrix2x2;

    fn mul(self, other: f32) -> Matrix2x2 {
        Matrix2x2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl ops::MulAssign for Matrix2x2 {
    fn mul_assign(&mut self, other: Matrix2x2) {
        *self = *self * other;
    }
}

impl ops::MulAssign<f32> for Matrix2x2 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
    }
}

impl ops::Div<f32> for Matrix2x2 {
    type Output = Matrix2x2;

    fn div(self, other: f32) -> Matrix2x2 {
        Matrix2x2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl ops::DivAssign<f32> for Matrix2x2 {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
    }
}

impl ops::Neg for Matrix2x2 {
    type Output = Matrix2x2;

    fn neg(self) -> Matrix2x2 {
        Matrix2x2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl ops::Index<usize> for Matrix2x2 {
    type Output = Vec2;

    fn index(&self, index: usize) -> &Vec2 {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl ops::IndexMut<usize> for Matrix2x2 {
    fn index_mut(&mut self, index: usize) -> &mut Vec2 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Index out of bounds"),
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_matrix2x2_determinant() {
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        assert_eq!(m.determinant(), -2.0);
    }

    #[test]
    fn test_matrix2x2_inverse() {
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let inv = m.inverse();
        assert_eq!(inv.x.x, -2.0);
        assert_eq!(inv.x.y, 1.0);
        assert_eq!(inv.y.x, 1.5);
        assert_eq!(inv.y.y, -0.5);
    }

    #[test]
    #[should_panic]
    fn test_matrix2x2_inverse_panic() {
        // determinant is 0
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(2.0, 4.0));
        m.inverse();
    }

    #[test]
    fn test_matrix2x2_transpose() {
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let t = m.transpose();
        assert_eq!(t.x.x, 1.0);
        assert_eq!(t.x.y, 3.0);
        assert_eq!(t.y.x, 2.0);
        assert_eq!(t.y.y, 4.0);
    }

    #[test]
    fn test_matrix2x2_add() {
        let m1 = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let m2 = Matrix2x2::new(Vec2::new(5.0, 6.0), Vec2::new(7.0, 8.0));
        let m3 = m1 + m2;
        assert_eq!(m3.x.x, 6.0);
        assert_eq!(m3.x.y, 8.0);
        assert_eq!(m3.y.x, 10.0);
        assert_eq!(m3.y.y, 12.0);
    }

    #[test]
    fn test_matrix2x2_sub() {
        let m1 = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let m2 = Matrix2x2::new(Vec2::new(5.0, 6.0), Vec2::new(7.0, 8.0));
        let m3 = m1 - m2;
        assert_eq!(m3.x.x, -4.0);
        assert_eq!(m3.x.y, -4.0);
        assert_eq!(m3.y.x, -4.0);
        assert_eq!(m3.y.y, -4.0);
    }

    #[test]
    fn test_matrix2x2_mul() {
        let m1 = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let m2 = Matrix2x2::new(Vec2::new(5.0, 6.0), Vec2::new(7.0, 8.0));
        let m3 = m1 * m2;
        assert_eq!(m3.x.x, 19.0);
        assert_eq!(m3.x.y, 22.0);
        assert_eq!(m3.y.x, 43.0);
        assert_eq!(m3.y.y, 50.0);
    }

    #[test]
    fn test_matrix2x2_mul_vec2() {
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let v = Vec2::new(5.0, 6.0);
        let v2 = m * v;
        assert_eq!(v2.x, 17.0);
        assert_eq!(v2.y, 39.0);
    }

    #[test]
    fn test_matrix2x2_mul_scalar() {
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let m2 = m * 2.0;
        assert_eq!(m2.x.x, 2.0);
        assert_eq!(m2.x.y, 4.0);
        assert_eq!(m2.y.x, 6.0);
        assert_eq!(m2.y.y, 8.0);
    }

    #[test]
    fn test_matrix2x2_div_scalar() {
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let m2 = m / 2.0;
        assert_eq!(m2.x.x, 0.5);
        assert_eq!(m2.x.y, 1.0);
        assert_eq!(m2.y.x, 1.5);
        assert_eq!(m2.y.y, 2.0);
    }

    #[test]
    fn test_matrix2x2_neg() {
        let m = Matrix2x2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0));
        let m2 = -m;
        assert_eq!(m2.x.x, -1.0);
        assert_eq!(m2.x.y, -2.0);
        assert_eq!(m2.y.x, -3.0);
        assert_eq!(m2.y.y, -4.0);
    }
}
