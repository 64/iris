#![allow(dead_code)]

use super::{Point3, Vec3};

pub struct Matrix {
    m: [[f32; 4]; 4],
}

impl Matrix {
    pub fn id() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
        }
    }

    pub fn inverse() -> Self {
        todo!()
    }
}

impl<S> std::ops::Mul<Vec3<S>> for Matrix {
    type Output = Vec3<S>;

    fn mul(self, other: Vec3<S>) -> Vec3<S> {
        let w = self.m[3][0] * other.x + self.m[3][1] * other.y + self.m[3][2] * other.z;
        debug_assert!(w != 0.0);

        Vec3::new(
            self.m[0][0] * other.x + self.m[0][1] * other.y + self.m[0][2] * other.z,
            self.m[1][0] * other.x + self.m[1][1] * other.y + self.m[1][2] * other.z,
            self.m[2][0] * other.x + self.m[2][1] * other.y + self.m[2][2] * other.z,
        ) / w
    }
}

impl<S> std::ops::Mul<Point3<S>> for Matrix {
    type Output = Point3<S>;

    fn mul(self, other: Point3<S>) -> Point3<S> {
        let w = self.m[3][0] * other.x + self.m[3][1] * other.y + self.m[3][2] * other.z + self.m[3][3];
        debug_assert!(w != 0.0);

        Point3::new(
            (self.m[0][0] * other.x + self.m[0][1] * other.y + self.m[0][2] * other.z + self.m[0][3]) / w,
            (self.m[1][0] * other.x + self.m[1][1] * other.y + self.m[1][2] * other.z + self.m[1][3]) / w,
            (self.m[2][0] * other.x + self.m[2][1] * other.y + self.m[2][2] * other.z + self.m[2][3]) / w,
        )
    }
}
