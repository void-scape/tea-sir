#![feature(vec_into_raw_parts)]
#![allow(clippy::too_many_arguments)]

use boids::*;
use math::*;
use rast::tint::*;

use crate::{
    blender::BlenderMemory,
    camera::{Camera, CameraController},
    neutron::NeutronMemory,
};

#[allow(unused)]
mod blender;
#[allow(unused)]
mod boids;
mod camera;
#[allow(unused)]
pub mod io;
#[allow(unused)]
pub mod math;
#[allow(unused)]
pub mod model;
#[allow(unused)]
mod neutron;
mod rng;

pub const MAX_WIDTH: usize = 640 * 2;
pub const MAX_HEIGHT: usize = 360 * 2;
pub const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;

const VOLUME: f32 = 0.0;

// Initialization code for the platform. The game code provides a static depth
// buffer and the game memory on startup. This memory is persisted when hot
// reloaded.

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
                translation: Vec3::new(0.0, 100.0, -100.0),
                pitch: 0.0,
                // pitch: 45f32.to_radians(),
                yaw: 0.0,
                fov: 90f32.to_radians(),
                nearz: 0.1,
                farz: 1_000.0,
            },
            divine_comedy: io::debug_audio_file("assets/divine-comedy.bin")
                .expect("could not load `divine-comedy.bin`"),
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

    #[allow(unused)]
    boid_memory: BoidMemory,
    #[allow(unused)]
    neutron_memory: NeutronMemory,
    #[allow(unused)]
    blender_memory: BlenderMemory,

    bg: f32,
    divine_comedy: Vec<i16>,
    play_cursor: usize,
}

#[unsafe(no_mangle)]
pub fn handle_input(glazer::PlatformInput { memory, input, .. }: glazer::PlatformInput<Memory>) {
    camera::handle_input(input, &mut memory.camera, &mut memory.controller);
}

#[unsafe(no_mangle)]
pub fn update_and_render(
    glazer::PlatformUpdate {
        memory,
        delta,
        //
        window,
        //
        frame_buffer,
        width,
        height,
        //
        // samples,
        // channels,
        // sample_rate,
        ..
    }: glazer::PlatformUpdate<Memory>,
) {
    window.set_title(&format!("Tea, Sir? - {:.2}", 1.0 / delta));

    // audio(memory, samples, channels, sample_rate as f32);
    camera::update_camera(&mut memory.camera, &memory.controller, delta);
    clear(memory, frame_buffer);

    // NEUTRON
    // neutron::update(&mut memory._neutron_memory, delta);
    // neutron::render(
    //     &mut memory._neutron_memory,
    //     frame_buffer,
    //     memory.zbuffer,
    //     width,
    //     height,
    //     &memory.camera,
    // );

    // BOIDS
    // boids::update(&mut memory._boid_memory, delta);
    // boids::render(
    //     &mut memory._boid_memory,
    //     frame_buffer,
    //     &mut memory.zbuffer,
    //     width,
    //     height,
    //     &memory.camera,
    // );

    // BLENDER
    blender::render(
        &mut memory.blender_memory,
        frame_buffer,
        memory.zbuffer,
        &memory.camera,
        width,
        height,
        delta,
    );
}

#[allow(unused)]
fn audio(memory: &mut Memory, samples: &mut [f32], channels: usize, sample_rate: f32) {
    assert_eq!(sample_rate, 44_100.0);
    assert_eq!(channels, 2);

    for sample in samples.iter_mut() {
        if memory.play_cursor >= memory.divine_comedy.len() {
            memory.play_cursor = 0;
        }
        *sample = memory.divine_comedy[memory.play_cursor] as f32 / i16::MAX as f32 * VOLUME;
        memory.play_cursor += 1;
    }
}

fn clear(memory: &mut Memory, frame_buffer: &mut [Srgb]) {
    frame_buffer.fill(Srgb::from_rgb(82, 82, 82));
    memory.zbuffer.fill(1.0);
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
