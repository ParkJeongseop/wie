//! The handset 3D package some SK-VM titles draw their world through, named `m`: a vector
//! (`V3`), a 3x4 affine transform (`A3`) and a renderer (`XO_World`). The arithmetic is answered
//! exactly — titles compose a camera, transform points and project them themselves (크레이지버스) —
//! while the renderer keeps no model and draws nothing, as `com.skt.m3d.Graphics3D` here.
//!
//! The fixed point is 4096 for one, and an angle is a 4096th of a circle: a title's own perspective
//! is `4096 * sin(fov / 2) / cos(fov / 2)` with a field of view of 512.

mod a3;
mod v3;
mod xo_world;

pub use self::{a3::A3, v3::V3, xo_world::XOWorld};

/// The fixed-point one, and the number of angle units in a full circle.
const ONE: i64 = 4096;
const TURN: f64 = 4096.0;

fn angle_to_radians(angle: i32) -> f64 {
    angle as f64 * 2.0 * core::f64::consts::PI / TURN
}

fn fixed(value: f64) -> i32 {
    libm::round(value * ONE as f64) as i32
}
