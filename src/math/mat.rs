use crate::math::vec::Vec4;

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

    pub fn mult_vec4(&self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.r1.dot(rhs),
            self.r2.dot(rhs),
            self.r3.dot(rhs),
            self.r4.dot(rhs),
        )
    }

    pub fn transpose(&self) -> Self {
        Self {
            r1: Vec4::new(self.r1.x, self.r2.x, self.r3.x, self.r4.x),
            r2: Vec4::new(self.r1.y, self.r2.y, self.r3.y, self.r4.y),
            r3: Vec4::new(self.r1.z, self.r2.z, self.r3.z, self.r4.z),
            r4: Vec4::new(self.r1.w, self.r2.w, self.r3.w, self.r4.w),
        }
    }

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
