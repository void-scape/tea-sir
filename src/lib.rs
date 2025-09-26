#![feature(vec_into_raw_parts)]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use math::*;
use rast::tint::*;

use crate::{
    camera::{Camera, CameraController},
    model::Model,
};

mod camera;
mod io;
pub mod math;
mod model;

pub const MAX_WIDTH: usize = 640 * 2;
pub const MAX_HEIGHT: usize = 360 * 2;
pub const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;

// Initilization code for the platform. The game code provides a static frame
// buffer and the game memory on startup. This memory is persisted when hot
// reloaded.

pub fn frame_buffer() -> &'static mut [Srgb] {
    static mut FRAME_BUFFER: [Srgb; MAX_PIXELS] = [Srgb::new(255, 255, 255, 255); MAX_PIXELS];
    static mut INIT: bool = false;

    // ## Safety
    //
    // `FRAME_BUFFER` is locally scoped. `INIT` verifies that this function
    // has only been called once. There cannot exist any other mutable references
    // to `FRAME_BUFFER` with safe Rust code.
    unsafe {
        if INIT {
            panic!("tried to call `frame_buffer` twice");
        }
        INIT = true;
        #[allow(static_mut_refs)]
        &mut FRAME_BUFFER
    }
}

pub fn memory() -> Memory<'static> {
    static mut DEPTH_BUFFER: [f32; MAX_PIXELS] = [1.0; MAX_PIXELS];
    static mut INIT: bool = false;

    // ## Safety
    //
    // `DEPTH_BUFFER` is locally scoped. `INIT` verifies that this function
    // has only been called once. There cannot exist any other mutable references
    // to `DEPTH_BUFFER` with safe Rust code.
    unsafe {
        if INIT {
            panic!("tried to call `memory` twice");
        }
        INIT = true;
        Memory {
            #[allow(static_mut_refs)]
            zbuffer: &mut DEPTH_BUFFER,
            camera: Camera {
                translation: Vec3::new(-4.0, 0.0, -4.0),
                pitch: 0.0,
                yaw: 20f32.to_radians(),
                fov: 100.0,
                nearz: 0.1,
                farz: 1000.0,
            },
            dummy_camera: Camera {
                translation: Vec3::new(0.0, 1.5, -8.0),
                pitch: 0.0,
                yaw: 0.0,
                fov: 75.0,
                nearz: 0.1,
                farz: 10.0,
            },
            divine_comedy: io::debug_audio_file("assets/divine_comedy.bin")
                .expect("could not load `divine_comedy.bin`"),
            teapot: io::debug_obj_file("assets/teapot.obj").expect("could not load `teapot.obj`"),
            ..Default::default()
        }
    }
}

// Game state.

#[derive(Default)]
pub struct Memory<'a> {
    zbuffer: &'a mut [f32],

    camera: Camera,
    controller: CameraController,
    dummy_camera: Camera,

    bg: f32,
    angle: f32,
    divine_comedy: Vec<i16>,
    teapot: Model,
    play_cursor: usize,
}

pub fn handle_input(glazer::PlatformInput { memory, input }: glazer::PlatformInput<Memory>) {
    camera::handle_input(input, &mut memory.camera, &mut memory.controller);
}

pub fn update_and_render(
    glazer::PlatformUpdate {
        memory,
        delta,
        //
        frame_buffer,
        width,
        height,
        //
        samples,
        channels,
        sample_rate,
        ..
    }: glazer::PlatformUpdate<Memory, Srgb>,
) {
    camera::update_camera(&mut memory.camera, &memory.controller, delta);
    audio(memory, samples, channels, sample_rate);
    render(memory, frame_buffer, width, height, delta);
}

fn audio(memory: &mut Memory, samples: &mut [i16], channels: usize, sample_rate: f32) {
    assert_eq!(sample_rate, 44_100.0);
    assert_eq!(channels, 2);

    for i in 0..samples.len() {
        if memory.play_cursor >= memory.divine_comedy.len() {
            memory.play_cursor = 0;
        }
        samples[i] = memory.divine_comedy[memory.play_cursor];
        memory.play_cursor += 1;
    }
}

fn render(memory: &mut Memory, frame_buffer: &mut [Srgb], width: usize, height: usize, delta: f32) {
    frame_buffer.fill(Srgb::rgb(82, 82, 82));

    memory.zbuffer.fill(1.0);
    memory.angle = (memory.angle + delta) % core::f32::consts::TAU;
    memory.dummy_camera.yaw = memory.angle;

    let obb = model::compute_obb(&memory.teapot);
    for x in -1..=1 {
        let translation = Vec3::x(x as f32 * 10.0);
        let pyr = Vec3::new(memory.angle, memory.angle, memory.angle);
        if model::obb_visible(width, height, &memory.camera, obb, translation, pyr) {
            model::draw_model(
                frame_buffer,
                memory.zbuffer,
                width,
                height,
                &memory.camera,
                &memory.teapot,
                translation,
                pyr,
            );

            let visible =
                model::obb_visible(width, height, &memory.dummy_camera, obb, translation, pyr);
            let color = if visible {
                Srgb::rgb(0, 255, 0)
            } else {
                Srgb::rgb(255, 0, 0)
            };
            model::debug_draw_obb(
                frame_buffer,
                memory.zbuffer,
                width,
                height,
                &memory.camera,
                obb,
                translation,
                pyr,
                color,
            );
        }
    }

    camera::debug_draw_frustum(
        frame_buffer,
        memory.zbuffer,
        width,
        height,
        &memory.dummy_camera,
        &memory.camera,
    );
}

#[expect(unused)]
fn fill_background(
    memory: &mut Memory,
    frame_buffer: &mut [Srgb],
    width: usize,
    height: usize,
    delta: f32,
) {
    memory.bg += delta * 50.0;
    memory.bg %= 255.0;
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            if memory.zbuffer[index] == f32::MAX {
                let r = ((x as f32 + memory.bg) % 255.0) as u8;
                let g = 0;
                let b = ((y as f32 + memory.bg) % 255.0) as u8;
                frame_buffer[index] = Srgb::new(r, g, b, 255);
            }
        }
    }
}
