use crate::{camera::Camera, math::*};
use glam::Vec4Swizzles;
use rast::tint::*;

#[derive(Default)]
pub struct Model {
    pub faces: Vec<usize>,
    // (uv index, texture index)
    pub face_textures: Vec<(usize, usize)>,

    pub verts: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub textures: Vec<(usize, usize, Vec<Srgb>)>,
}

#[derive(Debug, Default, Clone, Copy)]
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
    debug_assert_eq!(frame_buffer.len(), zbuffer.len());
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
                libm::floorf(v1.x) as i32,
                libm::floorf(v1.y) as i32,
                v1.z,
                libm::floorf(v2.x) as i32,
                libm::floorf(v2.y) as i32,
                v2.z,
                libm::floorf(v3.x) as i32,
                libm::floorf(v3.y) as i32,
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

pub fn draw_model_matrix(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
    model: &Model,
    translation: glam::Vec3,
    scale: glam::Vec3,
    rotation: glam::Quat,
) {
    draw_model_inner_matrix(
        frame_buffer,
        zbuffer,
        width,
        height,
        camera,
        model,
        translation,
        scale,
        rotation,
        false,
    );
}

fn draw_model_inner_matrix(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
    model: &Model,
    translation: glam::Vec3,
    scale: glam::Vec3,
    rotation: glam::Quat,
    backface: bool,
) {
    assert_model(model);
    debug_assert_eq!(frame_buffer.len(), zbuffer.len());

    let model_matrix = glam::Mat4::from_scale_rotation_translation(
        scale,
        rotation,
        glam::Vec3::new(0.0, -0.1, -1.0),
    );
    // let view_matrix = create_rh_view_matrix(
    //     glam::Vec3::new(
    //         camera.translation.x,
    //         camera.translation.y,
    //         camera.translation.z,
    //     ),
    //     camera.pitch,
    //     camera.yaw,
    // );
    let projection_matrix = glam::Mat4::perspective_rh(
        camera.fov,
        width as f32 / height as f32,
        camera.nearz,
        camera.farz,
    );

    let model_to_proj_matrix = projection_matrix
        // .mul_mat4(&view_matrix)
        .mul_mat4(&model_matrix);

    // fn calculate_camera_vectors(pitch: f32, yaw: f32) -> (glam::Vec3, glam::Vec3) {
    //     // 1. Calculate the Forward/Direction Vector (dir)
    //     let (sin_pitch, cos_pitch) = pitch.sin_cos();
    //     let (sin_yaw, cos_yaw) = yaw.sin_cos();
    //
    //     // Assuming Yaw is rotation around Y-axis (Up) and Pitch is rotation around local X-axis.
    //     let x = cos_pitch * sin_yaw;
    //     let y = sin_pitch;
    //     let z = cos_pitch * cos_yaw;
    //
    //     let dir = glam::Vec3::new(x, y, z).normalize();
    //
    //     // 2. Define the Up Vector
    //     // Since we are Y-up, the world Up vector is (0, 1, 0).
    //     let up = glam::Vec3::Y;
    //
    //     (dir, up)
    // }
    //
    // pub fn create_rh_view_matrix(translation: glam::Vec3, pitch: f32, yaw: f32) -> glam::Mat4 {
    //     // The camera's position in world space.
    //     let eye = translation;
    //
    //     let (dir, up) = calculate_camera_vectors(pitch, yaw);
    //
    //     // Mat4::look_to_rh (Right-Handed) takes:
    //     // 1. eye (The camera position/translation)
    //     // 2. dir (The direction vector the camera is looking)
    //     // 3. up (The 'up' direction in world space, usually +Y)
    //     glam::Mat4::look_to_rh(eye, dir, up)
    // }

    // let view_matrix = compute_view_matrix(camera.translation, camera.yaw, camera.pitch);
    // let proj_matrix = compute_perspective_proj_matrix(camera, width, height);

    fn camera_to_screen_space(width: usize, height: usize, mut v: glam::Vec4) -> glam::Vec3 {
        v.x /= v.w;
        v.y /= v.w;
        v.z /= v.w;
        glam::Vec3::new(
            (v.x + 1.0) / 2.0 * width as f32,
            (1.0 - (v.y + 1.0) / 2.0) * height as f32,
            v.z,
        )
    }

    // let model_to_view_matrix = view_matrix.mult_mat4(&model_matrix);
    // let model_to_proj_matrix = proj_matrix.mult_mat4(&model_to_view_matrix);

    let mut c = 1.0;
    let diff = 0.8 / (model.faces.len() / 3) as f32;

    if model.textures.is_empty() {
        for face in model.faces.chunks(3) {
            let mv1 = model.verts[face[0]].extend(1.0);
            let mv2 = model.verts[face[1]].extend(1.0);
            let mv3 = model.verts[face[2]].extend(1.0);

            let mv1 = glam::Vec4::new(mv1.x, mv1.y, mv1.z, mv1.w);
            let mv2 = glam::Vec4::new(mv2.x, mv2.y, mv2.z, mv2.w);
            let mv3 = glam::Vec4::new(mv3.x, mv3.y, mv3.z, mv3.w);

            let v1 = model_to_proj_matrix.mul_vec4(mv1);
            let v2 = model_to_proj_matrix.mul_vec4(mv2);
            let v3 = model_to_proj_matrix.mul_vec4(mv3);

            let v1 = camera_to_screen_space(width, height, v1);
            let v2 = camera_to_screen_space(width, height, v2);
            let v3 = camera_to_screen_space(width, height, v3);

            // let v1 = model_to_view_matrix.mult_vec4(mv1).reduce();
            // let v2 = model_to_view_matrix.mult_vec4(mv2).reduce();
            // let v3 = model_to_view_matrix.mult_vec4(mv3).reduce();
            //
            // let v1z = v1.z;
            // let v2z = v2.z;
            // let v3z = v3.z;
            //
            // if v1z <= camera.nearz
            //     || v1z >= camera.farz
            //     || v2z <= camera.nearz
            //     || v2z >= camera.farz
            //     || v3z <= camera.nearz
            //     || v3z >= camera.farz
            // {
            //     continue;
            // }
            //
            // if backface {
            //     // https://en.wikipedia.org/wiki/Back-face_culling#Implementation
            //     let normal = (v3 - v1).cross(v2 - v1);
            //     if v1.dot(normal) < 0.0 {
            //         continue;
            //     }
            // }

            // let v1 = model_to_proj_matrix.mult_vec4(mv1);
            // let v2 = model_to_proj_matrix.mult_vec4(mv2);
            // let v3 = model_to_proj_matrix.mult_vec4(mv3);

            // let v1 = camera_to_screen_space(width, height, v1);
            // let v2 = camera_to_screen_space(width, height, v2);
            // let v3 = camera_to_screen_space(width, height, v3);

            rast::rast_triangle_checked(
                frame_buffer,
                zbuffer,
                width,
                height,
                libm::floorf(v1.x) as i32,
                libm::floorf(v1.y) as i32,
                v1.z,
                libm::floorf(v2.x) as i32,
                libm::floorf(v2.y) as i32,
                v2.z,
                libm::floorf(v3.x) as i32,
                libm::floorf(v3.y) as i32,
                v3.z,
                LinearRgb::from_rgb(c, c, c),
                LinearRgb::from_rgb(c, c, c),
                LinearRgb::from_rgb(c, c, c),
                rast::ColorShader::default(),
            );

            c -= diff;
        }
    } else {
        todo!("update to glam");
        // for (face, face_textures) in model.faces.chunks(3).zip(model.face_textures.chunks(3)) {
        //     let mv1 = model.verts[face[0]].extend(1.0);
        //     let mv2 = model.verts[face[1]].extend(1.0);
        //     let mv3 = model.verts[face[2]].extend(1.0);
        //
        //     let v1 = model_to_view_matrix.mult_vec4(mv1).reduce();
        //     let v2 = model_to_view_matrix.mult_vec4(mv2).reduce();
        //     let v3 = model_to_view_matrix.mult_vec4(mv3).reduce();
        //
        //     let v1z = v1.z;
        //     let v2z = v2.z;
        //     let v3z = v3.z;
        //
        //     if v1z <= camera.nearz
        //         || v1z >= camera.farz
        //         || v2z <= camera.nearz
        //         || v2z >= camera.farz
        //         || v3z <= camera.nearz
        //         || v3z >= camera.farz
        //     {
        //         continue;
        //     }
        //
        //     if backface {
        //         // https://en.wikipedia.org/wiki/Back-face_culling#Implementation
        //         let normal = (v3 - v1).cross(v2 - v1);
        //         if v1.dot(normal) < 0.0 {
        //             continue;
        //         }
        //     }
        //
        //     let v1 = model_to_proj_matrix.mult_vec4(mv1);
        //     let v2 = model_to_proj_matrix.mult_vec4(mv2);
        //     let v3 = model_to_proj_matrix.mult_vec4(mv3);
        //
        //     let v1 = camera_to_screen_space(width, height, v1);
        //     let v2 = camera_to_screen_space(width, height, v2);
        //     let v3 = camera_to_screen_space(width, height, v3);
        //
        //     debug_assert_eq!(face_textures[0].1, face_textures[1].1);
        //     debug_assert_eq!(face_textures[2].1, face_textures[1].1);
        //
        //     let texture = &model.textures[face_textures[0].1];
        //     let (uv, _) = face_textures[0];
        //     let uv1 = model.uvs[uv];
        //     let (uv, _) = face_textures[1];
        //     let uv2 = model.uvs[uv];
        //     let (uv, _) = face_textures[2];
        //     let uv3 = model.uvs[uv];
        //
        //     rast::rast_triangle_checked(
        //         frame_buffer,
        //         zbuffer,
        //         width,
        //         height,
        //         libm::floorf(v1.x) as i32,
        //         libm::floorf(v1.y) as i32,
        //         v1.z,
        //         libm::floorf(v2.x) as i32,
        //         libm::floorf(v2.y) as i32,
        //         v2.z,
        //         libm::floorf(v3.x) as i32,
        //         libm::floorf(v3.y) as i32,
        //         v3.z,
        //         (uv1.x, uv1.y),
        //         (uv2.x, uv2.y),
        //         (uv3.x, uv3.y),
        //         rast::TextureShader {
        //             width: texture.0,
        //             height: texture.1,
        //             texture: texture.2.as_slice(),
        //             sampler: rast::Sampler::Bilinear,
        //             blend_mode: rast::BlendMode::None,
        //         },
        //     );
        // }
    }
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
    debug_assert_eq!(frame_buffer.len(), zbuffer.len());

    if model.textures.is_empty() {
        draw_model_inner_no_textures(
            frame_buffer,
            zbuffer,
            width,
            height,
            camera,
            model,
            translation,
            pitch_yaw_roll,
            backface,
        );
        return;
    }

    for (face, face_textures) in model.faces.chunks(3).zip(model.face_textures.chunks(3)) {
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

            debug_assert_eq!(face_textures[0].1, face_textures[1].1);
            debug_assert_eq!(face_textures[2].1, face_textures[1].1);

            let texture = &model.textures[face_textures[0].1];
            let (uv, _) = face_textures[0];
            let uv1 = model.uvs[uv];
            let (uv, _) = face_textures[1];
            let uv2 = model.uvs[uv];
            let (uv, _) = face_textures[2];
            let uv3 = model.uvs[uv];

            let (v1, v2, v3) = triangle_camera_to_screen_space(width, height, camera, v1, v2, v3);
            rast::rast_triangle_checked(
                frame_buffer,
                zbuffer,
                width,
                height,
                libm::floorf(v1.x) as i32,
                libm::floorf(v1.y) as i32,
                v1.z,
                libm::floorf(v2.x) as i32,
                libm::floorf(v2.y) as i32,
                v2.z,
                libm::floorf(v3.x) as i32,
                libm::floorf(v3.y) as i32,
                v3.z,
                (uv1.x, uv1.y),
                (uv2.x, uv2.y),
                (uv3.x, uv3.y),
                rast::TextureShader {
                    width: texture.0,
                    height: texture.1,
                    texture: texture.2.as_slice(),
                    sampler: rast::Sampler::Bilinear,
                    blend_mode: rast::BlendMode::None,
                },
            );
        }
    }
}

fn draw_model_inner_no_textures(
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
                libm::floorf(v1.x) as i32,
                libm::floorf(v1.y) as i32,
                v1.z,
                libm::floorf(v2.x) as i32,
                libm::floorf(v2.y) as i32,
                v2.z,
                libm::floorf(v3.x) as i32,
                libm::floorf(v3.y) as i32,
                v3.z,
                LinearRgb::from_rgb(1.0, 0.0, 0.0),
                LinearRgb::from_rgb(0.0, 1.0, 0.0),
                LinearRgb::from_rgb(0.0, 0.0, 1.0),
                rast::ColorShader::default(),
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
    debug_assert!(model.faces.len().is_multiple_of(3));
    debug_assert!(model.face_textures.len().is_multiple_of(3));
    debug_assert!(
        model.face_textures.is_empty() == model.uvs.is_empty()
            && model.uvs.is_empty() == model.textures.is_empty()
    );
    if !model.face_textures.is_empty() {
        debug_assert_eq!(model.faces.len(), model.face_textures.len());
    }
}
