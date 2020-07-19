//! # vector2d
//! A simple and convenient 2D vector library without excessive use of external
//! dependencies. If other vector crates are swiss-army knives, vector2d is a
//! spoon; safe, intuitive, and convenient. As an added bonus, you won't run
//! into any excursions with the law using this library thanks to the awfully
//! permissive Unlicense.
//!
//! The only type in this crate is [`Vector2D`], which is highly generic;
//! shifting functionality depending upon the traits implemented by its internal
//! components' types.
//!
//! [`Vector2D`]: struct.Vector2D.html
//!
//! # Example
//! ```
//! use vector2d::Vector2D;
//!
//! fn main() {
//!     // Vectors have fields X and Y, these can be of any type
//!     let v1: Vector2D<i32> = Vector2D { x: 10, y: 5 };
//!
//!     // Alternatively you can use new(..) to condense instantiation
//!     let v2: Vector2D<f64> = Vector2D::new(13.0, 11.5);
//!
//!     // There are two ways to cast between Vector2Ds, depending on the source
//!     // and target types.
//!     //
//!     // If the target type has a implementation of From<SourceType>, then you
//!     // can either use source.into_vec2d() or Vector2D::from_vec2d(source).
//!     assert_eq!(Vector2D::new(10.0, 5.0), v1.into_vec2d());
//!     assert_eq!(Vector2D::new(10.0, 5.0), Vector2D::from_vec2d(v1));
//!
//!     // If there is no From or Into implementation, then you're out of luck
//!     // unless you are using specific primitives, such as i32 and f64. In
//!     // this case you can use specialised functions, as shown below:
//!     assert_eq!(Vector2D::new(13, 11), v2.as_i32s());
//!
//!     // The full list of interoperable primitives is as follows:
//!     //   - i32, i64, isize
//!     //   - u32, u64, usize
//!     //   - f32, f64
//!
//!     // As primitives generally implement From/Into for lossless casts,
//!     // an as_Ts() function is not available for those types, and
//!     // from(..)/into() should be favoured.
//!     //
//!     // Casts between signed and unsigned primitives will perform bounds
//!     // checking, so casting the vector (-10.0, 2.0) to a Vector2D<u32> will
//!     // result in the vector (0, 2).
//!
//!     // For types with an Add and Mul implementation, the functions dot() and
//!     // length_squared() are available. For access to length(), normalise(),
//!     // or angle() however, you must be using either Vector2D<f32> or
//!     // Vector2D<f64>.
//!     let _v1_len_sq = v1.length_squared();
//!     let v2_len = v2.length();
//!     let v2_dir = v2.normalise();
//!
//!     // Assuming the operator traits are implemented for the types involved,
//!     // you can add and subtract Vector2Ds from one-another, as well as
//!     // multiply and divide them with scalar values.
//!     assert_eq!(v2, v2_dir * v2_len);
//!     assert_eq!(Vector2D::new(23.0, 16.5),  v2 + v1.into_vec2d()) ;
//!
//!     // If you feel the need to multiply or divide individual components of
//!     // vectors with the same type, you can use mul_components(...) or
//!     // div_components(...) provided that their types can be multiplied or
//!     // divided.
//!
//!     // For any Vector2D<T>, there is an implementation of
//!     // From<(T, T)> and From<[T; 2]>
//!     let v4: Vector2D<f64> = Vector2D::new(1.5, 2.3);
//!     assert_eq!(v4, (1.5, 2.3).into());
//!     assert_eq!(v4, [1.5, 2.3].into());
//!
//!     // Additionally, there is an Into<(T, T)> implementation for any types
//!     // that the vector components have their own Into implementations for
//!     assert_eq!((1.5, 2.3), v4.into());
//!
//!     // If you want the normal of a vector you can just call normal()
//!     let v5 = Vector2D::new(-10.0, -2.3);
//!     assert_eq!(Vector2D::new(2.3, -10.0), v5.normal());
//!
//!     // You can get a vector consisting of only the horizontal or vertical
//!     // component of a vector by calling horizontal() or vertical()
//!     // respectively
//!     let v6 = Vector2D::new(12.3, 83.2);
//!     assert_eq!(Vector2D::new(12.3, 0.0), v6.horizontal());
//!     assert_eq!(Vector2D::new(0.0, 83.2), v6.vertical());
//! }
//! ```

#[cfg(test)]
mod test;

use proc_vector2d::{fn_lower_bounded_as, fn_simple_as};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A 2D vector, containing an `x` and a `y` component. While many types can be
/// used for a `Vector2D`'s components, the traits they implement determine
/// what functions are available.
///
/// Provided that the components implement the necessary traits, `Vector2D`s
/// can be added to or subtracted from one-another, and they can be mulitplied
/// and divided by scalar values.
///
/// There are generally two options for converting between `Vector2D` types. If
/// the internal components' type has an implementation of `Into` that targets
/// the desired type, then [`into_vec2d()`] can be called from the source object,
/// or [`from_vec2d(..)`] can be called and the source object can be provided.
///
/// If no `Into` implementation exists, then the only option is to use one of the
/// flavours of casting with `as`. These are in the form `as_types()`, and are only
/// implemented for specific types of components. An example usage would look like
/// this:
/// ```
/// use vector2d::Vector2D;
/// let f64_vector: Vector2D<f64> = Vector2D::new(10.3, 11.1);
/// let i32_vector: Vector2D<i32> = f64_vector.as_i32s();
/// assert_eq!(Vector2D::new(10, 11), i32_vector);
/// ```
///
/// Implementations of `as_types()` are only available when an implementation of
/// [`into_vec2d()`] is unavailable. This is to seperate between the lossless casting
/// of primitives with `into()` and `from(..)`, and the lossy casting between
/// primitives of varying detail.
///
/// Casts from signed types to unsigned types have a small additional check that
/// ensures a lower bound of 0 on the signed value, to reduce the chances of
/// experiencing undefined behaviour. This means that a `Vector2D<f64>` with a
/// value of `(-10.3, 11.1)` would become `(0, 11)` when cast to a `Vector2D<u32>`
/// with [`as_u32s()`].
///
/// The current list of interoperable types that can be cast with the `as` family of
/// functions is as follows:
///   - `i32`
///   - `i64`,
///   - `isize`
///   - `u32`
///   - `u64`
///   - `usize`
///   - `f32`
///   - `f64`
///
/// [`into_vec2d()`]: struct.Vector2D.html#method.into_vec2d
/// [`from_vec2d(..)`]: struct.Vector2D.html#method.from_vec2d
/// [`as_u32s()`]: struct.Vector2D.html#method.as_u32s-1
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy + Clone> Vector2D<T> {
    /// Create a new `Vector2D` with the provided components.
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Convert a `Vector2` of type `U` to one of type `T`. Available only when
    /// type T has implemented `From<U>`.
    ///
    /// # Example
    /// ```
    /// use vector2d::Vector2D;
    /// let i32_vector: Vector2D<i32> = Vector2D::new(25, 8);
    /// let f64_vector: Vector2D<f64> = Vector2D::from_vec2d(i32_vector);
    /// assert_eq!(Vector2D::new(25.0, 8.0), f64_vector);
    /// ```
    pub fn from_vec2d<U: Into<T> + Copy + Clone>(src: Vector2D<U>) -> Vector2D<T> {
        Vector2D {
            x: src.x.into(),
            y: src.y.into(),
        }
    }

    /// Convert a `Vector2` of type `T` to one of type `U`. Available only when
    /// type T has implemented `Into<U>`.
    ///
    /// # Example
    /// ```
    /// use vector2d::Vector2D;
    /// let i32_vector: Vector2D<i32> = Vector2D::new(25, 8);
    /// let i32_vector: Vector2D<i32> = Vector2D::new(25, 8);
    /// let f64_vector: Vector2D<f64> = i32_vector.into_vec2d();
    /// assert_eq!(Vector2D::new(25.0, 8.0), f64_vector);
    /// ```
    pub fn into_vec2d<U: From<T>>(self) -> Vector2D<U> {
        Vector2D {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

impl<T: Default> Vector2D<T> {
    /// Returns a vector with only the horizontal component of the current one
    ///
    /// # Example
    /// ```
    /// use vector2d::Vector2D;
    /// let v = Vector2D::new(10, 20);
    /// assert_eq!(Vector2D::new(10, 0), v.horizontal());
    /// ```
    pub fn horizontal(self) -> Self {
        Self {
            x: self.x,
            y: Default::default(),
        }
    }

    /// Returns a vector with only the vertical component of the current one
    ///
    /// # Example
    /// ```
    /// use vector2d::Vector2D;
    /// let v = Vector2D::new(10, 20);
    /// assert_eq!(Vector2D::new(0, 20), v.vertical());
    pub fn vertical(self) -> Self {
        Self {
            x: Default::default(),
            y: self.y,
        }
    }
}

impl<T> Vector2D<T>
where
    T: Mul<T, Output = T> + Copy + Clone,
{
    /// Returns a new vector with components equal to each of the current vector's
    /// components multiplied by the corresponding component of the provided vector
    ///
    /// # Example
    /// ```
    /// use vector2d::Vector2D;
    /// let v1 = Vector2D::new(11.0, -2.5);
    /// let v2 = Vector2D::new(0.5, -2.0);
    /// assert_eq!(Vector2D::new(5.5, 5.0), v1.mul_components(v2));
    /// ```
    pub fn mul_components(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T> Vector2D<T>
where
    T: Div<T, Output = T> + Copy + Clone,
{
    /// Returns a new vector with components equal to each of the current vector's
    /// components divided by the corresponding component of the provided vector
    ///
    /// # Example
    /// ```
    /// use vector2d::Vector2D;
    /// let v1 = Vector2D::new(11.0, -2.5);
    /// let v2 = Vector2D::new(0.5, -2.0);
    /// assert_eq!(Vector2D::new(22.0, 1.25), v1.div_components(v2));
    /// ```
    pub fn div_components(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl<T, U> Neg for Vector2D<T>
where
    T: Neg<Output = U> + Copy + Clone,
{
    type Output = Vector2D<U>;
    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> Vector2D<T>
where
    T: Neg<Output = T> + Copy + Clone,
{
    /// Returns a vector perpendicular to the current one.
    ///
    /// # Example
    /// ```
    /// use vector2d::Vector2D;
    /// let v = Vector2D::new(21.3, -98.1);
    /// assert_eq!(Vector2D::new(98.1, 21.3), v.normal());
    /// ```
    pub fn normal(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

impl<T, U, V> Vector2D<T>
where
    T: Mul<T, Output = U> + Copy + Clone,
    U: Add<U, Output = V> + Copy + Clone,
{
    /// Get the scalar/dot product of the two `Vector2D`.
    pub fn dot(v1: Self, v2: Self) -> V {
        v1.x * v2.x + v1.y * v2.y
    }

    /// Get the squared length of a `Vector2D`. This is more performant than using
    /// `length()` -- which is only available for `Vector2D<f32>` and `Vector2D<f64>`
    /// -- as it does not perform any square root operation.
    pub fn length_squared(self) -> V {
        self.x * self.x + self.y * self.y
    }
}

impl<T> Vector2D<T>
where
    T: Sub<T, Output = T> + Mul<T, Output = T> + Add<T, Output = T> + Copy + Clone,
{
    /// Linearly interpolates between two vectors
    pub fn lerp(start: Self, end: Self, progress: T) -> Self {
        start + ((end - start) * progress)
    }
}

// From/Into Implementations

impl<T, U> Into<(U, U)> for Vector2D<T>
where
    T: Into<U> + Copy + Clone,
{
    fn into(self) -> (U, U) {
        (self.x.into(), self.y.into())
    }
}

impl<T, U> From<(U, U)> for Vector2D<T>
where
    T: From<U>,
    U: Copy + Clone,
{
    fn from(src: (U, U)) -> Vector2D<T> {
        Vector2D {
            x: src.0.into(),
            y: src.1.into(),
        }
    }
}

impl<T, U> From<[U; 2]> for Vector2D<T>
where
    T: From<U>,
    U: Copy + Clone,
{
    fn from(src: [U; 2]) -> Vector2D<T> {
        Vector2D {
            x: src[0].into(),
            y: src[1].into(),
        }
    }
}

// Specific Primitive Implementations

impl Vector2D<f32> {
    /// Get the length of the vector. If possible, favour `length_squared()` over
    /// this function, as it is more performant.
    pub fn length(self) -> f32 {
        f32::sqrt(self.length_squared())
    }

    /// Get a new vector with the same direction as this vector, but with a length
    /// of 1.0. If the the length of the vector is 0, then the original vector is
    /// returned.
    pub fn normalise(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            self
        } else {
            self / len
        }
    }

    /// Get the vector's direction in radians.
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    fn_simple_as!(i32);
    fn_simple_as!(i64);
    fn_simple_as!(isize);
    fn_lower_bounded_as!(f32, u32, 0.0);
    fn_lower_bounded_as!(f32, u64, 0.0);
    fn_lower_bounded_as!(f32, usize, 0.0);
}

impl Vector2D<f64> {
    /// Get the length of the vector. If possible, favour `length_squared()` over
    /// this function, as it is more performant.
    pub fn length(self) -> f64 {
        f64::sqrt(self.length_squared())
    }

    /// Get a new vector with the same direction as this vector, but with a length
    /// of 1.0. If the the length of the vector is 0, then the original vector is
    /// returned.
    pub fn normalise(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            self
        } else {
            self / len
        }
    }

    /// Get the vector's direction in radians.
    pub fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }

    fn_simple_as!(i32);
    fn_simple_as!(i64);
    fn_simple_as!(isize);
    fn_simple_as!(f32);
    fn_lower_bounded_as!(f64, u32, 0.0);
    fn_lower_bounded_as!(f64, u64, 0.0);
    fn_lower_bounded_as!(f64, usize, 0.0);
}

impl Vector2D<i32> {
    fn_simple_as!(isize);
    fn_simple_as!(f32);
    fn_simple_as!(f64);
    fn_lower_bounded_as!(i32, u32, 0);
    fn_lower_bounded_as!(i32, u64, 0);
    fn_lower_bounded_as!(i32, usize, 0);
}

impl Vector2D<i64> {
    fn_simple_as!(i32);
    fn_simple_as!(isize);
    fn_simple_as!(f32);
    fn_simple_as!(f64);
    fn_lower_bounded_as!(i64, u32, 0);
    fn_lower_bounded_as!(i64, u64, 0);
    fn_lower_bounded_as!(i64, usize, 0);
}

impl Vector2D<isize> {
    fn_simple_as!(i32);
    fn_simple_as!(i64);
    fn_simple_as!(f32);
    fn_simple_as!(f64);
    fn_lower_bounded_as!(isize, u32, 0);
    fn_lower_bounded_as!(isize, u64, 0);
    fn_lower_bounded_as!(isize, usize, 0);
}

impl Vector2D<u32> {
    fn_simple_as!(i32);
    fn_simple_as!(i64);
    fn_simple_as!(isize);
    fn_simple_as!(f32);
    fn_simple_as!(f64);
    fn_simple_as!(usize);
}

impl Vector2D<u64> {
    fn_simple_as!(i32);
    fn_simple_as!(i64);
    fn_simple_as!(isize);
    fn_simple_as!(f32);
    fn_simple_as!(f64);
    fn_simple_as!(u32);
    fn_simple_as!(usize);
}

impl Vector2D<usize> {
    fn_simple_as!(i32);
    fn_simple_as!(i64);
    fn_simple_as!(isize);
    fn_simple_as!(f32);
    fn_simple_as!(f64);
    fn_simple_as!(u32);
    fn_simple_as!(u64);
}

// Ops Implementations

impl<T, O> Add<Vector2D<T>> for Vector2D<T>
where
    T: Add<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn add(self, rhs: Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T, O> Add<&Vector2D<T>> for &Vector2D<T>
where
    T: Add<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn add(self, rhs: &Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> AddAssign<Vector2D<T>> for Vector2D<T>
where
    T: Add<T, Output = T> + Copy + Clone,
{
    fn add_assign(&mut self, rhs: Vector2D<T>) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl<T, O> Sub<Vector2D<T>> for Vector2D<T>
where
    T: Sub<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn sub(self, rhs: Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T, O> Sub<&Vector2D<T>> for &Vector2D<T>
where
    T: Sub<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn sub(self, rhs: &Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> SubAssign<Vector2D<T>> for Vector2D<T>
where
    T: Sub<T, Output = T> + Copy + Clone,
{
    fn sub_assign(&mut self, rhs: Vector2D<T>) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
    }
}

impl<T, O> Mul<T> for Vector2D<T>
where
    T: Mul<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn mul(self, rhs: T) -> Self::Output {
        Vector2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T, O> Mul<T> for &Vector2D<T>
where
    T: Mul<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vector2D<T>
where
    T: Mul<T, Output = T> + Copy + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl<T, O> Div<T> for Vector2D<T>
where
    T: Div<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T, O> Div<T> for &Vector2D<T>
where
    T: Div<T, Output = O> + Copy + Clone,
{
    type Output = Vector2D<O>;
    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> DivAssign<T> for Vector2D<T>
where
    T: Div<T, Output = T> + Copy + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
    }
}
