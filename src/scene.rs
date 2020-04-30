#![allow(unused)]
#![allow(dead_code)]
use crate::{
    bsdf::{Bsdf, FresnelBsdf, LambertianBsdf, MicrofacetBsdf, SampleableBsdf, SpecularBsdf},
    math::{PdfSet, Point3, Ray, Shading, Vec3},
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

        let upsample_table = UpsampleTable::load();

        //scene.add_emissive_material(
            //Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0),
            //LambertianBsdf::new(ConstantSpectrum::new(0.50)),
            //ConstantSpectrum::new(0.50),
        //);
        scene.add_emissive_material(
            Sphere::new(Point3::new(0.0, 3.2, 3.0), 1.0),
            LambertianBsdf::new(ConstantSpectrum::new(0.50)),
            ConstantSpectrum::new(3.0),
        );
        //scene.add_material(
            //Sphere::new(Point3::new(0.0, 1.5, 7.0), 3.0),
            //LambertianBsdf::new(upsample_table.get_spectrum([0.8, 0.1, 0.1])),
        //);
        scene.add_material(
            Sphere::new(Point3::new(0.0, 1.0, 3.0), 1.0),
            FresnelBsdf::new(ConstantSpectrum::new(1.0), ConstantSpectrum::new(1.0), 1.52),
        );
        scene.add_material(
            Sphere::new(Point3::new(0.0, -101.5, 2.0), 100.0),
            LambertianBsdf::new(ConstantSpectrum::new(0.80)),
        );

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

    fn background_emission(&self, ray: &Ray, _wavelength: Wavelength) -> SpectralSample {
        SpectralSample::splat(0.0)
    }

    fn intersection(&self, ray: &Ray) -> Option<(&Primitive, Intersection)> {
        let mut closest_t = INFINITY;
        let mut closest_prim_hit = None;

        for prim in &self.primitives {
            match prim.intersect(ray) {
                Some((hit, t)) if t < closest_t && t > 0.0 => {
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
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let mut throughput = SpectralSample::splat(1.0);
        let mut path_pdfs = PdfSet::splat(1.0);
        let mut radiance = SpectralSample::splat(0.0);
        let mut specular_bounce = false;

        for bounces in 0..MAX_DEPTH {
            if let Some((primitive, hit)) = self.intersection(&ray) {
                if bounces == 0 || specular_bounce {
                    if let Some(light) = primitive.get_light(&self.lights) {
                        radiance += mis::balance_heuristic_1(path_pdfs) * throughput * light.evaluate(wavelength);
                    }
                }

                if let Some(bsdf) = primitive.get_material(&self.materials) {
                    let shading_wo = hit.world_to_shading(-ray.d());

                    radiance += throughput * self.direct_lighting(bsdf, shading_wo, &hit, &ray, path_pdfs, wavelength, sampler);

                    // Indirect lighting
                    let (bsdf_sampled_wi, bsdf_values, bsdf_pdfs) =
                        bsdf.sample(shading_wo, wavelength, sampler);
                    if bsdf_pdfs.hero() == 0.0 {
                        break;
                    }

                    let cos_theta = bsdf_sampled_wi.cos_theta().abs();
                    throughput *= bsdf_values * cos_theta / bsdf_pdfs.hero();

                    ray = Ray::spawn(hit.point, hit.shading_to_world(bsdf_sampled_wi), hit.normal);
                    specular_bounce = bsdf.is_specular();
                    path_pdfs *= bsdf_pdfs;

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
                radiance += mis::balance_heuristic_1(path_pdfs) * throughput * self.background_emission(&ray, wavelength);
                break;
            }
        }

        radiance
    }

    pub fn direct_lighting(
        &self,
        bsdf: &Bsdf,
        shading_wo: Vec3<Shading>,
        hit: &Intersection,
        ray: &Ray,
        path_pdfs: PdfSet,
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        if bsdf.is_specular() {
            return SpectralSample::splat(0.0);
        }

        let light_idx = sampler.gen_array_index(self.lights.len());
        let light_weight = self.lights.len() as f32;

        self.sample_light(
            bsdf,
            shading_wo,
            hit,
            light_idx,
            light_weight,
            ray,
            path_pdfs,
            wavelength,
            sampler,
        )
    }

    pub fn sample_light(
        &self,
        bsdf: &Bsdf,
        shading_wo: Vec3<Shading>,
        hit: &Intersection,
        light_idx: usize,
        light_weight: f32,
        ray: &Ray,
        path_pdfs: PdfSet,
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let mut radiance = SpectralSample::splat(0.0);

        let light = &self.lights[light_idx];
        let light_prim = &self.primitives[light.prim_index];
        let (light_pos, light_pdf) = light_prim.sample(&hit, sampler);
        let light_emission = light.data.evaluate(wavelength);

        let ray_to_light = Ray::spawn_to(hit.point, light_pos, hit.normal);

        if light_pdf > 0.0
            && light_emission.sum() > 0.0 // TODO: !light_emission.is_zero()
            && self
                .intersection(&ray_to_light)
                .map(|(prim, light_hit)| std::ptr::eq(prim, light_prim))
                .unwrap_or(false)
        {
            let shading_wi = hit.world_to_shading(ray_to_light.d());
            let bsdf_values = bsdf.evaluate(shading_wi, shading_wo, wavelength);
            let bsdf_pdfs = bsdf.pdf(shading_wi, shading_wo, wavelength);
            let cos_theta = shading_wi.cos_theta().abs();

            let mis_weight = mis::balance_heuristic_2(path_pdfs * PdfSet::splat(light_pdf), path_pdfs * bsdf_pdfs);

            radiance += light_emission * bsdf_values * mis_weight * cos_theta / light_pdf;

            let (bsdf_sampled_wi, bsdf_values, bsdf_pdfs) = bsdf.sample(shading_wo, wavelength, sampler);
            let world_wi = hit.shading_to_world(bsdf_sampled_wi);
            let ray_to_light = Ray::spawn(hit.point, world_wi, hit.normal);
            let light_emission = light.data.evaluate(wavelength);

            if bsdf_pdfs.hero() > 0.0
                && light_emission.sum() > 0.0 // TODO: !light_emission.is_zero()
                && self
                    .intersection(&ray_to_light)
                    .map(|(prim, light_hit)| std::ptr::eq(prim, light_prim))
                    .unwrap_or(false)
            {
                let light_pdf = light_prim.pdf(hit, world_wi);
                let cos_theta = bsdf_sampled_wi.cos_theta().abs();
                let mis_weight = mis::balance_heuristic_2(path_pdfs * bsdf_pdfs, path_pdfs * PdfSet::splat(light_pdf));

                radiance += light_emission * bsdf_values * mis_weight * cos_theta / bsdf_pdfs.hero();
            }

            radiance * light_weight
        } else {
            SpectralSample::splat(0.0)
        }
    }
}
