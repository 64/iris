#![allow(unused)]
#![allow(dead_code)]
use crate::{
    bsdf::{Bsdf, LambertianBsdf, MicrofacetBsdf, SampleableBsdf, SpecularBsdf},
    math::{Point3, Ray, Shading, Vec3},
    sampling::{self, mis, Sampler},
    shapes::{Geometry, Intersection, Primitive, Shape, Sphere},
    spectrum::{
        upsample::UpsampleTable,
        ConstantSpectrum,
        SampleableSpectrum,
        SpectralSample,
        Spectrum,
        UpsampledHdrSpectrum,
        Wavelength,
    },
    types::PrimIndex,
};

use std::f32::INFINITY;

const MAX_DEPTH: u32 = 30;
const MIN_DEPTH: u32 = 3;

#[derive(Default)]
pub struct Scene {
    lights: Vec<PrimIndex<Spectrum>>,
    materials: Vec<PrimIndex<Bsdf>>,
    primitives: Vec<Primitive>,
    _env_map: Vec<UpsampledHdrSpectrum>,
}

impl Scene {
    pub fn dummy() -> Self {
        let mut scene = Self::default();

        scene.add_emissive_material(
            Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0),
            LambertianBsdf::new(ConstantSpectrum::new(0.25)),
            ConstantSpectrum::new(0.25),
        );
        //scene.add_emissive_material(
            //Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0),
            //SpecularBsdf::new(ConstantSpectrum::new(0.25), 1.8),
            //ConstantSpectrum::new(0.25),
        //);

        scene
    }

    fn add_light<G: Into<Geometry>, S: Into<Spectrum>>(&mut self, geom: G, light: S) {
        self.lights.push(PrimIndex {
            data: light.into(),
            prim_index: self.primitives.len(),
        });
        self.primitives
            .push(Primitive::new_light(geom.into(), self.lights.len() - 1));
    }

    fn add_material<G: Into<Geometry>, B: Into<Bsdf>>(&mut self, geom: G, material: B) {
        self.materials.push(PrimIndex {
            data: material.into(),
            prim_index: self.primitives.len(),
        });
        self.primitives.push(Primitive::new_material(
            geom.into(),
            self.materials.len() - 1,
        ));
    }

    fn add_emissive_material<G: Into<Geometry>, B: Into<Bsdf>, S: Into<Spectrum>>(
        &mut self,
        geom: G,
        material: B,
        light: S,
    ) {
        self.materials.push(PrimIndex {
            data: material.into(),
            prim_index: self.primitives.len(),
        });
        self.lights.push(PrimIndex {
            data: light.into(),
            prim_index: self.primitives.len(),
        });
        self.primitives.push(Primitive::new_emissive_material(
            geom.into(),
            self.materials.len() - 1,
            self.lights.len() - 1,
        ));
    }

    fn background_emission(&self, ray: &Ray, _hero_wavelength: Wavelength) -> SpectralSample {
        SpectralSample::splat(0.0)
    }

    fn intersection(&self, ray: &Ray, max_t: f32) -> Option<(&Primitive, Intersection)> {
        let mut closest_t = INFINITY;
        let mut closest_prim_hit = None;

        for prim in &self.primitives {
            match prim.intersect(ray) {
                Some((hit, t)) if t < closest_t && t > 0.0 && t < max_t => {
                    closest_t = t;
                    closest_prim_hit = Some((prim, hit));
                }
                _ => continue,
            }
        }

        closest_prim_hit
    }

    pub fn radiance(
        &self,
        mut ray: Ray,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let mut radiance = SpectralSample::splat(0.0);
        // Since we use 4 wavelengths
        // TODO: Should we start at 1.0 and compensate in another way?
        let mut throughput = SpectralSample::splat(0.25);

        for bounces in 0..MAX_DEPTH {
            if let Some((primitive, hit)) = self.intersection(&ray, INFINITY) {
                if let Some(light) = primitive.get_light(&self.lights) {
                    radiance += throughput * light.evaluate(hero_wavelength);
                }

                if let Some(bsdf) = primitive.get_material(&self.materials) {
                    let shading_wo = hit.world_to_shading(-ray.d());

                    // Indirect lighting
                    let (bsdf_sampled_wi, bsdf_values, bsdf_pdfs) =
                        bsdf.sample(shading_wo, hero_wavelength, sampler);
                    if bsdf_pdfs.hero() == 0.0 {
                        break;
                    }

                    let cos_theta = bsdf_sampled_wi.cos_theta().abs();
                    let mis_weight = mis::balance_heuristic_1(bsdf_pdfs);

                    throughput *= bsdf_values * mis_weight * cos_theta / bsdf_pdfs.hero();

                    ray = Ray::spawn(hit.point, hit.shading_to_world(bsdf_sampled_wi), hit.normal);

                    // Russian roulette
                    if bounces >= MIN_DEPTH {
                        let p = throughput.sum().min(0.95);
                        if sampler.gen_0_1() > p {
                            break;
                        }

                        throughput /= SpectralSample::splat(p);
                    }
                }
            } else {
                radiance += throughput * self.background_emission(&ray, hero_wavelength);
                break;
            }
        }

        radiance
    }
}
