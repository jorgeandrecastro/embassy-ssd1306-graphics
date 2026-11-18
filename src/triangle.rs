//! Triangle outline using 3 lines; filling using scanlines.

use crate::{line, Graphics};
use embedded_hal_async::i2c::I2c;

/// Draws the **outline** of a triangle defined by three vertices.
///
/// Implemented as three calls to [`crate::line()`], with no additional logic.
///
/// # Exemple
///
/// ```rust,no_run
/// triangle(&mut gfx, 64, 4, 20, 59, 108, 59, true);
/// ```
#[inline]
pub fn triangle<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    x2: i32, y2: i32,
    on: bool,
) {
    line(gfx, x0, y0, x1, y1, on);
    line(gfx, x1, y1, x2, y2, on);
    line(gfx, x2, y2, x0, y0, on);
}

/// **Fills** a triangle defined by three vertices.
///
/// **Algorithm:** scanline; sort the vertices by Y, then
/// linearly interpolate the left and right edges using integer arithmetic
/// for each horizontal row.
///
/// # Exemple
///
/// ```rust,no_run
/// fill_triangle(&mut gfx, 64, 4, 20, 59, 108, 59, true);
/// ```
pub fn fill_triangle<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    x0: i32, mut y0: i32,
    x1: i32, mut y1: i32,
    x2: i32, mut y2: i32,
    on: bool,
) {
    // Sort vertices by ascending Y (bubble sort on 3 elements)
    let (mut x0, mut x1, mut x2) = (x0, x1, x2);
    if y0 > y1 { core::mem::swap(&mut y0, &mut y1); core::mem::swap(&mut x0, &mut x1); }
    if y1 > y2 { core::mem::swap(&mut y1, &mut y2); core::mem::swap(&mut x1, &mut x2); }
    if y0 > y1 { core::mem::swap(&mut y0, &mut y1); core::mem::swap(&mut x0, &mut x1); }

    let total_h = y2 - y0;
    if total_h == 0 {
        // Degenerate triangle — draw a single line
        let xmin = x0.min(x1).min(x2);
        let xmax = x0.max(x1).max(x2);
        for x in xmin..=xmax {
            gfx.pixel(x, y0, on);
        }
        return;
    }

    let upper_h = y1 - y0;
    let lower_h = y2 - y1;

    // Upper half: y0 → y1
    for y in y0..=y1 {
        let dy = y - y0;
        // Integer-only interpolation scaled by total_h to avoid division
        let xa = x0 + (x2 - x0) * dy / total_h;
        let xb = if upper_h == 0 {
            x1
        } else {
            x0 + (x1 - x0) * dy / upper_h
        };
        let (xmin, xmax) = if xa < xb { (xa, xb) } else { (xb, xa) };
        for x in xmin..=xmax {
            gfx.pixel(x, y, on);
        }
    }

    // Lower half: y1 → y2
    for y in y1..=y2 {
        let dy = y - y0;
        let xa = x0 + (x2 - x0) * dy / total_h;
        let xb = if lower_h == 0 {
            x1
        } else {
            x1 + (x2 - x1) * (y - y1) / lower_h
        };
        let (xmin, xmax) = if xa < xb { (xa, xb) } else { (xb, xa) };
        for x in xmin..=xmax {
            gfx.pixel(x, y, on);
        }
    }
}