#![allow(dead_code)]

use super::{Point3, Ray, Vec3};

#[derive(Debug, Clone, PartialEq)]
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
                [0.0, 0.0, 0.0, 1.0],
            ],
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
        }
    }

    pub fn translation(dir: Vec3) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, dir.x],
                [0.0, 1.0, 0.0, dir.y],
                [0.0, 0.0, 1.0, dir.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn inverse(&self) -> Self {
        // Adapted from https://github.com/mmp/pbrt-v3/blob/master/src/core/transform.cpp#L82
        let mut indxc = [0; 4];
        let mut indxr = [0; 4];
        let mut ipiv = [0; 4];

        let swap_elems = |matrix: &mut Matrix, a: usize, b, x: usize, y| {
            let temp = matrix.m[a][b];
            matrix.m[a][b] = matrix.m[x][y];
            matrix.m[x][y] = temp;
        };

        let mut inv = self.clone();

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
}

impl<S> std::ops::Mul<Vec3<S>> for &'_ Matrix {
    type Output = Vec3<S>;

    fn mul(self, other: Vec3<S>) -> Vec3<S> {
        Vec3::new(
            self.m[0][0] * other.x + self.m[0][1] * other.y + self.m[0][2] * other.z,
            self.m[1][0] * other.x + self.m[1][1] * other.y + self.m[1][2] * other.z,
            self.m[2][0] * other.x + self.m[2][1] * other.y + self.m[2][2] * other.z,
        )
    }
}

impl<S> std::ops::Mul<Point3<S>> for &'_ Matrix {
    type Output = Point3<S>;

    fn mul(self, other: Point3<S>) -> Point3<S> {
        let w =
            self.m[3][0] * other.x + self.m[3][1] * other.y + self.m[3][2] * other.z + self.m[3][3];
        debug_assert!(w != 0.0);

        Point3::new(
            (self.m[0][0] * other.x
                + self.m[0][1] * other.y
                + self.m[0][2] * other.z
                + self.m[0][3])
                / w,
            (self.m[1][0] * other.x
                + self.m[1][1] * other.y
                + self.m[1][2] * other.z
                + self.m[1][3])
                / w,
            (self.m[2][0] * other.x
                + self.m[2][1] * other.y
                + self.m[2][2] * other.z
                + self.m[2][3])
                / w,
        )
    }
}

impl<S> std::ops::Mul<Ray<S>> for &'_ Matrix {
    type Output = Ray<S>;

    fn mul(self, other: Ray<S>) -> Ray<S> {
        Ray::new(self * other.o, self * other.d)
    }
}

impl std::ops::Mul<&'_ Matrix> for &'_ Matrix {
    type Output = Matrix;

    fn mul(self, other: &Matrix) -> Matrix {
        let mut m = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }

        Matrix { m }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Global, *};

    #[test]
    fn test_matrix_multiply() {
        let id = Matrix::id();
        assert_eq!(&id * &id, id);

        let id2 = id.inverse();
        assert_eq!(id, id2);

        let v: Vec3<Global> = Vec3::new(1.0, 2.0, 3.0);
        let p: Point3<Global> = Point3::new(1.0, 2.0, 3.0);

        assert_eq!(v, &id * v);
        assert_eq!(p, &id * p);

        let trans = Matrix::translation(Vec3::new(2.0, 0.0, 1.0));
        let p1: Point3<Global> = Point3::new(0.0, 0.0, 0.0);
        let p2: Point3<Global> = Point3::new(2.0, 0.0, 1.0);

        assert_eq!(&trans * p1, p2);
    }
}
