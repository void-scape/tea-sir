use crate::math::mat::Mat4;
use crate::math::vec::Vec4;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat(pub Vec4);

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl core::ops::Deref for Quat {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Quat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Implementation based on:
// https://www.ljll.fr/~frey/papers/scientific%20visualisation/Shoemake%20K.,%20Quaternions.pdf
impl Quat {
    pub const IDENTITY: Self = Self(Vec4::new(0.0, 0.0, 0.0, 1.0));

    #[inline]
    #[must_use]
    pub fn from_rotation_x(angle: f32) -> Self {
        let (s, c) = libm::sincosf(angle * 0.5);
        Self(Vec4::new(s, 0.0, 0.0, c))
    }

    #[inline]
    #[must_use]
    pub fn from_rotation_y(angle: f32) -> Self {
        let (s, c) = libm::sincosf(angle * 0.5);
        Self(Vec4::new(0.0, s, 0.0, c))
    }

    #[inline]
    #[must_use]
    pub fn from_rotation_z(angle: f32) -> Self {
        let (s, c) = libm::sincosf(angle * 0.5);
        Self(Vec4::new(0.0, 0.0, s, c))
    }

    #[must_use]
    pub fn mul_quat(self, rhs: Self) -> Self {
        Self(Vec4::new(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        ))
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn compute_mat4(self) -> Mat4 {
        let nq = self.length_squared();
        let s = if nq > 0.0 { 2.0 / nq } else { 0.0 };

        let xs = self.x * s;
        let ys = self.y * s;
        let zs = self.z * s;
        let wx = self.w * xs;
        let wy = self.w * ys;
        let wz = self.w * zs;
        let xx = self.x * xs;
        let xy = self.x * ys;
        let xz = self.x * zs;
        let yy = self.y * ys;
        let yz = self.y * zs;
        let zz = self.z * zs;

        Mat4 {
            r1: Vec4::new(1.0 - (yy + zz), xy - wz,         xz + wy,         0.0),
            r2: Vec4::new(xy + wz,         1.0 - (xx + zz), yz - wx,         0.0),
            r3: Vec4::new(xz - wy,         yz + wx,         1.0 - (xx + yy), 0.0),
            r4: Vec4::new(0.0,             0.0,             0.0,             1.0),
        }
    }
}
