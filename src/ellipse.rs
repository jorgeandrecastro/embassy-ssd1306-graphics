//! Ellipse  midpoint ellipse algorithm (Bresenham généralisé).

use crate::Graphics;
use embedded_hal_async::i2c::I2c;

/// Trace le **contour** d'une ellipse.
///
/// **Algorithme :** midpoint ellipse integer-only (Bresenham généralisé).
/// Deux phases : région 1 (pente < -1) puis région 2 (pente > -1).
///
/// # Paramètres
///
/// - `(cx, cy)` : centre
/// - `rx` : demi-axe horizontal
/// - `ry` : demi-axe vertical
///
/// # Exemple
///
/// ```rust,no_run
/// ellipse(&mut gfx, 64, 32, 40, 20, true); // ellipse large
/// ellipse(&mut gfx, 64, 32, 10, 10, true); // cercle (rx == ry)
/// ```
pub fn ellipse<I: I2c>(gfx: &mut Graphics<'_, I>, cx: i32, cy: i32, rx: i32, ry: i32, on: bool) {
    if rx <= 0 || ry <= 0 {
        gfx.pixel(cx, cy, on);
        return;
    }

    let rx2 = rx * rx;
    let ry2 = ry * ry;

    let mut x = 0i32;
    let mut y = ry;

    // Région 1
    let mut d1 = ry2 - rx2 * ry + rx2 / 4;
    let mut dx = 2 * ry2 * x;
    let mut dy = 2 * rx2 * y;

    while dx < dy {
        gfx.pixel(cx + x, cy + y, on);
        gfx.pixel(cx - x, cy + y, on);
        gfx.pixel(cx + x, cy - y, on);
        gfx.pixel(cx - x, cy - y, on);

        x += 1;
        dx += 2 * ry2;
        if d1 < 0 {
            d1 += dx + ry2;
        } else {
            y -= 1;
            dy -= 2 * rx2;
            d1 += dx - dy + ry2;
        }
    }

    // Région 2
    let mut d2 = ry2 * (x * x + x) + rx2 * (y * y - 2 * y + 1) - rx2 * ry2 + rx2;

    while y >= 0 {
        gfx.pixel(cx + x, cy + y, on);
        gfx.pixel(cx - x, cy + y, on);
        gfx.pixel(cx + x, cy - y, on);
        gfx.pixel(cx - x, cy - y, on);

        y -= 1;
        dy -= 2 * rx2;
        if d2 > 0 {
            d2 += rx2 - dy;
        } else {
            x += 1;
            dx += 2 * ry2;
            d2 += dx - dy + rx2;
        }
    }
}