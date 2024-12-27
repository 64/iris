use crate::{bsdf::LambertianBsdf, math::Point3, shape::Sphere, spectrum::{upsample::UpsampleTable, ConstantSpectrum}};

use super::Scene;

pub fn scene() -> Scene {
    let mut scene = Scene::default();

    let upsample_table = UpsampleTable::load();

    // Light oven
    // scene.add_emissive_material(
    // Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0),
    // LambertianBsdf::new(ConstantSpectrum::new(0.50)),
    // ConstantSpectrum::new(0.25),
    //);

    scene.add_emissive_material(
        Sphere::new(Point3::new(0.0, 2.3, 3.0), 1.0),
        LambertianBsdf::new(ConstantSpectrum::new(0.5)),
        ConstantSpectrum::new(3.0),
    );
    // scene.add_material(
    // Sphere::new(Point3::new(0.0, 1.5, 7.0), 3.0),
    ////LambertianBsdf::new(upsample_table.get_spectrum([0.8, 0.1, 0.1])),
    // LambertianBsdf::new(ConstantSpectrum::new(0.5)),
    //);
    scene.add_material(
        Sphere::new(Point3::new(0.0, -0.2, 3.0), 1.0),
        // FresnelBsdf::new(
        // ConstantSpectrum::new(1.0),
        // ConstantSpectrum::new(1.0),
        // 1.5220,
        // 0.00459,
        //),
        LambertianBsdf::new(ConstantSpectrum::new(0.5)),
    );
    scene.add_material(
        Sphere::new(Point3::new(0.0, -101.5, 2.0), 100.0),
        LambertianBsdf::new(ConstantSpectrum::new(0.8)),
    );

    scene
}
