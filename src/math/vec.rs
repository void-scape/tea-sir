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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl_math_ops!(Vec4, f32, x, y, z, w);

impl Vec4 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self::new(v, v, v, v)
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
    pub const fn w(w: f32) -> Self {
        Self { w, ..Self::ZERO }
    }

    #[inline]
    #[must_use]
    pub const fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w)
    }

    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        libm::sqrtf(self.length_squared())
    }

    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f32 {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z) + (self.w * self.w)
    }

    #[inline]
    #[must_use]
    pub const fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    #[inline]
    #[must_use]
    pub const fn reduce(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

macro_rules! vec3 {
    ($ident:ident, $prim:ident, $vec2:ident, $sqrt:ident, $sincos:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq)]
        pub struct $ident {
            pub x: $prim,
            pub y: $prim,
            pub z: $prim,
        }
        impl_math_ops!($ident, $prim, x, y, z);

        impl $ident {
            pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

            #[inline]
            pub const fn new(x: $prim, y: $prim, z: $prim) -> Self {
                Self { x, y, z }
            }

            #[inline]
            pub const fn splat(v: $prim) -> Self {
                Self::new(v, v, v)
            }

            #[inline]
            pub const fn x(x: $prim) -> Self {
                Self { x, ..Self::ZERO }
            }

            #[inline]
            pub const fn y(y: $prim) -> Self {
                Self { y, ..Self::ZERO }
            }

            #[inline]
            pub const fn z(z: $prim) -> Self {
                Self { z, ..Self::ZERO }
            }

            #[inline]
            #[must_use]
            pub const fn cross(self, rhs: Self) -> Self {
                Self {
                    x: self.y * rhs.z - rhs.y * self.z,
                    y: self.z * rhs.x - rhs.z * self.x,
                    z: self.x * rhs.y - rhs.x * self.y,
                }
            }

            #[inline]
            #[must_use]
            pub fn dot(self, rhs: Self) -> $prim {
                (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
            }

            #[inline]
            #[must_use]
            pub fn length(self) -> $prim {
                libm::$sqrt(self.length_squared())
            }

            #[inline]
            #[must_use]
            pub fn length_squared(self) -> $prim {
                (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
            }

            #[inline]
            #[must_use]
            pub fn distance(self, rhs: Self) -> $prim {
                (self - rhs).length()
            }

            #[inline]
            #[must_use]
            pub fn distance_squared(self, rhs: Self) -> $prim {
                (self - rhs).length_squared()
            }

            #[inline]
            #[must_use]
            pub fn normalize(self) -> Self {
                let length = self.length();
                assert!(
                    length != 0.0,
                    "tried to call `{}::normalize` with a length of 0.0",
                    stringify!($ident),
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
            pub fn element_sum(self) -> $prim {
                self.x + self.y + self.z
            }

            #[inline]
            #[must_use]
            pub fn rotate_x(self, angle: $prim) -> Self {
                let (sin, cos) = libm::$sincos(angle);
                Self {
                    x: self.x,
                    y: self.y * cos - self.z * sin,
                    z: self.y * sin + self.z * cos,
                }
            }

            #[inline]
            #[must_use]
            pub fn rotate_y(self, angle: $prim) -> Self {
                let (sin, cos) = libm::$sincos(angle);
                Self {
                    x: self.x * cos + self.z * sin,
                    y: self.y,
                    z: -self.x * sin + self.z * cos,
                }
            }

            #[inline]
            #[must_use]
            pub fn rotate_z(self, angle: $prim) -> Self {
                let (sin, cos) = libm::$sincos(angle);
                Self {
                    x: self.x * cos - self.y * sin,
                    y: self.x * sin + self.y * cos,
                    z: self.z,
                }
            }

            #[inline]
            #[must_use]
            pub const fn to_array(self) -> [$prim; 3] {
                [self.x, self.y, self.z]
            }

            #[inline]
            #[must_use]
            pub const fn reduce(self) -> $vec2 {
                $vec2 {
                    x: self.x,
                    y: self.y,
                }
            }
        }

        impl core::ops::Neg for $ident {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    x: -self.x,
                    y: -self.y,
                    z: -self.z,
                }
            }
        }
    };
}

macro_rules! vec2 {
    ($ident:ident, $prim:ident, $vec3:ident, $sqrt:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq)]
        pub struct $ident {
            pub x: $prim,
            pub y: $prim,
        }
        impl_math_ops!($ident, $prim, x, y);

        impl $ident {
            pub const ZERO: Self = Self::new(0.0, 0.0);

            #[inline]
            pub const fn new(x: $prim, y: $prim) -> Self {
                Self { x, y }
            }

            #[inline]
            pub const fn splat(v: $prim) -> Self {
                Self::new(v, v)
            }

            #[inline]
            #[must_use]
            pub const fn extend(self, z: $prim) -> $vec3 {
                $vec3 {
                    x: self.x,
                    y: self.y,
                    z,
                }
            }

            #[inline]
            pub const fn x(x: $prim) -> Self {
                Self { x, ..Self::ZERO }
            }

            #[inline]
            pub const fn y(y: $prim) -> Self {
                Self { y, ..Self::ZERO }
            }

            #[inline]
            #[must_use]
            pub const fn cross(self, rhs: Self) -> $prim {
                (self.x * rhs.y) - (self.y * rhs.x)
            }

            #[inline]
            #[must_use]
            pub fn dot(self, rhs: Self) -> $prim {
                (self.x * rhs.x) + (self.y * rhs.y)
            }

            #[inline]
            #[must_use]
            pub fn length(self) -> $prim {
                libm::$sqrt(self.length_squared())
            }

            #[inline]
            #[must_use]
            pub fn length_squared(self) -> $prim {
                (self.x * self.x) + (self.y * self.y)
            }

            #[inline]
            #[must_use]
            pub fn normalize(self) -> Self {
                let length = self.length();
                assert!(
                    length != 0.0,
                    "tried to call `{}::normalize` with a length of 0.0",
                    stringify!($ident),
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
            pub fn element_sum(self) -> $prim {
                self.x + self.y
            }

            #[inline]
            #[must_use]
            pub fn to_array(self) -> [$prim; 2] {
                [self.x, self.y]
            }
        }

        impl core::ops::Neg for $ident {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    x: -self.x,
                    y: -self.y,
                }
            }
        }
    };
}

vec3!(Vec3, f32, Vec2, sqrtf, sincosf);
impl Vec3 {
    #[inline]
    #[must_use]
    pub const fn extend(self, w: f32) -> Vec4 {
        Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w,
        }
    }

    #[inline]
    #[must_use]
    pub fn to_dvec3(self) -> DVec3 {
        DVec3 {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }
}
vec3!(DVec3, f64, DVec2, sqrt, sincos);
impl DVec3 {
    #[inline]
    #[must_use]
    pub fn to_vec3(self) -> Vec3 {
        Vec3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }
}

vec2!(Vec2, f32, Vec3, sqrtf);
impl Vec2 {
    #[inline]
    #[must_use]
    pub fn to_dvec2(self) -> DVec2 {
        DVec2 {
            x: self.x as f64,
            y: self.y as f64,
        }
    }
}
vec2!(DVec2, f64, DVec3, sqrt);
impl DVec2 {
    #[inline]
    #[must_use]
    pub fn to_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}
