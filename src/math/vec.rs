#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn x(x: f32) -> Self {
        Self { x, ..Self::ZERO }
    }

    #[inline]
    pub const fn y(y: f32) -> Self {
        Self { y, ..Self::ZERO }
    }

    #[inline]
    pub const fn z(z: f32) -> Self {
        Self { z, ..Self::ZERO }
    }

    #[inline]
    pub const fn to_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    #[inline]
    #[must_use]
    pub const fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - other.y * self.z,
            y: self.z * other.x - other.z * self.x,
            z: self.x * other.y - other.x * self.y,
        }
    }

    #[inline]
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }

    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        libm::sqrtf(self.length_squared())
    }

    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline]
    #[must_use]
    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).length_squared()
    }

    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        let length = self.length();
        assert!(
            length != 0.0,
            "tried to call `Vec3::normalize` with a length of 0.0"
        );
        Self {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    #[inline]
    #[must_use]
    pub fn normalize_or_zero(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::default()
        } else {
            Self {
                x: self.x / length,
                y: self.y / length,
                z: self.z / length,
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn element_sum(self) -> f32 {
        self.x + self.y + self.z
    }

    #[inline]
    #[must_use]
    pub fn rotate_x(self, angle: f32) -> Self {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);

        Self {
            x: self.x,
            y: self.y * cos - self.z * sin,
            z: self.y * sin + self.z * cos,
        }
    }

    #[inline]
    #[must_use]
    pub fn rotate_y(self, angle: f32) -> Self {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);

        Self {
            x: self.x * cos + self.z * sin,
            y: self.y,
            z: -self.x * sin + self.z * cos,
        }
    }

    #[inline]
    #[must_use]
    pub fn rotate_z(self, angle: f32) -> Self {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);

        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
            z: self.z,
        }
    }

    #[inline]
    #[must_use]
    pub fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl core::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    pub const fn extend(self, z: f32) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z,
        }
    }

    #[inline]
    #[must_use]
    pub const fn cross(self, other: Self) -> f32 {
        (self.x * other.y) - (self.y * other.x)
    }

    #[inline]
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        (self.x * other.x) + (self.y * other.y)
    }

    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        libm::sqrtf(self.length_squared())
    }

    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        let length = self.length();
        assert!(
            length != 0.0,
            "tried to call `Vec2::normalize` with a length of 0.0"
        );
        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }

    #[inline]
    #[must_use]
    pub fn normalize_or_zero(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::default()
        } else {
            Self {
                x: self.x / length,
                y: self.y / length,
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn element_sum(self) -> f32 {
        self.x + self.y
    }

    #[inline]
    #[must_use]
    pub fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl core::ops::Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct DVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl DVec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn x(x: f64) -> Self {
        Self { x, ..Self::ZERO }
    }

    #[inline]
    pub const fn y(y: f64) -> Self {
        Self { y, ..Self::ZERO }
    }

    #[inline]
    pub const fn z(z: f64) -> Self {
        Self { z, ..Self::ZERO }
    }

    #[inline]
    #[must_use]
    pub const fn to_vec3(self) -> Vec3 {
        Vec3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }

    #[inline]
    #[must_use]
    pub const fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - other.y * self.z,
            y: self.z * other.x - other.z * self.x,
            z: self.x * other.y - other.x * self.y,
        }
    }

    #[inline]
    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }

    #[inline]
    #[must_use]
    pub fn length(self) -> f64 {
        libm::sqrt(self.length_squared())
    }

    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    #[must_use]
    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }

    #[inline]
    #[must_use]
    pub fn distance_squared(self, other: Self) -> f64 {
        (self - other).length_squared()
    }

    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        let length = self.length();
        assert!(
            length != 0.0,
            "tried to call `DVec3::normalize` with a length of 0.0"
        );
        Self {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    #[inline]
    #[must_use]
    pub fn normalize_or_zero(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::default()
        } else {
            Self {
                x: self.x / length,
                y: self.y / length,
                z: self.z / length,
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn element_sum(self) -> f64 {
        self.x + self.y + self.z
    }

    #[inline]
    #[must_use]
    pub fn rotate_x(self, angle: f64) -> Self {
        let cos = libm::cos(angle);
        let sin = libm::sin(angle);

        Self {
            x: self.x,
            y: self.y * cos - self.z * sin,
            z: self.y * sin + self.z * cos,
        }
    }

    #[inline]
    #[must_use]
    pub fn rotate_y(self, angle: f64) -> Self {
        let cos = libm::cos(angle);
        let sin = libm::sin(angle);

        Self {
            x: self.x * cos + self.z * sin,
            y: self.y,
            z: -self.x * sin + self.z * cos,
        }
    }

    #[inline]
    #[must_use]
    pub fn rotate_z(self, angle: f64) -> Self {
        let cos = libm::cos(angle);
        let sin = libm::sin(angle);

        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
            z: self.z,
        }
    }

    #[inline]
    #[must_use]
    pub fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

impl core::ops::Neg for DVec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

macro_rules! impl_math_ops {
    ($ty:path, $prim:ident, $($field:ident),*) => {
        impl core::ops::Add for $ty {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field + rhs.$field,)*
                }
            }
        }

        impl core::ops::Sub for $ty {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field - rhs.$field,)*
                }
            }
        }

        impl core::ops::Mul for $ty {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field * rhs.$field,)*
                }
            }
        }

        impl core::ops::Div for $ty {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self {
                    $($field: self.$field / rhs.$field,)*
                }
            }
        }

        impl core::ops::AddAssign for $ty {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field += rhs.$field;)*
            }
        }

        impl core::ops::SubAssign for $ty {
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$field -= rhs.$field;)*
            }
        }

        impl core::ops::MulAssign for $ty {
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$field *= rhs.$field;)*
            }
        }

        impl core::ops::DivAssign for $ty {
            fn div_assign(&mut self, rhs: Self) {
                $(self.$field /= rhs.$field;)*
            }
        }

        impl core::ops::Add<$prim> for $ty {
            type Output = Self;

            fn add(self, rhs: $prim) -> Self::Output {
                Self {
                    $($field: self.$field + rhs,)*
                }
            }
        }

        impl core::ops::Sub<$prim> for $ty {
            type Output = Self;

            fn sub(self, rhs: $prim) -> Self::Output {
                Self {
                    $($field: self.$field - rhs,)*
                }
            }
        }

        impl core::ops::Mul<$prim> for $ty {
            type Output = Self;

            fn mul(self, rhs: $prim) -> Self::Output {
                Self {
                    $($field: self.$field * rhs,)*
                }
            }
        }

        impl core::ops::Div<$prim> for $ty {
            type Output = Self;

            fn div(self, rhs: $prim) -> Self::Output {
                Self {
                    $($field: self.$field / rhs,)*
                }
            }
        }

        impl core::ops::AddAssign<$prim> for $ty {
            fn add_assign(&mut self, rhs: $prim) {
                $(self.$field += rhs;)*
            }
        }

        impl core::ops::SubAssign<$prim> for $ty {
            fn sub_assign(&mut self, rhs: $prim) {
                $(self.$field -= rhs;)*
            }
        }

        impl core::ops::MulAssign<$prim> for $ty {
            fn mul_assign(&mut self, rhs: $prim) {
                $(self.$field *= rhs;)*
            }
        }

        impl core::ops::DivAssign<$prim> for $ty {
            fn div_assign(&mut self, rhs: $prim) {
                $(self.$field /= rhs;)*
            }
        }
    };
}

impl_math_ops!(Vec2, f32, x, y);
impl_math_ops!(Vec3, f32, x, y, z);
impl_math_ops!(DVec3, f64, x, y, z);
