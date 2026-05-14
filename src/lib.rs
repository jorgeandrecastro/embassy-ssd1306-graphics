#![no_std]
#![forbid(unsafe_code)]
//! # embassy-ssd1306-graphics
//!
//! Couche graphique 2D `no_std` pour écrans OLED SSD1306 (128×64),
//! construite au-dessus de [`embassy-ssd1306`].
//!
//! ## Rôle exact de ce crate
//!
//! Le driver [`embassy-ssd1306`] fournit déjà :
//! - `draw_pixel()`, `draw_hline()`, `draw_vline()`
//! - `draw_rect()`, `draw_filled_rect()`
//! - `draw_char()`, `draw_str()`, `draw_i16()`
//! - `draw_bitmap()`
//! - `clear()`, `fill()`, `flush()`
//!
//! Ce crate **ne duplique rien**. Il ajoute uniquement les primitives
//! que le driver ne propose pas :
//!
//! | Fonction          | Algorithme              |
//! |-------------------|-------------------------|
//! | [`line`]          | Bresenham integer-only  |
//! | [`circle`]        | Midpoint integer-only   |
//! | [`fill_circle`]   | Midpoint + hlines       |
//! | [`triangle`]      | 3 appels à `line()`     |
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────┐
//! │          Votre application           │
//! │  line() / circle() / triangle() …   │
//! │  oled.draw_str() / oled.draw_i16()  │  ← driver direct pour le texte
//! └──────────┬───────────────────────────┘
//!            │ &mut Graphics       │ &mut Ssd1306
//! ┌──────────▼───────────┐         │
//! │  Graphics (ce crate) │         │
//! │  clipping · pixel()  │         │
//! └──────────┬───────────┘         │
//!            └─────────────────────┘
//!                    │ draw_pixel()
//! ┌──────────────────▼───────────────────┐
//! │       embassy-ssd1306 (driver)       │
//! │  framebuffer · I2C · flush()         │
//! └──────────────────────────────────────┘
//! ```
//!
//! ## Patron de borrow
//!
//! `Graphics` tient un `&mut Ssd1306` pour toute sa durée de vie.
//! Pour appeler `oled.flush()`, `oled.clear()` ou `oled.draw_str()`,
//! `gfx` doit être sorti de portée au préalable.
//!
//! ```rust,no_run
//! loop {
//!     oled.clear();
//!     {
//!         let mut gfx = Graphics::new(&mut oled);
//!         line(&mut gfx, 0, 0, 127, 63, true);
//!         circle(&mut gfx, 64, 32, 20, true);
//!     } // ← borrow libéré
//!     oled.draw_str(40, 3, b"RPi2350");
//!     oled.flush().await.unwrap();
//! }
//! ```

use embassy_ssd1306::Ssd1306;
use embedded_hal_async::i2c::I2c;

// ─────────────────────────────────────────────────────────────────────────────
// Contexte graphique
// ─────────────────────────────────────────────────────────────────────────────

/// Contexte graphique.
///
/// Wraps minimalement un `&mut Ssd1306<I>` pour :
/// - centraliser le **clipping** des coordonnées
/// - fournir un `pixel()` signé (`i32`) aux algorithmes Bresenham / midpoint
///
/// Le driver reste propriétaire du framebuffer et du bus I2C.
pub struct Graphics<'a, I: I2c> {
    display: &'a mut Ssd1306<I>,
}

impl<'a, I: I2c> Graphics<'a, I> {
    /// Crée un contexte graphique pour un écran 128×64.
    #[inline]
    pub fn new(display: &'a mut Ssd1306<I>) -> Self {
        Self { display }
    }

    /// Dessine un pixel avec clipping automatique.
    ///
    /// Les coordonnées négatives ou hors de `[0, 128[` × `[0, 64[`
    /// sont silencieusement ignorées aucun panic, aucun wrapping.
    ///
    /// Le driver gère lui-même un second clipping sur `u8` ;
    /// ce niveau-ci permet aux algorithmes de travailler en `i32`
    /// sans conversions coûteuses.
    #[inline(always)]
    pub fn pixel(&mut self, x: i32, y: i32, on: bool) {
        if x >= 0 && y >= 0 && x < 128 && y < 64 {
            self.display.draw_pixel(x as u8, y as u8, on);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Ligne — Bresenham
// ─────────────────────────────────────────────────────────────────────────────

/// Trace une ligne entre `(x0, y0)` et `(x1, y1)`.
///
/// **Algorithme :** Bresenham integer-only.  
/// Zéro division flottante, zéro multiplication,safe sur tout MCU sans FPU.
///
/// # Exemple
///
/// ```rust,no_run
/// line(&mut gfx, 0, 0, 127, 63, true);  // diagonale complète
/// line(&mut gfx, 0, 0, 127, 63, false); // efface la diagonale
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

// ─────────────────────────────────────────────────────────────────────────────
// Cercle — midpoint
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Triangle
// ─────────────────────────────────────────────────────────────────────────────

/// Trace le **contour** d'un triangle défini par trois sommets.
///
/// Implémenté comme trois appels à [`line`] ,aucune logique propre.
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