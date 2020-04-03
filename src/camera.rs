use crate::math::{Matrix, Point3, Vec3};

pub struct Camera {
    pub position: Point3,
    pub world_to_clip: Matrix,
    pub clip_to_world: Matrix,
}

impl Camera {
    pub fn new(pos: Point3, aspect_ratio: f32) -> Self {
        let camera_to_clip = Matrix::projection(aspect_ratio, 0.1, 100.0, 90.0);
        let world_to_camera = Matrix::translation(-Vec3::new(pos.x, pos.y, pos.z));
        let world_to_clip = &camera_to_clip * &world_to_camera;

        Self {
            position: pos,
            clip_to_world: world_to_clip.inverse(),
            world_to_clip,
        }
    }
}
