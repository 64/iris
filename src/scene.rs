use crate::{
    bsdf::{Bsdf, LambertianBsdf, SampleableBsdf},
    math::{OrdFloat, Point3, Ray},
    sampling::{self, mis, Sampler},
    shapes::{Geometry, Intersection, Shape, Sphere},
    spectrum::{
        upsample::UpsampleTable,
        ConstantSpectrum,
        SampleableSpectrum,
        Spectrum,
        SpectrumSample,
        UpsampledHdrSpectrum,
        Wavelength,
    },
};

use std::{collections::HashMap, f32::INFINITY};

const MAX_DEPTH: u32 = 30;
const MIN_DEPTH: u32 = 3;

#[derive(Default)]
pub struct Scene {
    emissives: HashMap<usize, Spectrum>,
    materials: HashMap<usize, Bsdf>,
    geometry: Vec<Geometry>,
    env_map: Vec<UpsampledHdrSpectrum>,
}

impl Scene {
    pub fn dummy() -> Self {
        let mut scene = Self::default();

        let upsample_table = UpsampleTable::load();

        use image::hdr::HdrDecoder;
        use std::{fs::File, io::BufReader};
        let env_map_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/sculpture_exhibition_4k.hdr"
        );
        let env_map_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/cloud_layers_4k.hdr");
        scene.env_map = HdrDecoder::new(BufReader::new(File::open(env_map_path).unwrap()))
            .unwrap()
            .read_image_hdr()
            .unwrap()
            .into_iter()
            .map(|rgb| upsample_table.get_spectrum_hdr(rgb.0))
            .collect();

        let bsdf_red = LambertianBsdf::new(upsample_table.get_spectrum([0.8, 0.1, 0.1]));
        let bsdf_green = LambertianBsdf::new(upsample_table.get_spectrum([0.1, 0.8, 0.1]));
        let bsdf_blue = LambertianBsdf::new(upsample_table.get_spectrum([0.1, 0.1, 0.8]));
        let bsdf_white = LambertianBsdf::new(upsample_table.get_spectrum([0.9, 0.9, 0.9]));

        scene.add_material(Sphere::new(Point3::new(0.0, 0.0, 1.0), 0.5), bsdf_red);
        scene.add_material(Sphere::new(Point3::new(0.9, 0.0, 1.2), 0.3), bsdf_green);
        scene.add_material(Sphere::new(Point3::new(-0.9, 0.0, 1.2), 0.3), bsdf_blue);
        scene.add_material(
            Sphere::new(Point3::new(0.0, -100.5, 1.0), 100.0),
            bsdf_white,
        );
        scene.add_emissive(
            Sphere::new(Point3::new(0.0, 1.3, 1.0), 0.5),
            ConstantSpectrum::new(0.05),
        );

        scene
    }

    fn add_emissive<G: Into<Geometry>, S: Into<Spectrum>>(&mut self, geom: G, emission: S) {
        self.emissives.insert(self.geometry.len(), emission.into());
        self.geometry.push(geom.into());
    }

    fn add_material<G: Into<Geometry>, B: Into<Bsdf>>(&mut self, geom: G, bsdf: B) {
        self.materials.insert(self.geometry.len(), bsdf.into());
        self.geometry.push(geom.into());
    }

    fn background_emission(&self, ray: &Ray, hero_wavelength: Wavelength) -> SpectrumSample {
        let d = ray.d();

        let u = 0.5 + d.z().atan2(d.x()) / (2.0 * std::f32::consts::PI);
        let v = 0.5 - d.y().asin() / std::f32::consts::PI;

        let x = (u.clamp(0.0, 1.0) * 4095.99) as usize;
        let y = (v.clamp(0.0, 1.0) * 2047.99) as usize;

        0.001 * self.env_map[y * 4096 + x].evaluate(hero_wavelength)
        // SpectrumSample::splat((ray.d().y() / 3.0 + 0.5).powi(9))
    }

    fn intersection(&self, ray: &Ray, max_t: f32) -> Option<(usize, Intersection)> {
        // TODO: See if we can get a perf boost by rewriting this
        // It should at least clean up the call stack a bit
        self.geometry
            .iter()
            .enumerate()
            .filter_map(|(i, sphere)| sphere.intersect(ray).map(|h| (i, h)))
            .filter(|(_, (_, ray_t))| *ray_t <= max_t)
            .min_by_key(|(_i, (_hit, ray_t))| OrdFloat::new(*ray_t))
            .map(|(i, (hit, _ray_t))| (i, hit))
    }

    fn nth_light(&self, n: usize) -> Option<(&usize, &Spectrum)> {
        self.emissives.iter().skip(n).next()
    }

    pub fn radiance(
        &self,
        mut ray: Ray,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectrumSample {
        let mut radiance = SpectrumSample::splat(0.0);
        let mut throughput = SpectrumSample::splat(1.0 / hero_wavelength.pdf());

        for bounces in 0..MAX_DEPTH {
            if let Some((geom_index, hit)) = self.intersection(&ray, INFINITY) {
                if bounces == 0
                // || specular_bounce
                {
                    if let Some(emission) = self.emissives.get(&geom_index) {
                        radiance += throughput * emission.evaluate(hero_wavelength);
                    }
                }

                if let Some(bsdf) = self.materials.get(&geom_index) {
                    let shading_wo = hit.world_to_shading(-ray.d());

                    // Calculate direct lighting radiance via NEE
                    let light_index = sampler.gen_array_index(self.emissives.len() + 1);

                    if light_index == self.emissives.len() {
                        // Sample background
                        // Get ray
                        let light_dir_local =
                            sampling::cosine_unit_hemisphere(sampler.gen_0_1(), sampler.gen_0_1());

                        let light_dir = hit.shading_to_world(light_dir_local);
                        let light_ray = Ray::new(hit.point + hit.normal * 0.001, light_dir);
                        let cos_theta = light_dir_local.z().abs();
                        let pdf = sampling::pdf_cosine_unit_hemisphere(cos_theta);

                        if pdf != 0.0 && self.intersection(&light_ray, INFINITY).is_none() {
                            let bsdf = bsdf.evaluate(light_dir_local, shading_wo, hero_wavelength);
                            let mis_weight = 0.25;
                            let le = self.background_emission(&light_ray, hero_wavelength);

                            radiance += le
                                * throughput
                                * bsdf
                                * (cos_theta * mis_weight
                                    / pdf
                                    / (self.emissives.len() + 1) as f32);
                        }
                    } else if let Some((&light_geom_index, light_spectrum)) =
                        self.nth_light(light_index)
                    {
                        // Sample the emission
                        assert_ne!(light_geom_index, geom_index);

                        let (light_point, light_pdf) =
                            self.geometry[light_geom_index].sample(hit.point, sampler);
                        let hit_point = hit.point + hit.normal * 0.001;
                        let ray_to_light = Ray::new(hit_point, light_point - hit_point);
                        let max_t = (light_point - hit_point).len();

                        if light_pdf != 0.0 && self.intersection(&ray_to_light, max_t).is_none() {
                            let shading_wi = hit.world_to_shading(ray_to_light.d());
                            let bsdf = bsdf.evaluate(shading_wi, shading_wo, hero_wavelength);
                            let mis_weight = 0.25;
                            let cos_theta = shading_wi.z().abs();
                            let le = light_spectrum.evaluate(hero_wavelength);

                            radiance += le
                                * throughput
                                * bsdf
                                * (cos_theta * mis_weight
                                    / light_pdf
                                    / (self.emissives.len() + 1) as f32);
                        }
                    }

                    // Calculate indirect lighting
                    let (bsdf_sampled_wi, path_pdfs) =
                        bsdf.sample(shading_wo, hero_wavelength, sampler);

                    if path_pdfs[0] == 0.0 {
                        break;
                    }

                    let bsdf = bsdf.evaluate(bsdf_sampled_wi, shading_wo, hero_wavelength);
                    let cos_theta = bsdf_sampled_wi.z().abs();
                    let mis_weight = mis::hwss_weight(hero_wavelength, path_pdfs);

                    throughput *= bsdf * (cos_theta * mis_weight / path_pdfs[0]);

                    // Spawn the next ray
                    ray = Ray::new(
                        hit.point + hit.normal * 0.01,
                        hit.shading_to_world(bsdf_sampled_wi),
                    );

                    // Russian roulette
                    if bounces > MIN_DEPTH {
                        let p = throughput.x();
                        if sampler.gen_0_1() > p {
                            break;
                        }

                        throughput /= SpectrumSample::splat(p);
                    }
                } else {
                    // Hit some purely emissive object
                    break;
                }
            } else if bounces == 0
            // || specular
            {
                radiance += throughput * self.background_emission(&ray, hero_wavelength);
                break;
            } else {
                // Hit background but we don't need to accumulate it
                break;
            }
        }

        radiance
    }
}
