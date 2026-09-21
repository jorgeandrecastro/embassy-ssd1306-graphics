#![no_std]
#![forbid(unsafe_code)]
//! # embassy-ssd1306-graphics
//!
//! Couche graphique 2D `no_std` pour écrans OLED SSD1306 (128×64),
//! construite au-dessus de `embassy-ssd1306`.
//!
//! ## Rôle exact de ce crate
//!
//! Le driver `embassy-ssd1306` fournit déjà :
//! - `draw_pixel()`, `draw_hline()`, `draw_vline()`
//! - `draw_rect()`, `draw_filled_rect()`
//! - `draw_char()`, `draw_str()`, `draw_i16()`
//! - `draw_bitmap()`
//! - `clear()`, `fill()`, `flush()`
//!
//! Ce crate **ne duplique rien**. Il ajoute uniquement les primitives
//! que le driver ne propose pas :
//! | Fonction              | Algorithme                 | Fichier        |
//! |------------------------|-----------------------------|----------------|
//! | [`line()`]              | Bresenham integer-only      | `line.rs`      |
//! | [`circle()`]            | Midpoint integer-only       | `circle.rs`    |
//! | [`fill_circle()`]       | Midpoint + hlines           | `circle.rs`    |
//! | [`triangle()`]          | 3 appels à [`line()`]       | `triangle.rs`  |
//! | [`ellipse()`]           | Midpoint généralisé         | `ellipse.rs`   |
//! | [`bezier_quad()`]       | De Casteljau integer-only   | `bezier.rs`    |
//! | [`fill_triangle()`]     | Scanline integer-only       | `triangle.rs`  |
//!
//! Les modules (`line`, `circle`, `triangle`, `ellipse`, `bezier`) sont un
//! simple découpage interne du fichier source : ils restent privés et ne font
//! pas partie de l'API publique, seules les fonctions ci-dessus sont exportées.
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
//! ## Découpage des fichiers
//!
//! ```text
//! src/
//! ├── lib.rs        doc de crate, déclaration des modules, ré-exports
//! ├── context.rs    Graphics<'a, I> : new, pixel (clipping)
//! ├── line.rs        line()               (Bresenham)
//! ├── circle.rs       circle(), fill_circle() (Midpoint)
//! ├── triangle.rs      triangle(), fill_triangle() (3×line / scanline)
//! ├── ellipse.rs       ellipse()            (Midpoint généralisé)
//! └── bezier.rs        bezier_quad()         (De Casteljau)
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

mod bezier;
mod circle;
mod context;
mod ellipse;
mod line;
mod triangle;

pub use bezier::bezier_quad;
pub use circle::{circle, fill_circle};
pub use context::Graphics;
pub use ellipse::ellipse;
pub use line::line;
pub use triangle::{fill_triangle, triangle};