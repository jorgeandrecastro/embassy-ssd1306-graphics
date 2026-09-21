//! Courbe de Bézier quadratique  De Casteljau integer-only.

use crate::{line, Graphics};
use embedded_hal_async::i2c::I2c;

/// Trace une **courbe de Bézier quadratique** (3 points de contrôle).
///
/// **Algorithme :** De Casteljau integer-only avec subdivision fixe.
/// `steps` contrôle la finesse du tracé (16–32 suffisent pour 128×64).
///
/// Les interpolations sont faites en entiers avec précision ×1024
/// pour éviter tout flottant.
///
/// # Paramètres
///
/// - `(x0, y0)` : point de départ
/// - `(x1, y1)` : point de contrôle
/// - `(x2, y2)` : point d'arrivée
/// - `steps` : nombre de segments (recommandé : 16 à 32)
///
/// # Exemple
///
/// ```rust,no_run
/// bezier_quad(&mut gfx, 10, 50, 64, 5, 118, 50, 24, true); // arche
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
        // t = i / steps en virgule fixe ×1024
        let t  = (i * 1024) / steps;         // t  ∈ [0, 1024]
        let t1 = 1024 - t;                   // 1-t

        // B(t) = (1-t)²·P0 + 2(1-t)t·P1 + t²·P2  (tout ×1024²)
        let nx = (t1 * t1 * x0 + 2 * t1 * t * x1 + t * t * x2) / (1024 * 1024);
        let ny = (t1 * t1 * y0 + 2 * t1 * t * y1 + t * t * y2) / (1024 * 1024);

        line(gfx, px, py, nx, ny, on);
        px = nx;
        py = ny;
    }
}