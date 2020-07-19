use crate::Vector2D;

#[test]
fn dot() {
    let v1 = Vector2D::new(10.0, 5.0);
    let v2 = Vector2D::new(1.5, 2.0);

    let result = Vector2D::dot(v1, v2);

    assert_eq!(25.0, result);
}

#[test]
fn from_vec2d() {
    let iv = Vector2D::new(10, 5);
    let fv = Vector2D::from_vec2d(iv);
    assert_eq!(Vector2D::new(10.0, 5.0), fv);
}

#[test]
fn into_vec2d() {
    let iv = Vector2D::new(10, 5);
    let fv = iv.into_vec2d();
    assert_eq!(Vector2D::new(10.0, 5.0), fv);
}

#[test]
fn from_tuple() {
    let ituple = (10, 5);
    let fv = ituple.into();
    assert_eq!(Vector2D::new(10.0, 5.0), fv);
}

#[test]
fn from_array() {
    let arr = [10, 5];
    let fv = arr.into();
    assert_eq!(Vector2D::new(10.0, 5.0), fv);
}

#[test]
fn length_squared() {
    let v = Vector2D::new(10, 5);
    let r = v.length_squared();
    assert_eq!(125, r);
}

#[test]
fn length_f32() {
    let v: Vector2D<f32> = Vector2D::new(3.0, 4.0);
    let r: f32 = v.length();
    assert_eq!(5.0, r);
}

#[test]
fn length_f64() {
    let v: Vector2D<f64> = Vector2D::new(3.0, 4.0);
    let r: f64 = v.length();
    assert_eq!(5.0, r);
}

#[test]
fn angle_f32() {
    let v: Vector2D<f32> = Vector2D::new(2.0, 2.0);
    let r: f32 = v.angle();
    assert_eq!(std::f32::consts::PI / 4.0, r);
}

#[test]
fn angle_f64() {
    let v: Vector2D<f64> = Vector2D::new(2.0, 2.0);
    let r: f64 = v.angle();
    assert_eq!(std::f64::consts::PI / 4.0, r);
}

#[test]
fn add() {
    let v1 = Vector2D::new(10.0, 5.0);
    let v2 = Vector2D::new(1.5, 2.0);

    let result = v1 + v2;

    assert_eq!(Vector2D::new(11.5, 7.0), result);
}

#[test]
fn add_assign() {
    let mut v1 = Vector2D::new(10.0, 5.0);
    let v2 = Vector2D::new(1.5, 2.0);

    v1 += v2;

    assert_eq!(Vector2D::new(11.5, 7.0), v1);
}
#[test]
fn sub() {
    let v1 = Vector2D::new(10.0, 5.0);
    let v2 = Vector2D::new(1.5, 2.0);

    let result = v1 - v2;

    assert_eq!(Vector2D::new(8.5, 3.0), result);
}

#[test]
fn sub_assign() {
    let mut v1 = Vector2D::new(10.0, 5.0);
    let v2 = Vector2D::new(1.5, 2.0);

    v1 -= v2;

    assert_eq!(Vector2D::new(8.5, 3.0), v1);
}

#[test]
fn mul() {
    let v = Vector2D::new(10.0, 5.0);
    let f = 2.0;

    let result = v * f;

    assert_eq!(Vector2D::new(20.0, 10.0), result);
}

#[test]
fn mul_assign() {
    let mut v = Vector2D::new(10.0, 5.0);
    let f = 2.0;

    v *= f;

    assert_eq!(Vector2D::new(20.0, 10.0), v);
}

#[test]
fn div() {
    let v = Vector2D::new(10.0, 5.0);
    let f = 2.0;

    let result = v / f;

    assert_eq!(Vector2D::new(5.0, 2.5), result);
}

#[test]
fn div_assign() {
    let mut v = Vector2D::new(10.0, 5.0);
    let f = 2.0;

    v /= f;

    assert_eq!(Vector2D::new(5.0, 2.5), v);
}

#[test]
fn f64_as_i32() {
    let fv: Vector2D<f64> = Vector2D::new(10.5, 11.2);
    let iv = fv.as_i32s();
    assert_eq!(Vector2D::new(10, 11), iv);
}

#[test]
fn f32_as_u32() {
    let fv: Vector2D<f32> = Vector2D::new(10.5, 11.2);
    let uv = fv.as_u32s();
    assert_eq!(Vector2D::new(10, 11), uv);
}

#[test]
fn f32_as_u32_bounded() {
    let fv: Vector2D<f32> = Vector2D::new(-10.5, -11.2);
    let uv = fv.as_u32s();
    assert_eq!(Vector2D::new(0, 0), uv);
}

#[test]
fn lerp() {
    let start = Vector2D::new(5.0, 10.0);
    let end = Vector2D::new(10.0, 11.5);

    let result = Vector2D::lerp(start, end, 0.5);

    assert_eq!(Vector2D::new(7.5, 10.75), result);
}
