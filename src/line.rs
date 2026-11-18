//! Line drawing with integer-only Bresenham.

use crate::Graphics;
use embedded_hal_async::i2c::I2c;

/// Draws a line between `(x0, y0)` and `(x1, y1)`.
///
/// **Algorithm:** integer-only Bresenham.
/// No floating-point division, no multiplication; safe on any MCU without an FPU.
///
/// # Exemple
///
/// ```rust,no_run
/// line(&mut gfx, 0, 0, 127, 63, true);  // draw the full diagonal
/// line(&mut gfx, 0, 0, 127, 63, false); // erase the diagonal
/// ```
pub fn line<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    on: bool,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        gfx.pixel(x0, y0, on);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}