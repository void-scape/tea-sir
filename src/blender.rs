use crate::Memory;
use crate::camera::Camera;
use crate::io;
use crate::math::*;
use crate::model;
use crate::model::Model;
use crate::model::Obb;
use rast::tint::*;

pub struct BlenderMemory {
    models: Vec<DisplayModel>,
}

impl Default for BlenderMemory {
    fn default() -> Self {
        // let materials = [
        //     ("assets/ibuki/bloomers.bin", "bloomers"),
        //     ("assets/ibuki/coat.bin", "coat"),
        //     ("assets/ibuki/face.bin", "face"),
        //     ("assets/ibuki/halo.bin", "halo"),
        //     ("assets/ibuki/package.bin", "package"),
        //     ("assets/ibuki/body.bin", "body"),
        //     ("assets/ibuki/eye.bin", "eye"),
        //     ("assets/ibuki/hair.bin", "hair"),
        //     ("assets/ibuki/shirt.bin", "shirt"),
        // ]
        // .into_iter()
        // .map(|(path, name)| io::debug_image_file(path).map(|img| (name.to_string(), img)))
        // .collect::<Option<Vec<_>>>()
        // .expect("failed to load ibuki materials");
        //
        // let ibuki = io::debug_obj_file("assets/ibuki/ibuki.obj", materials)
        //     .expect("could not load `ibuki.obj`");

        let dragon = include_str!("../assets/dragon.obj");
        let dragon_model = io::debug_obj_str(dragon, vec![]).unwrap();

        Self {
            models: vec![DisplayModel {
                obb: model::compute_obb(&dragon_model),
                model: dragon_model,
                translation: Vec3::ZERO,
                scale: 1.0,
                zrotation: 0.0,
            }],
        }
    }
}

struct DisplayModel {
    model: Model,
    obb: Obb,
    translation: Vec3,
    scale: f32,
    zrotation: f32,
}

pub fn render(
    memory: &mut BlenderMemory,
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    camera: &Camera,
    width: usize,
    height: usize,
    delta: f32,
) {
    for display in memory.models.iter_mut() {
        display.zrotation = (display.zrotation + delta) % core::f32::consts::TAU;

        let obb = display.obb;
        let translation = Vec3::ZERO;
        let scale = Vec3::splat(display.scale);
        let rotation = Quat::from_rotation_y(display.zrotation);
        let pyr = Vec3::ZERO;
        if model::obb_visible(width, height, camera, obb, translation, pyr) {
            let model_matrix = compute_model_matrix(translation, rotation, scale);
            model::draw_model_matrix(
                frame_buffer,
                zbuffer,
                width,
                height,
                camera,
                &display.model,
                glam::Vec3::ZERO,
                glam::Vec3::ONE,
                glam::Quat::from_rotation_y(display.zrotation),
            );

            // model::debug_draw_obb(
            //     frame_buffer,
            //     zbuffer,
            //     width,
            //     height,
            //     camera,
            //     obb,
            //     translation,
            //     pyr,
            //     Srgb::from_rgb(0, 255, 0),
            // );
        }
    }
}
