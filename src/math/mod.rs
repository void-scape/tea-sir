mod mat;
mod quat;
mod vec;

pub use mat::*;
pub use quat::*;
pub use vec::*;

// These matrix computations were stolen from `glam`.

#[inline]
#[must_use]
pub fn compute_model_matrix(translation: Vec3, rotation: Quat, scale: Vec3) -> Mat4 {
    let mut mat = rotation.compute_mat4();
    mat.r1.x *= scale.x;
    mat.r2.x *= scale.x;
    mat.r3.x *= scale.x;
    mat.r1.y *= scale.y;
    mat.r2.y *= scale.y;
    mat.r3.y *= scale.y;
    mat.r1.z *= scale.z;
    mat.r2.z *= scale.z;
    mat.r3.z *= scale.z;
    mat.r1.w = translation.x;
    mat.r2.w = translation.y;
    mat.r3.w = translation.z;
    mat
}

#[inline]
#[must_use]
pub fn compute_view_matrix(translation: Vec3, yaw: f32, pitch: f32) -> Mat4 {
    // https://www.3dgep.com/understanding-the-view-matrix/#The_View_Matrix

    let (ysin, ycos) = libm::sincosf(-yaw);
    let (psin, pcos) = libm::sincosf(-pitch);

    let xaxis = Vec3::new(ycos, 0.0, -ysin);
    let yaxis = Vec3::new(ysin * psin, pcos, ycos * psin);
    let zaxis = Vec3::new(ysin * pcos, -psin, pcos * ycos);

    Mat4 {
        r1: xaxis.extend(-xaxis.dot(translation)),
        r2: yaxis.extend(-yaxis.dot(translation)),
        r3: (-zaxis).extend(zaxis.dot(translation)),
        r4: Vec4::w(1.0),
    }
}

#[inline]
#[must_use]
pub fn compute_perspective_proj_matrix(camera: &Camera, width: usize, height: usize) -> Mat4 {
    let (sin_fov, cos_fov) = libm::sincosf(0.5 * camera.fov);
    let h = cos_fov / sin_fov;
    let w = h * height as f32 / width as f32;
    let r = camera.farz / (camera.nearz - camera.farz);
    Mat4 {
        r1: Vec4::new(w, 0.0, 0.0, 0.0),
        r2: Vec4::new(0.0, h, 0.0, 0.0),
        r3: Vec4::new(0.0, 0.0, r, r * camera.nearz),
        r4: Vec4::new(0.0, 0.0, -1.0, 0.0),
    }
}

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

pub fn vertex_world_to_screen_space_clipped(
    width: usize,
    height: usize,
    camera: &Camera,
    v: Vec3,
) -> Option<Vec3> {
    vertex_world_to_camera_space_clipped(camera, v)
        .map(|v| vertex_camera_to_screen_space(width, height, camera, v))
}

pub fn vertex_world_to_camera_space_clipped(camera: &Camera, v: Vec3) -> Option<Vec3> {
    let camera_space = (v - camera.translation)
        .rotate_y(-camera.yaw)
        .rotate_x(-camera.pitch);
    (camera_space.z >= camera.nearz && camera_space.z <= camera.farz).then_some(camera_space)
}

pub fn vertex_world_to_camera_space(camera: &Camera, v: Vec3) -> Vec3 {
    (v - camera.translation)
        .rotate_y(-camera.yaw)
        .rotate_x(-camera.pitch)
}

pub fn vertex_camera_to_clip_space(width: usize, height: usize, camera: &Camera, v: Vec3) -> Vec2 {
    // https://en.wikipedia.org/wiki/3D_projection
    // TODO: Precompute fov_scale
    let fov_scale = 1.0 / (camera.fov / 2.0).tan();
    let mut proj = v.reduce() * fov_scale / v.z;
    proj.x *= height as f32 / width as f32;
    proj
}

pub fn vertex_world_to_screen_space(width: usize, height: usize, camera: &Camera, v: Vec3) -> Vec3 {
    let v = vertex_world_to_camera_space(camera, v);
    let proj = vertex_camera_to_clip_space(width, height, camera, v);
    Vec3::new(
        (proj.x + 1.0) / 2.0 * width as f32,
        (1.0 - (proj.y + 1.0) / 2.0) * height as f32,
        (v.z - camera.nearz) / camera.farz,
    )
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
