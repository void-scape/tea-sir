mod vec;

pub use vec::*;

use crate::camera::Camera;

pub fn transform_vertex(translation: Vec3, pitch_yaw_roll: Vec3, v: Vec3) -> Vec3 {
    let mut rotated = v;
    if pitch_yaw_roll.z != 0.0 {
        rotated = rotated.rotate_z(pitch_yaw_roll.z);
    }
    if pitch_yaw_roll.y != 0.0 {
        rotated = rotated.rotate_y(pitch_yaw_roll.y);
    }
    if pitch_yaw_roll.x != 0.0 {
        rotated = rotated.rotate_x(pitch_yaw_roll.x);
    }
    rotated + translation
}

pub fn triangle_world_to_screen_space_clipped(
    width: usize,
    height: usize,
    camera: &Camera,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
) -> Option<(Vec3, Vec3, Vec3)> {
    triangle_world_to_camera_space_clipped(camera, v1, v2, v3)
        .map(|(v1, v2, v3)| triangle_camera_to_screen_space(width, height, camera, v1, v2, v3))
}

pub fn vertex_world_to_screen_space_clipped(
    width: usize,
    height: usize,
    camera: &Camera,
    v: Vec3,
) -> Option<Vec3> {
    vertex_world_to_camera_space_clipped(camera, v)
        .map(|v| vertex_camera_to_screen_space(width, height, camera, v))
}

pub fn triangle_world_to_camera_space_clipped(
    camera: &Camera,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
) -> Option<(Vec3, Vec3, Vec3)> {
    vertex_world_to_camera_space_clipped(camera, v1).and_then(|v1| {
        vertex_world_to_camera_space_clipped(camera, v2)
            .and_then(|v2| vertex_world_to_camera_space_clipped(camera, v3).map(|v3| (v1, v2, v3)))
    })
}

pub fn vertex_world_to_camera_space_clipped(camera: &Camera, v: Vec3) -> Option<Vec3> {
    let camera_space = (v - camera.translation)
        .rotate_y(-camera.yaw)
        .rotate_x(-camera.pitch);
    (camera_space.z > camera.nearz).then_some(camera_space)
}

pub fn triangle_camera_to_screen_space(
    width: usize,
    height: usize,
    camera: &Camera,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
) -> (Vec3, Vec3, Vec3) {
    (
        vertex_camera_to_screen_space(width, height, camera, v1),
        vertex_camera_to_screen_space(width, height, camera, v2),
        vertex_camera_to_screen_space(width, height, camera, v3),
    )
}

pub fn vertex_camera_to_clip_space(width: usize, height: usize, camera: &Camera, v: Vec3) -> Vec2 {
    // https://en.wikipedia.org/wiki/3D_projection
    // TODO: Precompute fov_scale
    let fov_scale = 1.0 / (camera.fov.to_radians() / 2.0).tan();
    let mut proj = v.to_vec2() * fov_scale / v.z;
    proj.x *= height as f32 / width as f32;
    proj
}

pub fn vertex_camera_to_screen_space(
    width: usize,
    height: usize,
    camera: &Camera,
    v: Vec3,
) -> Vec3 {
    let proj = vertex_camera_to_clip_space(width, height, camera, v);
    Vec3::new(
        (proj.x + 1.0) / 2.0 * width as f32,
        (1.0 - (proj.y + 1.0) / 2.0) * height as f32,
        (v.z - camera.nearz) / camera.farz,
    )
}
