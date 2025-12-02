use crate::math::vec::Vec4;

pub mod simd {
    use std::arch::x86_64::{__m128, _mm_add_ps, _mm_mul_ps, _mm_shuffle_ps};

    #[repr(C)]
    union UnionCast {
        a: [f32; 4],
        v: std::mem::ManuallyDrop<Vec4>,
    }

    /// [x, y, z, w] packed into 128-bit SIMD register.
    pub struct Vec4(__m128);

    impl Vec4 {
        pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
            unsafe { Vec4(UnionCast { a: [x, y, z, w] }.v.0) }
        }
    }

    /// Column Major 4x4 Matrix.
    #[repr(C)]
    pub struct Mat4 {
        pub x_axis: Vec4,
        pub y_axis: Vec4,
        pub z_axis: Vec4,
        pub w_axis: Vec4,
    }

    /// │ a b c d │ │x│   │ x'│
    /// │ e f g h │ │y│ = │ y'│
    /// │ i j k l │ │z│   │ z'│
    /// │ m n o p │ │w│   │ w'│
    impl Mat4 {
        #[inline]
        pub fn mult_vec4(&self, v: Vec4) -> Vec4 {
            unsafe {
                let xxxx = _mm_shuffle_ps(v.0, v.0, 0b00_00_00_00);
                let yyyy = _mm_shuffle_ps(v.0, v.0, 0b01_01_01_01);
                let zzzz = _mm_shuffle_ps(v.0, v.0, 0b10_10_10_10);
                let wwww = _mm_shuffle_ps(v.0, v.0, 0b11_11_11_11);

                let x_column = _mm_mul_ps(self.x_axis.0, xxxx);
                let y_column = _mm_mul_ps(self.y_axis.0, yyyy);
                let z_column = _mm_mul_ps(self.z_axis.0, zzzz);
                let w_column = _mm_mul_ps(self.w_axis.0, wwww);

                Vec4(_mm_add_ps(
                    x_column,
                    _mm_add_ps(y_column, _mm_add_ps(z_column, w_column)),
                ))
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub r1: Vec4,
    pub r2: Vec4,
    pub r3: Vec4,
    pub r4: Vec4,
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        r1: Vec4::new(1.0, 0.0, 0.0, 0.0),
        r2: Vec4::new(0.0, 1.0, 0.0, 0.0),
        r3: Vec4::new(0.0, 0.0, 1.0, 0.0),
        r4: Vec4::new(0.0, 0.0, 0.0, 1.0),
    };

    #[inline]
    pub fn mult_vec4(&self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.r1.dot(rhs),
            self.r2.dot(rhs),
            self.r3.dot(rhs),
            self.r4.dot(rhs),
        )
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            r1: Vec4::new(self.r1.x, self.r2.x, self.r3.x, self.r4.x),
            r2: Vec4::new(self.r1.y, self.r2.y, self.r3.y, self.r4.y),
            r3: Vec4::new(self.r1.z, self.r2.z, self.r3.z, self.r4.z),
            r4: Vec4::new(self.r1.w, self.r2.w, self.r3.w, self.r4.w),
        }
    }

    #[inline]
    pub fn mult_mat4(&self, rhs: &Self) -> Self {
        let rhs = rhs.transpose();

        Self {
            r1: Vec4::new(
                self.r1.dot(rhs.r1),
                self.r1.dot(rhs.r2),
                self.r1.dot(rhs.r3),
                self.r1.dot(rhs.r4),
            ),
            r2: Vec4::new(
                self.r2.dot(rhs.r1),
                self.r2.dot(rhs.r2),
                self.r2.dot(rhs.r3),
                self.r2.dot(rhs.r4),
            ),
            r3: Vec4::new(
                self.r3.dot(rhs.r1),
                self.r3.dot(rhs.r2),
                self.r3.dot(rhs.r3),
                self.r3.dot(rhs.r4),
            ),
            r4: Vec4::new(
                self.r4.dot(rhs.r1),
                self.r4.dot(rhs.r2),
                self.r4.dot(rhs.r3),
                self.r4.dot(rhs.r4),
            ),
        }
    }
}
