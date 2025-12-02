use criterion::{Criterion, criterion_group, criterion_main};
use glam::{Mat4, Quat, Vec3, Vec4};
use std::hint::black_box;
use tea_sir::model::Model;

fn criterion_benchmark(c: &mut Criterion) {
    let dragon = include_str!("../assets/dragon.obj");
    let model = tea_sir::io::debug_obj_str(dragon, vec![]).unwrap();
    let model_matrix = Mat4::from_scale_rotation_translation(
        Vec3::ZERO,
        Quat::default(),
        glam::Vec3::new(0.0, -0.1, -1.0),
    );
    let projection_matrix = glam::Mat4::perspective_rh(
        90f32.to_radians(),
        tea_sir::MAX_WIDTH as f32 / tea_sir::MAX_HEIGHT as f32,
        0.1,
        1000.0,
    );
    let model_to_proj_matrix = projection_matrix.mul_mat4(&model_matrix);

    let manual_model_to_proj_matrix = unsafe {
        std::mem::transmute::<[f32; 16], tea_sir::math::Mat4>(
            model_to_proj_matrix.transpose().to_cols_array(),
        )
    };

    let simd_model_to_proj_matrix = unsafe {
        std::mem::transmute::<[f32; 16], tea_sir::math::simd::Mat4>(
            model_to_proj_matrix.to_cols_array(),
        )
    };

    c.bench_function("glam", |b| {
        b.iter(|| {
            glam(black_box(&model_to_proj_matrix), black_box(&model));
        })
    });
    c.bench_function("manual", |b| {
        b.iter(|| {
            naive(black_box(&manual_model_to_proj_matrix), black_box(&model));
        })
    });
    c.bench_function("simd", |b| {
        b.iter(|| {
            simd(black_box(&simd_model_to_proj_matrix), black_box(&model));
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(60);
    targets = criterion_benchmark
);
criterion_main!(benches);

#[inline(never)]
fn glam(model_to_proj_matrix: &Mat4, model: &Model) {
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

        keep(black_box(v1));
        keep(black_box(v2));
        keep(black_box(v3));
    }

    #[inline(never)]
    fn keep(v: Vec4) {
        unsafe {
            std::ptr::read_volatile(&v);
        }
    }
}

#[inline(never)]
fn naive(model_to_proj_matrix: &tea_sir::math::Mat4, model: &Model) {
    for face in model.faces.chunks(3) {
        let mv1 = model.verts[face[0]].extend(1.0);
        let mv2 = model.verts[face[1]].extend(1.0);
        let mv3 = model.verts[face[2]].extend(1.0);

        let v1 = model_to_proj_matrix.mult_vec4(mv1);
        let v2 = model_to_proj_matrix.mult_vec4(mv2);
        let v3 = model_to_proj_matrix.mult_vec4(mv3);

        keep(black_box(v1));
        keep(black_box(v2));
        keep(black_box(v3));
    }

    #[inline(never)]
    fn keep(v: tea_sir::math::Vec4) {
        unsafe {
            std::ptr::read_volatile(&v);
        }
    }
}

#[inline(never)]
fn simd(model_to_proj_matrix: &tea_sir::math::simd::Mat4, model: &Model) {
    for face in model.faces.chunks(3) {
        let mv1 = model.verts[face[0]].extend(1.0);
        let mv2 = model.verts[face[1]].extend(1.0);
        let mv3 = model.verts[face[2]].extend(1.0);

        let mv1 = tea_sir::math::simd::Vec4::new(mv1.x, mv1.y, mv1.z, mv1.w);
        let mv2 = tea_sir::math::simd::Vec4::new(mv2.x, mv2.y, mv2.z, mv2.w);
        let mv3 = tea_sir::math::simd::Vec4::new(mv3.x, mv3.y, mv3.z, mv3.w);

        let v1 = model_to_proj_matrix.mult_vec4(mv1);
        let v2 = model_to_proj_matrix.mult_vec4(mv2);
        let v3 = model_to_proj_matrix.mult_vec4(mv3);

        keep(black_box(v1));
        keep(black_box(v2));
        keep(black_box(v3));
    }

    #[inline(never)]
    fn keep(v: tea_sir::math::simd::Vec4) {
        unsafe {
            std::ptr::read_volatile(&v);
        }
    }
}
