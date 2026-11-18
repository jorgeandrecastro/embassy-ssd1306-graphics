//! Graphics context: [`Graphics`], the entry point for all primitives.

use embassy_ssd1306::Ssd1306;
use embedded_hal_async::i2c::I2c;

/// Contexte graphique.
///
/// Minimally wraps a `&mut Ssd1306<I>` to:
/// - centralize coordinate **clipping**
/// - provide a signed (`i32`) `pixel()` method to Bresenham / midpoint algorithms
///
/// The driver remains the owner of the framebuffer and I2C bus.
pub struct Graphics<'a, I: I2c> {
    display: &'a mut Ssd1306<I>,
}

impl<'a, I: I2c> Graphics<'a, I> {
    /// Creates a graphics context for a 128×64 display.
    #[inline]
    pub fn new(display: &'a mut Ssd1306<I>) -> Self {
        Self { display }
    }

    /// Draws a pixel with automatic clipping.
    ///
    /// Negative coordinates or coordinates outside `[0, 128[` × `[0, 64[`
    /// are silently ignored: no panic and no wrapping.
    ///
    /// The driver performs a second clipping check on `u8` itself;
    /// this layer lets algorithms work with `i32` without costly conversions.
    #[inline(always)]
    pub fn pixel(&mut self, x: i32, y: i32, on: bool) {
        if x >= 0 && y >= 0 && x < 128 && y < 64 {
            self.display.draw_pixel(x as u8, y as u8, on);
        }
    }
}