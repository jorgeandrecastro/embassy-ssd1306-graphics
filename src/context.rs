//! Contexte graphique : [`Graphics`], le point d'entrée de toutes les primitives.

use embassy_ssd1306::Ssd1306;
use embedded_hal_async::i2c::I2c;

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
    /// sont silencieusement ignorées  aucun panic, aucun wrapping.
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