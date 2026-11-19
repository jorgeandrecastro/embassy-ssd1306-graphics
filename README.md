# embassy-ssd1306-graphics

[![Crates.io](https://img.shields.io/crates/v/embassy-ssd1306-graphics)](https://crates.io/crates/embassy-ssd1306-graphics)
[![Docs.rs](https://docs.rs/embassy-ssd1306-graphics/badge.svg)](https://docs.rs/embassy-ssd1306-graphics)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-✓-green)](https://docs.rust-embedded.org/book/)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

`no_std` graphics primitives for SSD1306 OLED displays,
built on top of [`embassy-ssd1306`](https://crates.io/crates/embassy-ssd1306).

---

## What this crate does

The `embassy-ssd1306` driver already provides:

| Already in the driver | Use |
|---|---|
| `draw_pixel()` | Individual pixels |
| `draw_hline()` / `draw_vline()` | Horizontal and vertical lines |
| `draw_rect()` / `draw_filled_rect()` | Rectangles |
| `draw_char()` / `draw_str()` / `draw_i16()` | Text and numbers |
| `draw_bitmap()` | 1bpp bitmap |
| `clear()` / `fill()` / `flush()` | Framebuffer and I2C |

This crate **does not duplicate any of these**. It only adds primitives that the driver does not provide:

| Function | Algorithm | Adds |
|---|---|---|
| `line()` | Integer-only Bresenham | Arbitrary diagonal lines |
| `circle()` | Integer-only midpoint | Circle outline |
| `fill_circle()` | Midpoint + horizontal lines | Filled disk |
| `triangle()` | 3 × `line()` | Triangle outline |
| `ellipse()` | Generalized midpoint | Ellipse outline |
| `bezier_quad()` | Integer-only De Casteljau | Quadratic Bézier curve |
| `fill_triangle()` | Integer-only scanline | Filled triangle |

---

## Design principles

| Principle | Details |
|---|---|
| No duplication | Delegates everything the driver can already do |
| No allocation | No `Vec`, no `Box` |
| Integer-only | No floating-point arithmetic in the algorithms |
| `#![forbid(unsafe_code)]` | Enforced at compile time |

---

## Installation

```toml
[dependencies]
embassy-ssd1306          = "0.7"
embassy-ssd1306-graphics = "0.2"
```

---

## RISC-V support

The crate can be used on RISC-V microcontrollers with Embassy. Its graphics
algorithms are integer-only and do not depend on floating-point hardware.

Use the `time-riscv` feature to select the RISC-V Embassy time backend:

```toml
[dependencies]
embassy-ssd1306 = { version = "0.7", default-features = false }
embassy-ssd1306-graphics = { version = "0.2", default-features = false, features = ["time-riscv"] }
```

The time backends are alternatives: use `time-riscv` for RISC-V builds and
`time-upstream` (enabled by default) for standard Embassy builds. Do not enable
both time backends together.

---

## Quick start

```rust
#![no_std]
#![no_main]

use embassy_ssd1306::Ssd1306;
use embassy_ssd1306_graphics::{Graphics, circle, ellipse, fill_triangle, line, triangle, bezier_quad};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let i2c = /* initialize I2C for your MCU */;

    let mut oled = Ssd1306::new(i2c, 0x3C);
    oled.init().await.unwrap();

    loop {
        oled.clear();

        {
            let mut gfx = Graphics::new(&mut oled);

            // Primitives from this crate
            line(&mut gfx, 0, 0, 127, 63, true);
            circle(&mut gfx, 64, 32, 20, true);
            ellipse(&mut gfx, 64, 32, 40, 16, true);
            fill_triangle(&mut gfx, 64, 4, 20, 59, 108, 59, true);
            bezier_quad(&mut gfx, 10, 50, 64, 5, 118, 50, 24, true);
        }
        // ↑ borrow released — oled is accessible again

        // Draw text and flush directly through the driver
        oled.draw_str(40, 7, b"Hello!");
        oled.flush().await.unwrap();
    }
}
```

> **⚠ Borrowing**: `Graphics` holds an exclusive mutable borrow of `oled`.
> Wrap it in a `{}` block so you can call `oled.draw_str()`,
> `oled.clear()`, and `oled.flush()` outside the block.

---

## Architecture

```
┌──────────────────────────────────────────┐
│            Your application              │
│  line() / circle() / triangle()          │  ← this crate
│  oled.draw_str() / oled.draw_i16() …     │  ← driver directly
└──────────┬───────────────────────────────┘
           │ &mut Graphics
┌──────────▼──────────────┐
│  Graphics (this crate)  │
│  i32 clipping · pixel() │
└──────────┬──────────────┘
           │ draw_pixel(u8, u8)
┌──────────▼──────────────────────────────┐
│       embassy-ssd1306 (driver)           │
│  framebuffer · I2C · flush()             │
└─────────────────────────────────────────┘
```

This crate is itself divided into modules (`context`, `line`, `circle`, `triangle`,
`ellipse`, `bezier`), each containing only its corresponding algorithm. See the
crate documentation (`cargo doc --open`) for details on the module layout.

---

## API

### `Graphics<'a, I>`

```rust
/// Creates the graphics context.
pub fn new(display: &'a mut Ssd1306<I>) -> Self

/// Draws an individual pixel with automatic i32 clipping.
/// Coordinates outside [0,128[ × [0,64[ are silently ignored.
pub fn pixel(&mut self, x: i32, y: i32, on: bool)
```

---

### `line`

```rust
pub fn line<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    on: bool,
)
```

Draws a line between any two points.  
Integer-only **Bresenham** algorithm: no floating-point division, safe on MCUs without an FPU.

---

### `circle`

```rust
pub fn circle<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    cx: i32, cy: i32,
    r: i32,
    on: bool,
)
```

Draws a circle outline.  
8-octant **midpoint** algorithm: 8 symmetric pixels per iteration.

---

### `fill_circle`

```rust
pub fn fill_circle<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    cx: i32, cy: i32,
    r: i32,
    on: bool,
)
```

Fills a disk by drawing symmetric horizontal scanlines.

---

### `triangle`

```rust
pub fn triangle<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    x2: i32, y2: i32,
    on: bool,
)
```

Draws a triangle outline using three calls to `line()`.

---

### `ellipse`

```rust
pub fn ellipse<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    cx: i32, cy: i32,
    rx: i32, ry: i32,
    on: bool,
)
```

Draws an ellipse outline using a generalized, integer-only midpoint algorithm.

---

### `bezier_quad`

```rust
pub fn bezier_quad<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    x2: i32, y2: i32,
    steps: i32,
    on: bool,
)
```

Draws a quadratic Bézier curve using integer-only De Casteljau subdivision.

---

### `fill_triangle`

```rust
pub fn fill_triangle<I: I2c>(
    gfx: &mut Graphics<'_, I>,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    x2: i32, y2: i32,
    on: bool,
)
```

Fills a triangle using an integer-only scanline algorithm.

---

## Examples

### Rotating arm animation (with `embedded-trig-f32`)

```rust
use embedded_trig_f32::{sin, cos};

let mut angle: f32 = 0.0;
loop {
    oled.clear();
    {
        let mut gfx = Graphics::new(&mut oled);
        let x = (64.0 + 25.0 * cos(angle)) as i32;
        let y = (32.0 + 25.0 * sin(angle)) as i32;
        line(&mut gfx, 64, 32, x, y, true);
        circle(&mut gfx, x, y, 4, true);
    }
    oled.draw_str(0, 7, b"RP2350");
    oled.flush().await.unwrap();

    angle += 0.08;
    if angle > embedded_trig_f32::consts::TAU { angle = 0.0; }
    Timer::after_millis(50).await;
}
```

### Progress bar (driver only; this crate is not needed)

```rust
// draw_filled_rect is already provided by the driver
oled.draw_rect(10, 28, 108, 8, true);
oled.draw_filled_rect(11, 29, filled, 6, true);
```

---


## 📋 Changelog

**v0.2.0**: Split the crate into modules (`context`, `line`, `circle`, `triangle`,
`ellipse`, `bezier`) and updated the `embassy-ssd1306` dependency to `0.7.0`.

**v0.1.0**: Initial release with `line`, `circle`, `fill_circle`, `triangle`,
`fill_triangle`, `ellipse`, and `bezier_quad`.

---

## License

Distributed under the **GNU GPL v2.0 or later** license.  
See [LICENSE](LICENSE) for the full text.

---

## Author

**Jorge Andre Castro**
