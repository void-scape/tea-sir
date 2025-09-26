use crate::{camera::Camera, math::*};
use alloc::vec::Vec;
use rast::tint::*;

#[derive(Default)]
pub struct Model {
    pub faces: Vec<usize>,
    pub verts: Vec<Vec3>,
}

#[derive(Debug, Clone, Copy)]
pub struct Obb {
    pub min: Vec3,
    pub max: Vec3,
}

pub fn compute_obb(model: &Model) -> Obb {
    assert_model(model);

    let (minx, maxx) = min_max(model, |v| v.x);
    let (miny, maxy) = min_max(model, |v| v.y);
    let (minz, maxz) = min_max(model, |v| v.z);

    Obb {
        min: Vec3::new(minx, miny, minz),
        max: Vec3::new(maxx, maxy, maxz),
    }
}

fn min_max(model: &Model, f: impl Fn(&Vec3) -> f32) -> (f32, f32) {
    let min = model
        .verts
        .iter()
        .min_by(|a, b| f(a).total_cmp(&f(b)))
        .unwrap();
    let max = model
        .verts
        .iter()
        .max_by(|a, b| f(a).total_cmp(&f(b)))
        .unwrap();
    (f(min), f(max))
}

pub fn obb_visible(
    width: usize,
    height: usize,
    camera: &Camera,
    obb: Obb,
    translation: Vec3,
    pitch_yaw_roll: Vec3,
) -> bool {
    for v in obb_corners(obb, translation, pitch_yaw_roll).into_iter() {
        // TODO: Matrix multiplication would probably be more efficient, especially
        // for `vertex_camera_to_clip_space`
        if crate::math::vertex_world_to_camera_space_clipped(camera, v).is_some_and(|v| {
            let v = crate::math::vertex_camera_to_clip_space(width, height, camera, v);
            v.x >= -1.0 && v.x <= 1.0 && v.y >= -1.0 && v.y <= 1.0
        }) {
            return true;
        }
    }
    false
}

pub fn debug_draw_obb(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
    obb: Obb,
    translation: Vec3,
    pitch_yaw_roll: Vec3,
    color: Srgb,
) {
    let corners = obb_corners(obb, translation, pitch_yaw_roll);
    let vertices = [
        corners[0], corners[1], corners[2], corners[0], corners[2], corners[3], corners[5],
        corners[4], corners[7], corners[5], corners[7], corners[6], corners[4], corners[0],
        corners[3], corners[4], corners[3], corners[7], corners[1], corners[5], corners[6],
        corners[1], corners[6], corners[2], corners[4], corners[5], corners[1], corners[4],
        corners[1], corners[0], corners[3], corners[2], corners[6], corners[3], corners[6],
        corners[7],
    ];

    for face in vertices.chunks(3) {
        let v1 = face[0];
        let v2 = face[1];
        let v3 = face[2];

        if let Some((v1, v2, v3)) =
            crate::math::triangle_world_to_screen_space_clipped(width, height, camera, v1, v2, v3)
        {
            // TODO: Investigate z fighting
            rast::rast_triangle_wireframe_checked(
                frame_buffer,
                zbuffer,
                width,
                height,
                v1.x,
                v1.y,
                v1.z,
                v2.x,
                v2.y,
                v2.z,
                v3.x,
                v3.y,
                v3.z,
                color,
            );
        }
    }
}

pub fn obb_corners(obb: Obb, translation: Vec3, pitch_yaw_roll: Vec3) -> [Vec3; 8] {
    assert_obb(obb);
    let [minx, miny, minz] = obb.min.to_array();
    let [maxx, maxy, maxz] = obb.max.to_array();
    [
        Vec3::new(minx, miny, minz),
        Vec3::new(maxx, miny, minz),
        Vec3::new(maxx, maxy, minz),
        Vec3::new(minx, maxy, minz),
        Vec3::new(minx, miny, maxz),
        Vec3::new(maxx, miny, maxz),
        Vec3::new(maxx, maxy, maxz),
        Vec3::new(minx, maxy, maxz),
    ]
    .map(|v| crate::math::transform_vertex(translation, pitch_yaw_roll, v))
}

#[allow(unused)]
pub fn draw_model(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
    model: &Model,
    translation: Vec3,
    pitch_yaw_roll: Vec3,
) {
    draw_model_inner(
        frame_buffer,
        zbuffer,
        width,
        height,
        camera,
        model,
        translation,
        pitch_yaw_roll,
        false,
    );
}

#[allow(unused)]
pub fn draw_model_backface_culled(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
    model: &Model,
    translation: Vec3,
    pitch_yaw_roll: Vec3,
) {
    draw_model_inner(
        frame_buffer,
        zbuffer,
        width,
        height,
        camera,
        model,
        translation,
        pitch_yaw_roll,
        true,
    );
}

fn draw_model_inner(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
    model: &Model,
    translation: Vec3,
    pitch_yaw_roll: Vec3,
    backface: bool,
) {
    assert_model(model);
    for face in model.faces.chunks(3) {
        let v1 = transform_vertex(translation, pitch_yaw_roll, model.verts[face[0]]);
        let v2 = transform_vertex(translation, pitch_yaw_roll, model.verts[face[1]]);
        let v3 = transform_vertex(translation, pitch_yaw_roll, model.verts[face[2]]);

        if let Some((v1, v2, v3)) =
            crate::math::triangle_world_to_camera_space_clipped(camera, v1, v2, v3)
        {
            if backface {
                // https://en.wikipedia.org/wiki/Back-face_culling#Implementation
                let normal = (v3 - v1).cross(v2 - v1);
                if v1.dot(normal) < 0.0 {
                    continue;
                }
            }

            let (v1, v2, v3) = triangle_camera_to_screen_space(width, height, camera, v1, v2, v3);
            rast::rast_triangle_checked(
                frame_buffer,
                zbuffer,
                width,
                height,
                v1.x,
                v1.y,
                v1.z,
                v2.x,
                v2.y,
                v2.z,
                v3.x,
                v3.y,
                v3.z,
                LinearRgb::rgb(1.0, 0.0, 0.0),
                LinearRgb::rgb(0.0, 1.0, 0.0),
                LinearRgb::rgb(0.0, 0.0, 1.0),
                rast::ColorShader,
            );
        }
    }
}

fn assert_obb(obb: Obb) {
    debug_assert!(obb.min.x <= obb.max.x);
    debug_assert!(obb.min.y <= obb.max.y);
    debug_assert!(obb.min.z <= obb.max.z);
}

fn assert_model(model: &Model) {
    debug_assert!(!model.verts.is_empty());
    debug_assert!(!model.faces.is_empty());
    debug_assert!(model.faces.len() % 3 == 0);
}
