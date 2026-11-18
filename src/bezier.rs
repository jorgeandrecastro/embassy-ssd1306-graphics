//! Quadratic Bézier curve using integer-only De Casteljau.

use crate::{line, Graphics};
use embedded_hal_async::i2c::I2c;

/// Draws a **quadratic Bézier curve** (3 control points).
///
/// **Algorithm:** integer-only De Casteljau with a fixed number of subdivisions.
/// `steps` controls the curve's smoothness (16–32 is enough for 128×64).
///
/// Interpolations use integer arithmetic with ×1024 precision
/// to avoid floating-point operations.
///
/// # Parameters
///
/// - `(x0, y0)`: start point
/// - `(x1, y1)`: control point
/// - `(x2, y2)`: end point
/// - `steps`: number of segments (recommended: 16 to 32)
///
/// # Exemple
///
/// ```rust,no_run
/// bezier_quad(&mut gfx, 10, 50, 64, 5, 118, 50, 24, true); // arch
/// ```
pub fn bezier_quad<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    x2: i32, y2: i32,
    steps: i32,
    on: bool,
) {
    if steps <= 0 {
        return;
    }

    let mut px = x0;
    let mut py = y0;

    for i in 1..=steps {
        // t = i / steps in fixed-point ×1024
        let t  = (i * 1024) / steps;         // t  ∈ [0, 1024]
        let t1 = 1024 - t;                   // 1-t

        // B(t) = (1-t)²·P0 + 2(1-t)t·P1 + t²·P2 (all scaled by 1024²)
        let nx = (t1 * t1 * x0 + 2 * t1 * t * x1 + t * t * x2) / (1024 * 1024);
        let ny = (t1 * t1 * y0 + 2 * t1 * t * y1 + t * t * y2) / (1024 * 1024);

        line(gfx, px, py, nx, ny, on);
        px = nx;
        py = ny;
    }
}