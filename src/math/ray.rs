#![allow(dead_code)]

use super::{Point3, Vec3, World};

pub const RAY_EPSILON: f32 = 0.001;

#[derive(Debug, Clone)]
pub struct Ray<System = World> {
    o: Point3<System>,
    d: Vec3<System>,
}

impl<S> Ray<S> {
    pub fn new(o: Point3<S>, d: Vec3<S>) -> Self {
        Self {
            o,
            d: d.normalize(),
        }
    }

    pub fn spawn(o: Point3<S>, d: Vec3<S>, normal: Vec3<S>) -> Self {
        let o = if d.dot(normal) < 0.0 {
            offset_origin(o, -normal)
        } else {
            offset_origin(o, normal)
        };

        Self {
            o,
            d: d.normalize(),
        }
    }

    pub fn spawn_to(o: Point3<S>, p: Point3<S>, normal: Vec3<S>) -> Self {
        let o = if (p - o).dot(normal) < 0.0 {
            offset_origin(o, -normal)
        } else {
            offset_origin(o, normal)
        };

        Self {
            o,
            d: (p - o).normalize(),
        }
    }

    pub fn o(&self) -> Point3<S> {
        self.o
    }

    pub fn d(&self) -> Vec3<S> {
        self.d
    }

    pub fn point_at(&self, t: f32) -> Point3<S> {
        self.o + self.d * t
    }
}

// https://link.springer.com/content/pdf/10.1007%2F978-1-4842-4427-2_6.pdf
pub fn offset_origin<S>(p: Point3<S>, n: Vec3<S>) -> Point3<S> {
    const ORIGIN: f32 = 1.0 / 32.0;
    const FLOAT_SCALE: f32 = 1.0 / 65536.0;
    const INT_SCALE: f32 = 256.0;

    // TODO: Are we sure that these float -> int casts are sound?
    // TODO: SIMD this
    let of_i = [
        (INT_SCALE * n.x()) as i32,
        (INT_SCALE * n.y()) as i32,
        (INT_SCALE * n.z()) as i32,
    ];

    let p_i = [
        f32::from_bits((p.x().to_bits() as i32 + if p.x() < 0.0 { -of_i[0] } else { of_i[0] }) as u32),
        f32::from_bits((p.y().to_bits() as i32 + if p.y() < 0.0 { -of_i[1] } else { of_i[1] }) as u32),
        f32::from_bits((p.z().to_bits() as i32 + if p.z() < 0.0 { -of_i[2] } else { of_i[2] }) as u32),
    ];

    Point3::new(
        if p.x.abs() < ORIGIN { p.x() + FLOAT_SCALE * n.x() } else { p_i[0] },
        if p.y.abs() < ORIGIN { p.y() + FLOAT_SCALE * n.y() } else { p_i[1] },
        if p.z.abs() < ORIGIN { p.z() + FLOAT_SCALE * n.z() } else { p_i[2] },
    )
}
