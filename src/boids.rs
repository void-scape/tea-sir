#![allow(unused)]

use crate::{camera::Camera, math::*};
use rast::tint::*;

const BOUNDS: f32 = 240.0;
const BOID_COUNT: usize = 1200 * 3;
const MAX_SPEED: f32 = 100.0;
const MAX_SPEED_SQ: f32 = MAX_SPEED * MAX_SPEED;
const MIN_SPEED: f32 = 60.0;
const MIN_SPEED_SQ: f32 = MIN_SPEED * MIN_SPEED;

pub struct BoidMemory {
    boids: [Boid; BOID_COUNT],
    vertices: [Vec3; 36],
    //
    margin: f32,
    turn_factor: f32,
    //
    separation_factor: f32,
    cohesion_factor: f32,
    alignment_factor: f32,
    //
    view_radius_squared: f32,
    separation_radius_squared: f32,
}

impl Default for BoidMemory {
    fn default() -> Self {
        BoidMemory {
            boids: core::array::from_fn(|i| Boid {
                translation: Vec3::new(
                    crate::rng::sample_f32(i * 3) as f32,
                    crate::rng::sample_f32(i * 3 + 1) as f32,
                    crate::rng::sample_f32(i * 3 + 2) as f32,
                )
                .normalize_or_zero()
                    * 2.0
                    * BOUNDS
                    - BOUNDS,
                velocity: Vec3::new(
                    crate::rng::sample_f32(i * 3) as f32,
                    crate::rng::sample_f32(i * 3 + 1) as f32,
                    crate::rng::sample_f32(i * 3 + 2) as f32,
                )
                .normalize_or_zero()
                    * 2.0
                    * MAX_SPEED
                    - MAX_SPEED,
            }),
            vertices: boid_vertices(),
            //
            turn_factor: 1.0,
            margin: BOUNDS / 4.0,
            //
            separation_factor: 0.015,
            cohesion_factor: 0.0005,
            alignment_factor: 0.01,
            //
            view_radius_squared: libm::powf(18.0, 2.0),
            separation_radius_squared: libm::powf(8.0, 2.0),
        }
    }
}

pub fn update(memory: &mut BoidMemory, dt: f32) {
    boid_forces(memory);
    avoid_bounds(memory);
    apply_velocity(memory, dt);
}

pub fn render(
    memory: &mut BoidMemory,
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
) {
    draw_boids(memory, frame_buffer, zbuffer, width, height, camera);
    draw_bounds(frame_buffer, zbuffer, width, height, camera);
}

#[derive(Default)]
struct Boid {
    translation: Vec3,
    velocity: Vec3,
}

fn apply_velocity(memory: &mut BoidMemory, dt: f32) {
    for boid in memory.boids.iter_mut() {
        if boid.velocity.length_squared() > MAX_SPEED_SQ {
            boid.velocity = boid.velocity.normalize_or_zero() * MAX_SPEED;
        } else if boid.velocity.length_squared() < MIN_SPEED_SQ {
            boid.velocity = boid.velocity.normalize_or_zero() * MIN_SPEED;
        }
        boid.translation += boid.velocity * dt;
    }
}

fn avoid_bounds(memory: &mut BoidMemory) {
    for boid in memory.boids.iter_mut() {
        if boid.translation.x < -BOUNDS + memory.margin {
            boid.velocity.x += memory.turn_factor;
        }
        if boid.translation.x > BOUNDS - memory.margin {
            boid.velocity.x -= memory.turn_factor;
        }
        if boid.translation.y < -BOUNDS + memory.margin {
            boid.velocity.y += memory.turn_factor;
        }
        if boid.translation.y > BOUNDS - memory.margin {
            boid.velocity.y -= memory.turn_factor;
        }
        if boid.translation.z < -BOUNDS + memory.margin {
            boid.velocity.z += memory.turn_factor;
        }
        if boid.translation.z > BOUNDS - memory.margin {
            boid.velocity.z -= memory.turn_factor;
        }
    }
}

fn boid_forces(memory: &mut BoidMemory) {
    let mut velocity_changes = [Vec3::ZERO; BOID_COUNT];

    for i in 0..memory.boids.len() {
        let current_boid = &memory.boids[i];
        let mut separation = Vec3::ZERO;
        let mut cohesion_center = Vec3::ZERO;
        let mut alignment_avg = Vec3::ZERO;
        let mut cohesion_count = 0;
        let mut alignment_count = 0;

        for j in 0..memory.boids.len() {
            if i != j {
                let other_boid = &memory.boids[j];
                let distance_sq = current_boid
                    .translation
                    .distance_squared(other_boid.translation);

                if distance_sq <= memory.separation_radius_squared {
                    separation += current_boid.translation - other_boid.translation;
                }

                if distance_sq <= memory.view_radius_squared {
                    cohesion_center += other_boid.translation;
                    alignment_avg += other_boid.velocity;
                    cohesion_count += 1;
                    alignment_count += 1;
                }
            }
        }

        let mut total_change = separation * memory.separation_factor;
        if cohesion_count > 0 {
            cohesion_center /= cohesion_count as f32;
            total_change += (cohesion_center - current_boid.translation) * memory.cohesion_factor;
        }
        if alignment_count > 0 {
            alignment_avg /= alignment_count as f32;
            total_change += (alignment_avg - current_boid.velocity) * memory.alignment_factor;
        }
        velocity_changes[i] = total_change;
    }

    for i in 0..memory.boids.len() {
        memory.boids[i].velocity += velocity_changes[i];
    }
}

fn draw_boids(
    memory: &BoidMemory,
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
) {
    for boid in memory.boids.iter() {
        for face in memory.vertices.chunks(3) {
            let v1 = face[0] + boid.translation;
            let v2 = face[1] + boid.translation;
            let v3 = face[2] + boid.translation;

            if let Some((v1, v2, v3)) = crate::math::triangle_world_to_screen_space_clipped(
                width, height, camera, v1, v2, v3,
            ) {
                let r = (boid.translation.x.clamp(-BOUNDS, BOUNDS) / BOUNDS + 1.0) / 2.0;
                let g = (boid.translation.y.clamp(-BOUNDS, BOUNDS) / BOUNDS + 1.0) / 2.0;
                let b = (boid.translation.z.clamp(-BOUNDS, BOUNDS) / BOUNDS + 1.0) / 2.0;
                let color = LinearRgb::rgb(r, g, b).to_srgb();
                rast::rast_triangle_colored_checked(
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
}

fn draw_bounds(
    frame_buffer: &mut [Srgb],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    camera: &Camera,
) {
    let corners = cube(Vec3::new(BOUNDS, BOUNDS, BOUNDS));
    #[rustfmt::skip]
    let edges = [
        (0, 1), (1, 2), (2, 3),
        (3, 0), (4, 5), (5, 6),
        (6, 7), (7, 4), (0, 4),
        (1, 5), (2, 6), (3, 7),
    ];
    for (i1, i2) in edges.into_iter() {
        let v1 =
            crate::math::vertex_world_to_screen_space_clipped(width, height, camera, corners[i1]);
        let v2 =
            crate::math::vertex_world_to_screen_space_clipped(width, height, camera, corners[i2]);
        if let (Some(v1), Some(v2)) = (v1, v2) {
            rast::rast_line_checked(
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
                Srgb::rgb(0, 255, 0),
            );
        }
    }
}

fn cube(size: Vec3) -> [Vec3; 8] {
    let minx = -size.x;
    let miny = -size.y;
    let minz = -size.z;
    let maxx = size.z;
    let maxy = size.y;
    let maxz = size.z;
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
}

fn boid_vertices() -> [Vec3; 36] {
    let corners = cube(Vec3::new(1.0, 1.0, 1.0));
    [
        corners[0], corners[1], corners[2], corners[0], corners[2], corners[3], corners[5],
        corners[4], corners[7], corners[5], corners[7], corners[6], corners[4], corners[0],
        corners[3], corners[4], corners[3], corners[7], corners[1], corners[5], corners[6],
        corners[1], corners[6], corners[2], corners[4], corners[5], corners[1], corners[4],
        corners[1], corners[0], corners[3], corners[2], corners[6], corners[3], corners[6],
        corners[7],
    ]
}
