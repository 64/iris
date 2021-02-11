use enum_dispatch::enum_dispatch;

use crate::{
    bsdf::Bsdf,
    math::{Point3, Ray, Shading, Vec3, World},
    sampling::Sampler,
    spectrum::Spectrum,
    types::PrimIndex,
};

mod sphere;
pub use sphere::Sphere;

#[derive(Debug)]
pub struct Intersection {
    pub point: Point3,
    pub normal: Vec3,
    pub tangeant: Vec3,
    pub bitangeant: Vec3,
    pub back_face: bool,
}

impl Intersection {
    pub fn world_to_shading(&self, w: Vec3<World>) -> Vec3<Shading> {
        Vec3::new(
            self.bitangeant.dot(w),
            self.tangeant.dot(w),
            self.normal.dot(w),
        )
    }

    pub fn shading_to_world(&self, s: Vec3<Shading>) -> Vec3<World> {
        let x = self.bitangeant.x() * s.x() + self.tangeant.x() * s.y() + self.normal.x() * s.z();
        let y = self.bitangeant.y() * s.x() + self.tangeant.y() * s.y() + self.normal.y() * s.z();
        let z = self.bitangeant.z() * s.x() + self.tangeant.z() * s.y() + self.normal.z() * s.z();
        Vec3::new(x, y, z)
    }
}

#[enum_dispatch]
pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<(Intersection, f32)>;

    fn sample(&self, hit: &Intersection, sampler: &mut Sampler) -> (Point3, f32);

    fn pdf(&self, hit: &Intersection, wi: Vec3) -> f32;
}

#[enum_dispatch(Shape)]
#[derive(Debug, Clone)]
pub enum Geometry {
    Sphere,
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub geometry: Geometry,
    pub light_index: Option<usize>,
    pub material_index: Option<usize>,
}

impl Shape for Primitive {
    fn intersect(&self, ray: &Ray) -> Option<(Intersection, f32)> {
        self.geometry.intersect(ray)
    }

    fn sample(&self, hit: &Intersection, sampler: &mut Sampler) -> (Point3, f32) {
        self.geometry.sample(hit, sampler)
    }

    fn pdf(&self, hit: &Intersection, wi: Vec3) -> f32 {
        self.geometry.pdf(hit, wi)
    }
}

impl Primitive {
    pub fn new_light(geometry: Geometry, light_index: usize) -> Self {
        Self {
            geometry,
            material_index: None,
            light_index: Some(light_index),
        }
    }

    pub fn new_material(geometry: Geometry, material_index: usize) -> Self {
        Self {
            geometry,
            material_index: Some(material_index),
            light_index: None,
        }
    }

    pub fn new_emissive_material(
        geometry: Geometry,
        material_index: usize,
        light_index: usize,
    ) -> Self {
        Self {
            geometry,
            material_index: Some(material_index),
            light_index: Some(light_index),
        }
    }

    pub fn get_light<'a>(&self, lights: &'a [PrimIndex<Spectrum>]) -> Option<&'a Spectrum> {
        self.light_index.map(|i| &lights[i].data)
    }

    pub fn get_material<'a>(&self, materials: &'a [PrimIndex<Bsdf>]) -> Option<&'a Bsdf> {
        self.material_index.map(|i| &materials[i].data)
    }
}
