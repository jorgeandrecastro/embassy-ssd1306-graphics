#![no_std]
#![forbid(unsafe_code)]
//! # embassy-ssd1306-graphics
//!
//! `no_std` 2D graphics layer for SSD1306 OLED displays (128×64),
//! built on top of `embassy-ssd1306`.
//!
//! ## What this crate does
//!
//! The `embassy-ssd1306` driver already provides:
//! - `draw_pixel()`, `draw_hline()`, `draw_vline()`
//! - `draw_rect()`, `draw_filled_rect()`
//! - `draw_char()`, `draw_str()`, `draw_i16()`
//! - `draw_bitmap()`
//! - `clear()`, `fill()`, `flush()`
//!
//! This crate **does not duplicate anything**. It only adds primitives
//! that the driver does not provide:
//! | Function              | Algorithm                  | File           |
//! |------------------------|-----------------------------|----------------|
//! | [`line()`]              | Integer-only Bresenham      | `line.rs`      |
//! | [`circle()`]            | Integer-only midpoint       | `circle.rs`    |
//! | [`fill_circle()`]       | Midpoint + hlines           | `circle.rs`    |
//! | [`triangle()`]          | 3 calls to [`line()`]       | `triangle.rs`  |
//! | [`ellipse()`]           | Generalized midpoint        | `ellipse.rs`   |
//! | [`bezier_quad()`]       | Integer-only De Casteljau   | `bezier.rs`    |
//! | [`fill_triangle()`]     | Integer-only scanline       | `triangle.rs`  |
//!
//! The modules (`line`, `circle`, `triangle`, `ellipse`, `bezier`) are an
//! internal organization of the source files: they remain private and are not
//! part of the public API; only the functions listed above are exported.
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────┐
//! │          Your application            │
//! │  line() / circle() / triangle() …   │
//! │  oled.draw_str() / oled.draw_i16()  │  ← driver directly for text
//! └──────────┬───────────────────────────┘
//!            │ &mut Graphics       │ &mut Ssd1306
//! ┌──────────▼───────────┐         │
//! │  Graphics (this crate)│         │
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
//! ## File layout
//!
//! ```text
//! src/
//! ├── lib.rs         crate docs, module declarations, re-exports
//! ├── context.rs     Graphics<'a, I>: new, pixel (clipping)
//! ├── line.rs        line()               (Bresenham)
//! ├── circle.rs      circle(), fill_circle() (midpoint)
//! ├── triangle.rs    triangle(), fill_triangle() (3×line / scanline)
//! ├── ellipse.rs     ellipse()            (generalized midpoint)
//! └── bezier.rs      bezier_quad()         (De Casteljau)
//! ```
//!
//! ## Borrowing pattern
//!
//! `Graphics` holds a `&mut Ssd1306` for its entire lifetime.
//! To call `oled.flush()`, `oled.clear()`, or `oled.draw_str()`,
//! `gfx` must first go out of scope.
//!
//! ```rust,no_run
//! loop {
//!     oled.clear();
//!     {
//!         let mut gfx = Graphics::new(&mut oled);
//!         line(&mut gfx, 0, 0, 127, 63, true);
//!         circle(&mut gfx, 64, 32, 20, true);
//!     } // ← borrow released
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