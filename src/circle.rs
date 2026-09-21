//! Cercle midpoint circle algorithm, integer-only.

use crate::Graphics;
use embedded_hal_async::i2c::I2c;

/// Trace le **contour** d'un cercle.
///
/// **Algorithme :** midpoint circle integer-only.
/// Exploite la symétrie 8-octants : chaque itération dessine 8 pixels
/// symétriques, ce qui minimise le nombre d'appels à `pixel()`.
///
/// # Paramètres
///
/// - `(cx, cy)` : centre
/// - `r` : rayon en pixels
///
/// # Exemple
///
/// ```rust,no_run
/// circle(&mut gfx, 64, 32, 20, true);
/// ```
pub fn circle<I: I2c>(gfx: &mut Graphics<'_, I>, cx: i32, cy: i32, r: i32, on: bool) {
    if r <= 0 {
        gfx.pixel(cx, cy, on);
        return;
    }
    let mut x = r;
    let mut y = 0;
    let mut err = 0;

    while x >= y {
        gfx.pixel(cx + x, cy + y, on);
        gfx.pixel(cx + y, cy + x, on);
        gfx.pixel(cx - y, cy + x, on);
        gfx.pixel(cx - x, cy + y, on);
        gfx.pixel(cx - x, cy - y, on);
        gfx.pixel(cx - y, cy - x, on);
        gfx.pixel(cx + y, cy - x, on);
        gfx.pixel(cx + x, cy - y, on);

        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err += 2 * (y - x) + 1;
        }
    }
}

/// **Remplit** un cercle (disque plein).
///
/// Utilise le même algorithme midpoint, mais dessine des lignes
/// horizontales entre les points symétriques à chaque rangée.
/// Beaucoup plus rapide que d'appeler `circle()` en spirale.
///
/// # Exemple
///
/// ```rust,no_run
/// fill_circle(&mut gfx, 64, 32, 15, true);
/// ```
pub fn fill_circle<I: I2c>(gfx: &mut Graphics<'_, I>, cx: i32, cy: i32, r: i32, on: bool) {
    if r <= 0 {
        gfx.pixel(cx, cy, on);
        return;
    }
    let mut x = r;
    let mut y = 0;
    let mut err = 0;

    while x >= y {
        // Lignes horizontales symétriques (haut/bas, gauche/droite)
        for px in (cx - x)..=(cx + x) {
            gfx.pixel(px, cy + y, on);
            gfx.pixel(px, cy - y, on);
        }
        for px in (cx - y)..=(cx + y) {
            gfx.pixel(px, cy + x, on);
            gfx.pixel(px, cy - x, on);
        }

        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err += 2 * (y - x) + 1;
        }
    }
}