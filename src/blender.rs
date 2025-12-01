use crate::Memory;
use crate::camera::Camera;
use crate::io;
use crate::math::*;
use crate::model;
use rast::tint::*;

pub struct BlenderMemory {
    teapot: model::Model,
    ibuki_obb: model::Obb,
    ibuki: model::Model,
    angle: f32,
}

impl Default for BlenderMemory {
    fn default() -> Self {
        let materials = [
            ("assets/ibuki/bloomers.bin", "bloomers"),
            ("assets/ibuki/coat.bin", "coat"),
            ("assets/ibuki/face.bin", "face"),
            ("assets/ibuki/halo.bin", "halo"),
            ("assets/ibuki/package.bin", "package"),
            ("assets/ibuki/body.bin", "body"),
            ("assets/ibuki/eye.bin", "eye"),
            ("assets/ibuki/hair.bin", "hair"),
            ("assets/ibuki/shirt.bin", "shirt"),
        ]
        .into_iter()
        .map(|(path, name)| io::debug_image_file(path).map(|img| (name.to_string(), img)))
        .collect::<Option<Vec<_>>>()
        .expect("failed to load ibuki materials");

        let ibuki = io::debug_obj_file("assets/ibuki/ibuki.obj", materials)
            .expect("could not load `ibuki.obj`");

        Self {
            teapot: io::debug_obj_file("assets/teapot.obj", Vec::new())
                .expect("could not load `teapot.obj`"),
            ibuki_obb: model::compute_obb(&ibuki),
            ibuki,
            angle: 0.0,
        }
    }
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
    memory.angle = (memory.angle + delta) % core::f32::consts::TAU;

    let obb = memory.ibuki_obb;
    let translation = Vec3::ZERO;
    let scale = Vec3::splat(1.0);
    let rotation = Quat::default();
    let pyr = Vec3::ZERO;
    // let pyr = Vec3::new(0.0, memory.angle, 0.0);
    if model::obb_visible(width, height, camera, obb, translation, pyr) {
        let (dur, _) = glazer::debug_time_millis(|| {
            // let (dur, model_matrix) =
            //     glazer::debug_time_nanos(|| compute_model_matrix(translation, rotation, scale));
            // glazer::log!("  compute matrix: {dur}ns");
            let model_matrix = compute_model_matrix(translation, rotation, scale);
            model::draw_model_matrix(
                frame_buffer,
                zbuffer,
                width,
                height,
                camera,
                &memory.ibuki,
                model_matrix,
            );
        });
        // glazer::log!("matrix: {dur}ms");

        // let (dur, _) = glazer::debug_time_millis(|| {
        //     model::draw_model(
        //         frame_buffer,
        //         zbuffer,
        //         width,
        //         height,
        //         camera,
        //         &memory.ibuki,
        //         translation,
        //         pyr,
        //     );
        // });
        // glazer::log!("manual: {dur}ms");

        // model::debug_draw_obb(
        //     frame_buffer,
        //     zbuffer,
        //     width,
        //     height,
        //     camera,
        //     obb,
        //     translation,
        //     pyr,
        //     Srgb::rgb(0, 255, 0),
        // );
    }

    let obb = model::compute_obb(&memory.teapot);
    for x in -1..=1 {
        let translation = Vec3::x(x as f32 * 10.0 + 50.0);
        let pyr = Vec3::new(memory.angle, memory.angle, memory.angle);
        if model::obb_visible(width, height, camera, obb, translation, pyr) {
            model::draw_model(
                frame_buffer,
                zbuffer,
                width,
                height,
                camera,
                &memory.teapot,
                translation,
                pyr,
            );

            model::debug_draw_obb(
                frame_buffer,
                zbuffer,
                width,
                height,
                camera,
                obb,
                translation,
                pyr,
                Srgb::from_rgb(0, 255, 0),
            );
        }
    }
}
