use crate::{
    bsdf::{Bsdf, LambertianBsdf, MicrofacetBsdf, SampleableBsdf},
    math::{OrdFloat, Point3, Ray, Shading, Vec3},
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

        // use image::hdr::HdrDecoder;
        // use std::{fs::File, io::BufReader};
        // let env_map_path = concat!(
        // env!("CARGO_MANIFEST_DIR"),
        //"/data/small_cathedral_4k.hdr"
        //);
        // let env_map_path = concat!(env!("CARGO_MANIFEST_DIR"),
        // "/data/cloud_layers_4k.hdr"); scene.env_map =
        // HdrDecoder::new(BufReader::new(File::open(env_map_path).unwrap()))
        //.unwrap()
        //.read_image_hdr()
        //.unwrap()
        //.into_iter()
        //.map(|rgb| upsample_table.get_spectrum_hdr(rgb.0))
        //.collect();

        let bsdf_red =
            MicrofacetBsdf::new(upsample_table.get_spectrum([0.8, 0.1, 0.1]), 0.80, 0.20);
        // let bsdf_red = LambertianBsdf::new(upsample_table.get_spectrum([0.8, 0.1,
        // 0.1])); let bsdf_green =
        // MicrofacetBsdf::new(upsample_table.get_spectrum([0.1, 0.8, 0.1]), 0.2, 0.2);
        let bsdf_green = LambertianBsdf::new(upsample_table.get_spectrum([0.1, 0.8, 0.1]));
        // let bsdf_blue = MicrofacetBsdf::new(upsample_table.get_spectrum([0.1, 0.1,
        // 0.8]), 0.2, 0.2);
        let bsdf_blue = LambertianBsdf::new(upsample_table.get_spectrum([0.1, 0.1, 0.8]));
        // let bsdf_white = MicrofacetBsdf::new(upsample_table.get_spectrum([0.8, 0.8,
        // 0.8]), 0.2, 0.2);
        let bsdf_white = LambertianBsdf::new(upsample_table.get_spectrum([0.8, 0.8, 0.8]));

        scene.add_material(Sphere::new(Point3::new(0.0, 0.0, 1.0), 0.5), bsdf_red);
        scene.add_material(Sphere::new(Point3::new(0.9, 0.0, 1.2), 0.3), bsdf_green);
        scene.add_material(Sphere::new(Point3::new(-0.9, 0.0, 1.2), 0.3), bsdf_blue);
        scene.add_material(
            Sphere::new(Point3::new(0.0, -100.5, 1.0), 100.0),
            bsdf_white,
        );
        // scene.add_light(
        // Sphere::new(Point3::new(0.0, 1.2, 0.8), 0.5),
        // ConstantSpectrum::new(2000.0),
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

    fn background_emission(&self, ray: &Ray, _hero_wavelength: Wavelength) -> SpectralSample {
        // let d = ray.d();

        // let u = 0.5 + d.z().atan2(d.x()) / (2.0 * std::f32::consts::PI);
        // let v = 0.5 - d.y().asin() / std::f32::consts::PI;

        // let x = ((u + 0.6).fract() * 4095.99) as usize;
        // let y = ((v + 0.15).fract() * 2047.99) as usize;

        // 0.02 * self.env_map[y * 4096 + x].evaluate(hero_wavelength)
        // SpectralSample::splat((ray.d().y() / 2.0 + 0.5).powi(5) * 200.0)
        SpectralSample::splat(200.0)
    }

    fn intersection(&self, ray: &Ray, max_t: f32) -> Option<(&Primitive, Intersection)> {
        // TODO: See if we can get a perf boost by rewriting this as a loop
        // It should at least clean up the call stack a bit
        self.primitives
            .iter()
            .filter_map(|prim| prim.intersect(ray).map(|h| (prim, h)))
            .filter(|(_, (_, ray_t))| *ray_t <= max_t)
            .min_by_key(|(_, (_, ray_t))| OrdFloat::new(*ray_t))
            .map(|(prim, (hit, _))| (prim, hit))
    }

    pub fn radiance(
        &self,
        mut ray: Ray,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let mut radiance = SpectralSample::splat(0.0);
        let mut throughput = SpectralSample::splat(1.0);

        for bounces in 0..MAX_DEPTH {
            if let Some((primitive, hit)) = self.intersection(&ray, INFINITY) {
                if bounces == 0 {
                    if let Some(light) = primitive.get_light(&self.lights) {
                        radiance += throughput * light.evaluate(hero_wavelength);
                    }
                }

                if let Some(bsdf) = primitive.get_material(&self.materials) {
                    let shading_wo = hit.world_to_shading(-ray.d());

                    // Direct lighting (NEE)
                    let direct = throughput
                        * self.direct_lighting(shading_wo, bsdf, &hit, hero_wavelength, sampler);
                    if direct.sum() > 1000.0 {
                        // print!("dl: {:?} {:?} {:?}\n", direct, radiance,
                        // throughput);
                    }
                    radiance += direct;

                    // Indirect lighting
                    let (bsdf_sampled_wi, path_pdfs) =
                        bsdf.sample(shading_wo, hero_wavelength, sampler);

                    if path_pdfs[0] == 0.0 {
                        break;
                    }

                    let bsdf = bsdf.evaluate(bsdf_sampled_wi, shading_wo, hero_wavelength);
                    let cos_theta = bsdf_sampled_wi.cos_theta().abs();
                    let mis_weight = mis::hwss_weight(hero_wavelength, path_pdfs);

                    throughput *= bsdf * (cos_theta * mis_weight / path_pdfs[0]);

                    ray = Ray::spawn(hit.point, hit.shading_to_world(bsdf_sampled_wi), hit.normal);

                    // Russian roulette
                    if bounces >= MIN_DEPTH {
                        // TODO: Can we rewrite without 1 - t?
                        let p = 1.0 - throughput.sum() / 4.0;
                        if p > 0.0 && sampler.gen_0_1() < p.max(0.05) {
                            break;
                        }

                        throughput /= SpectralSample::splat(1.0 - p.max(0.05));
                    }
                } else {
                    // Hit some purely emissive object
                    unreachable!();
                    // break;
                }
            } else if bounces == 0 {
                radiance += throughput * self.background_emission(&ray, hero_wavelength);
                if radiance.sum() > 1000.0 {
                    // print!("bg: {:?} {:?}\n", radiance, throughput);
                }
                break;
            } else {
                // Hit background but we don't need to accumulate it
                if radiance.sum() > 1000.0 {
                    // print!("bn: {:?} {:?} {}\n", radiance, throughput,
                    // bounces);
                }
                break;
            }
        }

        if radiance.sum() > 1000.0 {
            // print!("ex: {:?} {:?}\n", radiance, throughput);
        }

        radiance
    }

    fn direct_lighting(
        &self,
        shading_wo: Vec3<Shading>,
        bsdf: &Bsdf,
        hit: &Intersection,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let light_index = sampler.gen_array_index(self.lights.len() + 1);
        let light_weight = (self.lights.len() + 1) as f32;
        let mut radiance = SpectralSample::splat(0.0);

        if light_index == self.lights.len() {
            let light_dir_local =
                sampling::cosine_unit_hemisphere(sampler.gen_0_1(), sampler.gen_0_1());

            let light_dir = hit.shading_to_world(light_dir_local);
            let light_ray = Ray::spawn(hit.point, light_dir, hit.normal);
            let cos_theta = light_dir_local.cos_theta().abs();
            let light_pdf = sampling::pdf_cosine_unit_hemisphere(cos_theta);

            if light_pdf != 0.0 && self.intersection(&light_ray, INFINITY).is_none() {
                let bsdf_pdf = bsdf.pdf(light_dir_local, shading_wo, hero_wavelength);
                let bsdf_value = bsdf.evaluate(light_dir_local, shading_wo, hero_wavelength);
                let mis_weight = mis::balance_heuristic([light_pdf; 4], bsdf_pdf);
                let le = self.background_emission(&light_ray, hero_wavelength);

                radiance += le * bsdf_value * cos_theta * mis_weight / (light_pdf * light_weight);
            }
        } else {
            // Sample the emission
            let light = &self.lights[light_index];
            // assert_ne!(light.index, prim_index);

            let (light_point, light_pdf) =
                self.primitives[light.prim_index].sample(hit.point, sampler);
            // TODO: There is a slight error in this ray spawn
            // let ray_to_light = Ray::spawn(hit.point, light_point - hit.point,
            // hit.normal);
            let hit_point = hit.point + hit.normal * 0.001;
            let ray_to_light = Ray::new(hit_point, light_point - hit_point);
            let max_t = (light_point - ray_to_light.o()).len();

            // TODO: Check if it intersected the given light
            if light_pdf != 0.0 && self.intersection(&ray_to_light, max_t).is_none() {
                let shading_wi = hit.world_to_shading(ray_to_light.d());
                let bsdf_pdf = bsdf.pdf(shading_wi, shading_wo, hero_wavelength);
                let bsdf_value = bsdf.evaluate(shading_wi, shading_wo, hero_wavelength);
                let cos_theta = shading_wi.cos_theta().abs();
                let le = light.data.evaluate(hero_wavelength);
                let mis_weight = mis::balance_heuristic([light_pdf; 4], bsdf_pdf);

                radiance += le * bsdf_value * cos_theta * mis_weight / (light_pdf * light_weight);
            }
        }

        let (bsdf_sampled_wi, bsdf_pdfs) = bsdf.sample(shading_wo, hero_wavelength, sampler);
        let bsdf_values = bsdf.evaluate(bsdf_sampled_wi, shading_wo, hero_wavelength);

        if !radiance.is_zero() && bsdf_pdfs[0] != 0.0 {
            let cos_theta = bsdf_sampled_wi.cos_theta().abs();
            let light_pdf = sampling::pdf_cosine_unit_hemisphere(cos_theta);

            let mis_weight = mis::balance_heuristic(bsdf_pdfs, [light_pdf; 4]);
            let world_dir = hit.shading_to_world(bsdf_sampled_wi);
            let ray = Ray::spawn(hit.point, world_dir, hit.normal);
            let le = self.background_emission(&ray, hero_wavelength);

            if self.intersection(&ray, INFINITY).is_none() {
                radiance += le * bsdf_values * cos_theta * mis_weight / bsdf_pdfs[0];
            }
        }

        radiance
    }
}
