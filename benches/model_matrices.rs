use criterion::{Criterion, criterion_group, criterion_main};
use glam::{Mat4, Quat, Vec3};
use std::hint::black_box;

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

    let naive_cm_model_to_proj_matrix = unsafe {
        std::mem::transmute::<[f32; 16], tea_sir::math::cm::Mat4>(
            model_to_proj_matrix.to_cols_array(),
        )
    };

    let naive_rm_model_to_proj_matrix = unsafe {
        std::mem::transmute::<[f32; 16], tea_sir::math::Mat4>(
            model_to_proj_matrix.transpose().to_cols_array(),
        )
    };

    let simd_cm_model_to_proj_matrix = unsafe {
        std::mem::transmute::<[f32; 16], tea_sir::math::simd_cm::Mat4>(
            model_to_proj_matrix.to_cols_array(),
        )
    };

    let simd_rm_model_to_proj_matrix = unsafe {
        std::mem::transmute::<[f32; 16], tea_sir::math::simd_rm::Mat4>(
            model_to_proj_matrix.transpose().to_cols_array(),
        )
    };

    let mvs = unsafe { std::mem::transmute(model.verts.as_slice()) };
    c.bench_function("glam", |b| {
        b.iter(|| {
            glam(black_box(&model_to_proj_matrix), black_box(mvs));
        })
    });

    let mvs = unsafe { std::mem::transmute(model.verts.as_slice()) };
    c.bench_function("naive_cm", |b| {
        b.iter(|| {
            naive_cm(black_box(&naive_cm_model_to_proj_matrix), black_box(mvs));
        })
    });

    let mvs = unsafe { std::mem::transmute(model.verts.as_slice()) };
    c.bench_function("naive_rm", |b| {
        b.iter(|| {
            naive_rm(black_box(&naive_rm_model_to_proj_matrix), black_box(mvs));
        })
    });

    let mvs = unsafe { std::mem::transmute(model.verts.as_slice()) };
    c.bench_function("simd_cm", |b| {
        b.iter(|| {
            simd_cm(black_box(&simd_cm_model_to_proj_matrix), black_box(mvs));
        })
    });

    let mvs = unsafe { std::mem::transmute(model.verts.as_slice()) };
    c.bench_function("simd_rm", |b| {
        b.iter(|| {
            simd_rm(black_box(&simd_rm_model_to_proj_matrix), black_box(mvs));
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
fn glam(model_to_proj_matrix: &Mat4, mvs: &[glam::Vec4]) {
    for mv in mvs.iter() {
        let v1 = model_to_proj_matrix.mul_vec4(*mv);
        std::hint::black_box(&v1);
    }
}

#[inline(never)]
fn naive_cm(model_to_proj_matrix: &tea_sir::math::cm::Mat4, mvs: &[tea_sir::math::Vec4]) {
    for mv in mvs.iter() {
        let v1 = model_to_proj_matrix.mult_vec4(*mv);
        std::hint::black_box(&v1);
    }
}

#[inline(never)]
fn naive_rm(model_to_proj_matrix: &tea_sir::math::Mat4, mvs: &[tea_sir::math::Vec4]) {
    for mv in mvs.iter() {
        let v1 = model_to_proj_matrix.mult_vec4(*mv);
        std::hint::black_box(&v1);
    }
}

#[inline(never)]
fn simd_cm(
    model_to_proj_matrix: &tea_sir::math::simd_cm::Mat4,
    mvs: &[tea_sir::math::simd_cm::Vec4],
) {
    for mv in mvs.iter() {
        let v1 = model_to_proj_matrix.mult_vec4(*mv);
        std::hint::black_box(&v1);
    }
}

#[inline(never)]
fn simd_rm(
    model_to_proj_matrix: &tea_sir::math::simd_rm::Mat4,
    mvs: &[tea_sir::math::simd_rm::Vec4],
) {
    for mv in mvs.iter() {
        let v1 = model_to_proj_matrix.mult_vec4(*mv);
        std::hint::black_box(&v1);
    }
}
