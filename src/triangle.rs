//! Triangle  contour via 3 lignes, remplissage via scanline.

use crate::{line, Graphics};
use embedded_hal_async::i2c::I2c;

/// Trace le **contour** d'un triangle défini par trois sommets.
///
/// Implémenté comme trois appels à [`crate::line()`], aucune logique propre.
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

/// **Remplit** un triangle défini par trois sommets.
///
/// **Algorithme :** scanline  tri des sommets par Y, puis
/// interpolation linéaire integer-only des bords gauche/droit
/// à chaque rangée horizontale.
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
    // Tri des sommets par Y croissant (bubble sort sur 3 éléments)
    let (mut x0, mut x1, mut x2) = (x0, x1, x2);
    if y0 > y1 { core::mem::swap(&mut y0, &mut y1); core::mem::swap(&mut x0, &mut x1); }
    if y1 > y2 { core::mem::swap(&mut y1, &mut y2); core::mem::swap(&mut x1, &mut x2); }
    if y0 > y1 { core::mem::swap(&mut y0, &mut y1); core::mem::swap(&mut x0, &mut x1); }

    let total_h = y2 - y0;
    if total_h == 0 {
        // Triangle dégénéré — tracer une seule ligne
        let xmin = x0.min(x1).min(x2);
        let xmax = x0.max(x1).max(x2);
        for x in xmin..=xmax {
            gfx.pixel(x, y0, on);
        }
        return;
    }

    let upper_h = y1 - y0;
    let lower_h = y2 - y1;

    // Moitié supérieure : y0 → y1
    for y in y0..=y1 {
        let dy = y - y0;
        // Interpolation integer-only ×total_h pour éviter la division
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

    // Moitié inférieure : y1 → y2
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