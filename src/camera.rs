use crate::math::*;
use glazer::winit::{
    event::{DeviceEvent, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
use rast::tint::*;

#[derive(Debug, Default)]
pub struct Camera {
    pub translation: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub fov: f32,
    pub nearz: f32,
    pub farz: f32,
}

#[derive(Debug, Default)]
pub struct CameraController {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub forward_pressed: bool,
    pub back_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
}

pub fn handle_input(input: glazer::Input, camera: &mut Camera, controller: &mut CameraController) {
    match input {
        glazer::Input::Window(event) => {
            if let WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } = event
            {
                let pressed = state.is_pressed();
                match code {
                    KeyCode::KeyW => {
                        controller.forward_pressed = pressed;
                    }
                    KeyCode::KeyS => {
                        controller.back_pressed = pressed;
                    }
                    KeyCode::KeyA => {
                        controller.left_pressed = pressed;
                    }
                    KeyCode::KeyD => {
                        controller.right_pressed = pressed;
                    }
                    KeyCode::Space => {
                        controller.up_pressed = pressed;
                    }
                    KeyCode::ShiftLeft => {
                        controller.down_pressed = pressed;
                    }
                    _ => {}
                }
            }
        }
        glazer::Input::Device(DeviceEvent::MouseMotion { delta }) => {
            let sensitivity = 0.005;
            camera.yaw += delta.0 as f32 * sensitivity;
            camera.pitch += delta.1 as f32 * sensitivity;

            // Keep the camera's angle from going too high/low.
            const SAFE_FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;
            if camera.pitch < -SAFE_FRAC_PI_2 {
                camera.pitch = -SAFE_FRAC_PI_2;
            } else if camera.pitch > SAFE_FRAC_PI_2 {
                camera.pitch = SAFE_FRAC_PI_2;
            }
        }
        _ => {}
    }
}

pub fn update_camera(camera: &mut Camera, controller: &CameraController, delta: f32) {
    let speed = 100.0 * delta;
    let mut camera_delta = Vec3::ZERO;

    if controller.forward_pressed {
        camera_delta.z += 1.0;
    }
    if controller.back_pressed {
        camera_delta.z -= 1.0;
    }
    if controller.right_pressed {
        camera_delta.x += 1.0;
    }
    if controller.left_pressed {
        camera_delta.x -= 1.0;
    }
    if controller.up_pressed {
        camera_delta.y += 1.0;
    }
    if controller.down_pressed {
        camera_delta.y -= 1.0;
    }

    if camera_delta != Vec3::ZERO {
        camera.translation += (camera_delta.normalize() * speed).rotate_y(camera.yaw);
    }
}

#[allow(unused)]
pub fn debug_draw_frustum(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera_to_debug: &Camera,
    camera_for_view: &Camera,
) {
    debug_assert_eq!(frame_buffer.len(), zbuffer.len());
    let camera = camera_to_debug;
    let fov_2y = camera.fov / 2.0;
    // NOTE: The angles are not linear, so you can't just do `fov_2y * aspect`. I
    // am not sure why...
    let fov_2x = libm::atanf(libm::tanf(fov_2y) * width as f32 / height as f32);

    let near_height = camera.nearz * libm::tanf(fov_2y);
    let near_width = camera.nearz * libm::tanf(fov_2x);
    let far_height = camera.farz * libm::tanf(fov_2y);
    let far_width = camera.farz * libm::tanf(fov_2x);

    let corners = [
        Vec3::new(-near_width, -near_height, camera.nearz),
        Vec3::new(near_width, -near_height, camera.nearz),
        Vec3::new(near_width, near_height, camera.nearz),
        Vec3::new(-near_width, near_height, camera.nearz),
        Vec3::new(-far_width, -far_height, camera.farz),
        Vec3::new(far_width, -far_height, camera.farz),
        Vec3::new(far_width, far_height, camera.farz),
        Vec3::new(-far_width, far_height, camera.farz),
    ]
    .map(|corner| corner.rotate_x(-camera.pitch).rotate_y(camera.yaw) + camera.translation);

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

        if let Some((v1, v2, v3)) = crate::math::triangle_world_to_screen_space_clipped(
            width,
            height,
            &camera_for_view,
            v1,
            v2,
            v3,
        ) {
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
                Srgb::from_rgb(0, 0, 255),
            );
        }
    }
}
