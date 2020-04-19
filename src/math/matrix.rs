#![allow(dead_code)]

use super::{Point3, Ray, Vec3, World};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<U = World, V = World> {
    m: [[f32; 4]; 4],
    _coord: PhantomData<(U, V)>,
}

impl<U, V> Matrix<U, V> {
    pub const fn new(m: [[f32; 4]; 4]) -> Self {
        Self {
            m,
            _coord: PhantomData,
        }
    }

    pub fn id() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            _coord: PhantomData,
        }
    }

    pub fn projection(aspect_ratio: f32, z_near: f32, z_far: f32, fov: f32) -> Self {
        let z_range = z_near - z_far;
        let tan_half_fov = (fov / 2.0).to_radians().tan();

        Self {
            m: [
                [1.0 / (tan_half_fov * aspect_ratio), 0.0, 0.0, 0.0],
                [0.0, 1.0 / tan_half_fov, 0.0, 0.0],
                [
                    0.0,
                    0.0,
                    (-z_near - z_far) / z_range,
                    2.0 * z_far * z_near / z_range,
                ],
                [0.0, 0.0, 1.0, 0.0],
            ],
            _coord: PhantomData,
        }
    }

    pub fn translation(dir: Vec3) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, dir.x()],
                [0.0, 1.0, 0.0, dir.y()],
                [0.0, 0.0, 1.0, dir.z()],
                [0.0, 0.0, 0.0, 1.0],
            ],
            _coord: PhantomData,
        }
    }

    pub fn inverse(&self) -> Matrix<V, U> {
        // Adapted from https://github.com/mmp/pbrt-v3/blob/master/src/core/transform.cpp#L82
        let mut indxc = [0; 4];
        let mut indxr = [0; 4];
        let mut ipiv = [0; 4];

        let swap_elems = |matrix: &mut Matrix<V, U>, a: usize, b, x: usize, y| {
            let temp = matrix.m[a][b];
            matrix.m[a][b] = matrix.m[x][y];
            matrix.m[x][y] = temp;
        };

        let mut inv = Matrix::<V, U>::id();
        std::mem::replace(&mut inv.m, self.m);

        for i in 0..4 {
            let mut irow = 0;
            let mut icol = 0;
            let mut big = 0.0;

            for j in 0..4 {
                if ipiv[j] != 1 {
                    for k in 0..4 {
                        if ipiv[k] == 0 {
                            if inv.m[j][k].abs() >= big {
                                big = inv.m[j][k].abs();
                                irow = j;
                                icol = k;
                            }
                        } else if ipiv[k] > 1 {
                            panic!("tried to invert singular matrix");
                        }
                    }
                }
            }

            ipiv[icol] += 1;

            if irow != icol {
                for k in 0..4 {
                    swap_elems(&mut inv, irow, k, icol, k);
                }
            }

            indxr[i] = irow;
            indxc[i] = icol;

            if inv.m[icol][icol] == 0.0 {
                panic!("tried to invert singular matrix");
            }

            let pivinv = 1.0 / inv.m[icol][icol];
            inv.m[icol][icol] = 1.0;
            for j in 0..4 {
                inv.m[icol][j] *= pivinv;
            }

            for j in 0..4 {
                if j != icol {
                    let save = inv.m[j][icol];
                    inv.m[j][icol] = 0.0;
                    for k in 0..4 {
                        inv.m[j][k] -= inv.m[icol][k] * save;
                    }
                }
            }
        }

        for j in (0..4).rev() {
            if indxr[j] != indxc[j] {
                for k in 0..4 {
                    swap_elems(&mut inv, k, indxr[j], k, indxc[j]);
                }
            }
        }

        inv
    }

    pub fn coordinate_system(u: Vec3<V>, v: Vec3<V>, w: Vec3<V>, point: Point3<V>) -> Self {
        Self {
            m: [
                [u.x(), v.x(), w.x(), point.x()],
                [u.x(), v.x(), w.x(), point.x()],
                [u.x(), v.x(), w.x(), point.x()],
                [0.0, 0.0, 0.0, 1.0],
            ],
            _coord: PhantomData,
        }
    }
}

impl<U, V> std::ops::Mul<Vec3<U>> for &'_ Matrix<U, V> {
    type Output = Vec3<V>;

    fn mul(self, other: Vec3<U>) -> Vec3<V> {
        Vec3::new(
            self.m[0][0] * other.x() + self.m[0][1] * other.y() + self.m[0][2] * other.z(),
            self.m[1][0] * other.x() + self.m[1][1] * other.y() + self.m[1][2] * other.z(),
            self.m[2][0] * other.x() + self.m[2][1] * other.y() + self.m[2][2] * other.z(),
        )
    }
}

impl<U, V> std::ops::Mul<Point3<U>> for &'_ Matrix<U, V> {
    type Output = Point3<V>;

    fn mul(self, other: Point3<U>) -> Point3<V> {
        let w = self.m[3][0] * other.x()
            + self.m[3][1] * other.y()
            + self.m[3][2] * other.z()
            + self.m[3][3];
        debug_assert!(w != 0.0);

        Point3::new(
            (self.m[0][0] * other.x()
                + self.m[0][1] * other.y()
                + self.m[0][2] * other.z()
                + self.m[0][3])
                / w,
            (self.m[1][0] * other.x()
                + self.m[1][1] * other.y()
                + self.m[1][2] * other.z()
                + self.m[1][3])
                / w,
            (self.m[2][0] * other.x()
                + self.m[2][1] * other.y()
                + self.m[2][2] * other.z()
                + self.m[2][3])
                / w,
        )
    }
}

impl<U, V> std::ops::Mul<Ray<U>> for &'_ Matrix<U, V> {
    type Output = Ray<V>;

    fn mul(self, other: Ray<U>) -> Ray<V> {
        Ray::new(self * other.o(), self * other.d())
    }
}

impl<U, V, W> std::ops::Mul<&'_ Matrix<U, V>> for &'_ Matrix<V, W> {
    type Output = Matrix<U, W>;

    fn mul(self, other: &Matrix<U, V>) -> Matrix<U, W> {
        let mut m = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }

        Matrix {
            m,
            _coord: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::World, *};

    #[test]
    fn test_matrix_multiply() {
        let id = Matrix::id();
        assert_eq!(&id * &id, id);

        let id2 = id.inverse();
        assert_eq!(id, id2);

        let v: Vec3<World> = Vec3::new(1.0, 2.0, 3.0);
        let p: Point3<World> = Point3::new(1.0, 2.0, 3.0);

        assert_eq!(v, &id * v);
        assert_eq!(p, &id * p);

        let trans = Matrix::translation(Vec3::new(2.0, 0.0, 1.0));
        let p1: Point3<World> = Point3::new(0.0, 0.0, 0.0);
        let p2: Point3<World> = Point3::new(2.0, 0.0, 1.0);

        assert_eq!(&trans * p1, p2);
    }
}
