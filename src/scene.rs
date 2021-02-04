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

const MAX_DEPTH: u32 = 10;
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

        // Light oven
         //scene.add_emissive_material(
         //Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0),
         //LambertianBsdf::new(ConstantSpectrum::new(0.50)),
         //ConstantSpectrum::new(0.50),
        //);

        scene.add_emissive_material(
            // Sphere::new(Point3::new(0.0, 3.2, 3.0), 1.0),
            Sphere::new(Point3::new(0.0, 2.3, 3.0), 1.0),
            LambertianBsdf::new(ConstantSpectrum::new(0.5)),
            ConstantSpectrum::new(3.0),
        );
        //scene.add_material(
            //Sphere::new(Point3::new(0.0, 1.5, 7.0), 3.0),
            ////LambertianBsdf::new(upsample_table.get_spectrum([0.8, 0.1, 0.1])),
            //LambertianBsdf::new(ConstantSpectrum::new(0.5)),
        //);
        scene.add_material(
            Sphere::new(Point3::new(0.0, -0.2, 3.0), 1.0),
            //FresnelBsdf::new(
                //ConstantSpectrum::new(1.0),
                //ConstantSpectrum::new(1.0),
                //1.5220,
                //0.00459,
            //),
            LambertianBsdf::new(ConstantSpectrum::new(0.5)),
        );
        scene.add_material(
            Sphere::new(Point3::new(0.0, -101.5, 2.0), 100.0),
            LambertianBsdf::new(ConstantSpectrum::new(0.8)),
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

        // Note: for some reason, the equivalent code with iterators is *much* slower
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

        let mut int = self.intersection(&ray);

        for bounces in 0..MAX_DEPTH {
            let (prim, hit) = if let Some(ph) = int {
                ph
            } else {
                let mis_weight = mis::balance_heuristic_1(path_pdfs);
                radiance += throughput * mis_weight * self.background_emission(&ray, wavelength);
                break;
            };

            if bounces == 0 || specular_bounce {
                if let Some(light) = prim.get_light(&self.lights) {
                    let mis_weight = mis::balance_heuristic_1(path_pdfs);
                    radiance += throughput * mis_weight * light.evaluate(wavelength);
                }
            }

            let bsdf = match prim.get_material(&self.materials) {
                Some(bsdf) => bsdf,
                None => break,
            };

            let shading_wo = hit.world_to_shading(-ray.d());

            // Next event estimation
            if !bsdf.is_specular() {
                let light_idx = sampler.gen_array_index(self.lights.len());
                let light = &self.lights[light_idx];
                let light_prim = &self.primitives[light.prim_index];

                let (light_pos, light_pdf) = light_prim.sample(&hit, sampler);
                let light_emission = light.data.evaluate(wavelength);

                let light_pick_weight = self.lights.len() as f32;
                let ray_to_light = Ray::spawn_to(hit.point, light_pos, hit.normal);

                if light_pdf > 0.0
                    && !light_emission.is_zero()
                    // TODO: Use t_max instead of checking whether it hit the same light
                    && self
                        .intersection(&ray_to_light)
                        .map(|(prim, light_hit)| std::ptr::eq(prim, light_prim))
                        .unwrap_or(false)
                {
                    let shading_wi = hit.world_to_shading(ray_to_light.d());

                    let bsdf_values = bsdf.evaluate(shading_wi, shading_wo, wavelength);
                    let bsdf_pdfs = bsdf.pdf(shading_wi, shading_wo, wavelength);
                    let cos_theta = shading_wi.cos_theta().abs();
                    //let mis_weight = mis::balance_heuristic_2(
                        //path_pdfs * PdfSet::splat(light_pdf),
                        //path_pdfs * bsdf_pdfs,
                    //);
                    let mis_weight = mis::balance_heuristic_1(path_pdfs * PdfSet::splat(light_pdf));

                    radiance += throughput
                        * light_emission
                        * bsdf_values
                        * mis_weight
                        * cos_theta
                        * light_pick_weight
                         / light_pdf;
                }
            }

            // Sample BSDF
            let (bsdf_sampled_wi, bsdf_values, bsdf_pdfs) =
                bsdf.sample(shading_wo, wavelength, sampler);
            let cos_theta = bsdf_sampled_wi.cos_theta().abs();
            if bsdf_pdfs.hero() == 0.0 || cos_theta == 0.0 {
                break;
            }

            let world_wi = hit.shading_to_world(bsdf_sampled_wi);
            ray = Ray::spawn(hit.point, world_wi, hit.normal);

            throughput *= bsdf_values * cos_theta / bsdf_pdfs.hero();

            // Russian roulette
            if bounces >= MIN_DEPTH {
                let p = throughput.sum().min(0.95);
                if sampler.gen_0_1() > p {
                    break;
                }

                throughput /= SpectralSample::splat(p);
            }

            int = self.intersection(&ray);
            //if !bsdf.is_specular() {
            if false {
                if let Some((next_prim, next_hit)) = &int {
                    if let Some(light) = next_prim.get_light(&self.lights) {
                        let light_emission = light.evaluate(wavelength);
                        let light_pdf =
                            prim.pdf(&hit, next_hit.point - hit.point) / self.lights.len() as f32;
                        let mis_weight = mis::balance_heuristic_2(
                            path_pdfs * bsdf_pdfs,
                            path_pdfs * PdfSet::splat(light_pdf),
                        );

                        radiance += throughput * mis_weight * light_emission;
                    }
                } else {
                    // Sample background
                    // unreachable!();
                    break;
                }
            }

            path_pdfs *= bsdf_pdfs;
            specular_bounce = bsdf.is_specular();
        }

        radiance
    }
}
