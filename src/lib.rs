#![feature(vec_into_raw_parts)]
#![no_std]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use boids::*;
use math::*;
use rast::tint::*;

use crate::{
    camera::{Camera, CameraController},
    model::{Model, Obb},
    neutron::NeutronMemory,
};

mod boids;
mod camera;
mod io;
pub mod math;
mod model;
mod neutron;
mod rng;

pub const MAX_WIDTH: usize = 640 * 2;
pub const MAX_HEIGHT: usize = 360 * 2;
pub const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;

// Initialization code for the platform. The game code provides a static frame
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
        Memory {
            #[allow(static_mut_refs)]
            zbuffer: &mut DEPTH_BUFFER,
            camera: Camera {
                translation: Vec3::new(0.0, 15.0, -15.0),
                pitch: 45f32.to_radians(),
                yaw: 0.0,
                fov: 80.0,
                nearz: 0.1,
                farz: 1000.0,
            },
            divine_comedy: io::debug_audio_file("assets/divine-comedy.bin")
                .expect("could not load `divine-comedy.bin`"),
            teapot: io::debug_obj_file("assets/teapot.obj", Vec::new())
                .expect("could not load `teapot.obj`"),
            ibuki_obb: model::compute_obb(&ibuki),
            ibuki,
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

    _boid_memory: BoidMemory,
    neutron_memory: NeutronMemory,

    bg: f32,
    angle: f32,
    divine_comedy: Vec<i16>,
    teapot: Model,
    ibuki: Model,
    ibuki_obb: Obb,
    play_cursor: usize,
}

#[unsafe(no_mangle)]
pub fn handle_input(glazer::PlatformInput { memory, input }: glazer::PlatformInput<Memory>) {
    camera::handle_input(input, &mut memory.camera, &mut memory.controller);
}

#[unsafe(no_mangle)]
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
    audio(memory, samples, channels, sample_rate);
    camera::update_camera(&mut memory.camera, &memory.controller, delta);

    neutron::update(&mut memory.neutron_memory, delta);
    neutron::render(
        &mut memory.neutron_memory,
        frame_buffer,
        &mut memory.zbuffer,
        width,
        height,
        &memory.camera,
    );

    // clear(memory, frame_buffer);
    // boids::update(&mut memory.boid_memory, delta);
    // boids::render(
    //     &mut memory.boid_memory,
    //     frame_buffer,
    //     &mut memory.zbuffer,
    //     width,
    //     height,
    //     &memory.camera,
    // );
    //
    // render(memory, frame_buffer, width, height, delta);
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

#[allow(unused)]
fn clear(memory: &mut Memory, frame_buffer: &mut [Srgb]) {
    frame_buffer.fill(Srgb::rgb(82, 82, 82));
    memory.zbuffer.fill(1.0);
}

#[allow(unused)]
fn render(memory: &mut Memory, frame_buffer: &mut [Srgb], width: usize, height: usize, delta: f32) {
    memory.angle = (memory.angle + delta) % core::f32::consts::TAU;

    let obb = memory.ibuki_obb;
    let translation = Vec3::ZERO;
    // let pyr = Vec3::ZERO;
    let pyr = Vec3::new(0.0, memory.angle, 0.0);
    if model::obb_visible(width, height, &memory.camera, obb, translation, pyr) {
        model::draw_model(
            frame_buffer,
            memory.zbuffer,
            width,
            height,
            &memory.camera,
            &memory.ibuki,
            translation,
            pyr,
        );

        model::debug_draw_obb(
            frame_buffer,
            memory.zbuffer,
            width,
            height,
            &memory.camera,
            obb,
            translation,
            pyr,
            Srgb::rgb(0, 255, 0),
        );
    }

    let obb = model::compute_obb(&memory.teapot);
    for x in -1..=1 {
        let translation = Vec3::x(x as f32 * 10.0 + 50.0);
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

            model::debug_draw_obb(
                frame_buffer,
                memory.zbuffer,
                width,
                height,
                &memory.camera,
                obb,
                translation,
                pyr,
                Srgb::rgb(0, 255, 0),
            );
        }
    }
}

#[expect(unused)]
fn draw_quad(memory: &mut Memory, frame_buffer: &mut [Srgb], width: usize, height: usize) {
    let scale = 200.0;
    let offset = 500.0;
    let pyr = Vec3::z(memory.angle);
    let corners = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ]
    .map(|v| math::transform_vertex(Vec3::new(offset, offset, offset), pyr, v * scale));

    rast::rast_quad(
        frame_buffer,
        width,
        height,
        corners[0].x as i32,
        corners[0].y as i32,
        corners[1].x as i32,
        corners[1].y as i32,
        corners[2].x as i32,
        corners[2].y as i32,
        corners[3].x as i32,
        corners[3].y as i32,
        Srgb::rgb(255, 0, 0).into(),
        Srgb::rgb(0, 255, 0).into(),
        Srgb::rgb(0, 0, 255).into(),
        Srgb::rgb(255, 255, 255).into(),
        rast::ColorShader,
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
