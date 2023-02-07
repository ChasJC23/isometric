use std::ops;
use crate::num;

#[macro_export]
macro_rules! vect {
    ($x:expr, $y:expr) => {
        Vec2 { x: $x, y: $y }
    };
    ($x:expr, $y:expr, $z:expr) => {
        Vec3 { x: $x, y: $y, z: $z }
    };
}

#[macro_export]
macro_rules! vectp {
    ($x:pat, $y:pat) => {
        Vec2 { x: $x, y: $y }
    };
    ($x:pat, $y:pat, $z:pat) => {
        Vec3 { x: $x, y: $y, z: $z }
    };
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2<T: Copy> {
    pub x: T,
    pub y: T
}
impl<T> Vec2<T> where T: Copy + ops::Add<Output=T> + ops::Mul<Output=T> + num::Sqrt<Output=T> + ops::Div<Output=T> {
    pub fn normalise(self) -> Self {
        self / self.magnitude()
    }
}
impl<T> Vec2<T> where T: Copy + ops::Add<Output=T> + ops::Mul<Output=T> + num::Sqrt<Output=T> {
    pub fn magnitude(self) -> T {
        self.square_magnitude().sqrt()
    }
}
impl<T> Vec2<T> where T: Copy + ops::Add<Output=T> + ops::Mul<Output=T> {
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y
    }
    pub fn square_magnitude(self) -> T {
        self.dot(self)
    }
}
impl<T> Vec2<T> where T: Copy + ops::Sub<Output=T> + ops::Mul<Output=T> {
    pub fn cross(self, other: Self) -> T {
        self.x * other.y - self.y * other.x
    }
}
impl<T> Vec2<T> where T: Copy + ops::Add<Output=T> + ops::Sub<Output=T> + ops::Mul<Output=T> + num::Sin<Output=T> + num::Cos<Output=T> {
    pub fn rot(self, angle: T) -> Vec2<T> {
        vect![angle.cos() * self.x - angle.sin() * self.y, angle.sin() * self.x + angle.cos() * self.y]
    }
}
impl<T> ops::Add for Vec2<T> where T: Copy + ops::Add<Output=T> {
    type Output = Vec2<T>;

    fn add(self, rhs: Self) -> Self::Output {
        vect![self.x + rhs.x, self.y + rhs.y]
    }
}
impl<T> ops::Sub for Vec2<T> where T: Copy + ops::Sub<Output=T> {
    type Output = Vec2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        vect![self.x - rhs.x, self.y - rhs.y]
    }
}
impl<T> ops::Add<(T, T)> for Vec2<T> where T: Copy + ops::Add<Output=T> {
    type Output = Vec2<T>;

    fn add(self, rhs: (T, T)) -> Self::Output {
        vect![self.x + rhs.0, self.y + rhs.1]
    }
}
impl<T> ops::Sub<(T, T)> for Vec2<T> where T: Copy + ops::Sub<Output=T> {
    type Output = Vec2<T>;

    fn sub(self, rhs: (T, T)) -> Self::Output {
        vect![self.x - rhs.0, self.y - rhs.1]
    }
}
impl<T> ops::AddAssign for Vec2<T> where T: Copy + ops::Add<Output=T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<T> ops::AddAssign<(T, T)> for Vec2<T> where T: Copy + ops::Add<Output=T> {
    fn add_assign(&mut self, rhs: (T, T)) {
        *self = *self + rhs;
    }
}
impl<T> ops::SubAssign for Vec2<T> where T: Copy + ops::Sub<Output=T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl<T> ops::SubAssign<(T, T)> for Vec2<T> where T: Copy + ops::Sub<Output=T> {
    fn sub_assign(&mut self, rhs: (T, T)) {
        *self = *self - rhs;
    }
}
impl<T> ops::Mul for Vec2<T> where T: Copy + ops::Mul<Output=T> {
    type Output = Vec2<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        vect![self.x * rhs.x, self.y * rhs.y]
    }
}
impl<T> ops::Div for Vec2<T> where T: Copy + ops::Div<Output=T> {
    type Output = Vec2<T>;

    fn div(self, rhs: Self) -> Self::Output {
        vect![self.x / rhs.x, self.y / rhs.y]
    }
}
impl<T> ops::Rem for Vec2<T> where T: Copy + ops::Rem<Output=T> {
    type Output = Vec2<T>;

    fn rem(self, rhs: Self) -> Self::Output {
        vect![self.x % rhs.x, self.y % rhs.y]
    }
}
impl<T> ops::Mul<T> for Vec2<T> where T: Copy + ops::Mul<Output=T> {
    type Output = Vec2<T>;

    fn mul(self, rhs: T) -> Self::Output {
        vect![self.x * rhs, self.y * rhs]
    }
}
impl<T> ops::Div<T> for Vec2<T> where T: Copy + ops::Div<Output=T> {
    type Output = Vec2<T>;

    fn div(self, rhs: T) -> Self::Output {
        vect![self.x / rhs, self.y / rhs]
    }
}
impl<T> ops::Rem<T> for Vec2<T> where T: Copy + ops::Rem<Output=T> {
    type Output = Vec2<T>;

    fn rem(self, rhs: T) -> Self::Output {
        vect![self.x % rhs, self.y % rhs]
    }
}
impl<T> From<(T, T)> for Vec2<T> where T: Copy {
    fn from(tup: (T, T)) -> Self {
        vect![tup.0, tup.1]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3<T: Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
}
impl<T> Vec3<T> where T: Copy + ops::Add<Output=T> + ops::Mul<Output=T> + num::Sqrt<Output=T> + ops::Div<Output=T> {
    pub fn normalise(self) -> Self {
        self / self.magnitude()
    }
}
impl<T> Vec3<T> where T: Copy + ops::Add<Output=T> + ops::Mul<Output=T> + num::Sqrt<Output=T> {
    pub fn magnitude(self) -> T {
        self.square_magnitude().sqrt()
    }
}
impl<T> Vec3<T> where T: Copy + ops::Add<Output=T> + ops::Mul<Output=T> {
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn square_magnitude(self) -> T {
        self.dot(self)
    }
}
impl<T> Vec3<T> where T: Copy + ops::Sub<Output=T> + ops::Mul<Output=T> {
    pub fn cross(self, other: Self) -> Vec3<T> {
        vect![
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        ]
    }
}
impl<T> ops::Add for Vec3<T> where T: Copy + ops::Add<Output=T> {
    type Output = Vec3<T>;

    fn add(self, rhs: Self) -> Self::Output {
        vect![self.x + rhs.x, self.y + rhs.y, self.z + rhs.z]
    }
}
impl<T> ops::Sub for Vec3<T> where T: Copy + ops::Sub<Output=T> {
    type Output = Vec3<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        vect![self.x - rhs.x, self.y - rhs.y, self.z - rhs.z]
    }
}
impl<T> ops::Add<(T, T, T)> for Vec3<T> where T: Copy + ops::Add<Output=T> {
    type Output = Vec3<T>;

    fn add(self, rhs: (T, T, T)) -> Self::Output {
        vect![self.x + rhs.0, self.y + rhs.1, self.z + rhs.2]
    }
}
impl<T> ops::Sub<(T, T, T)> for Vec3<T> where T: Copy + ops::Sub<Output=T> {
    type Output = Vec3<T>;

    fn sub(self, rhs: (T, T, T)) -> Self::Output {
        vect![self.x - rhs.0, self.y - rhs.1, self.z - rhs.2]
    }
}
impl<T> ops::AddAssign for Vec3<T> where T: Copy + ops::Add<Output=T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<T> ops::AddAssign<(T, T, T)> for Vec3<T> where T: Copy + ops::Add<Output=T> {
    fn add_assign(&mut self, rhs: (T, T, T)) {
        *self = *self + rhs;
    }
}
impl<T> ops::SubAssign for Vec3<T> where T: Copy + ops::Sub<Output=T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl<T> ops::SubAssign<(T, T, T)> for Vec3<T> where T: Copy + ops::Sub<Output=T> {
    fn sub_assign(&mut self, rhs: (T, T, T)) {
        *self = *self - rhs;
    }
}
impl<T> ops::Mul for Vec3<T> where T: Copy + ops::Mul<Output=T> {
    type Output = Vec3<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        vect![self.x * rhs.x, self.y * rhs.y, self.z * rhs.z]
    }
}
impl<T> ops::Div for Vec3<T> where T: Copy + ops::Div<Output=T> {
    type Output = Vec3<T>;

    fn div(self, rhs: Self) -> Self::Output {
        vect![self.x / rhs.x, self.y / rhs.y, self.z / rhs.z]
    }
}
impl<T> ops::Rem for Vec3<T> where T: Copy + ops::Rem<Output=T> {
    type Output = Vec3<T>;

    fn rem(self, rhs: Self) -> Self::Output {
        vect![self.x % rhs.x, self.y % rhs.y, self.z % rhs.z]
    }
}
impl<T> ops::Mul<T> for Vec3<T> where T: Copy + ops::Mul<Output=T> {
    type Output = Vec3<T>;

    fn mul(self, rhs: T) -> Self::Output {
        vect![self.x * rhs, self.y * rhs, self.z * rhs]
    }
}
impl<T> ops::Div<T> for Vec3<T> where T: Copy + ops::Div<Output=T> {
    type Output = Vec3<T>;

    fn div(self, rhs: T) -> Self::Output {
        vect![self.x / rhs, self.y / rhs, self.z / rhs]
    }
}
impl<T> ops::Rem<T> for Vec3<T> where T: Copy + ops::Rem<Output=T> {
    type Output = Vec3<T>;

    fn rem(self, rhs: T) -> Self::Output {
        vect![self.x % rhs, self.y % rhs, self.z % rhs]
    }
}
impl<T> From<(T, T, T)> for Vec3<T> where T: Copy {
    fn from(tup: (T, T, T)) -> Self {
        vect![tup.0, tup.1, tup.2]
    }
}
