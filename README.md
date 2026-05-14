# embassy-ssd1306-graphics

[![Crates.io](https://img.shields.io/crates/v/embassy-ssd1306-graphics)](https://crates.io/crates/embassy-ssd1306-graphics)
[![Docs.rs](https://docs.rs/embassy-ssd1306-graphics/badge.svg)](https://docs.rs/embassy-ssd1306-graphics)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-✓-green)](https://docs.rust-embedded.org/book/)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Primitives graphiques `no_std` pour écrans OLED SSD1306,
construites au-dessus de [`embassy-ssd1306`](https://crates.io/crates/embassy-ssd1306).

---

## Rôle exact de ce crate

Le driver `embassy-ssd1306` fournit déjà :

| Déjà dans le driver | Utilisation |
|---|---|
| `draw_pixel()` | Pixel individuel |
| `draw_hline()` / `draw_vline()` | Lignes directionnelles |
| `draw_rect()` / `draw_filled_rect()` | Rectangles |
| `draw_char()` / `draw_str()` / `draw_i16()` | Texte et nombres |
| `draw_bitmap()` | Bitmap 1bpp |
| `clear()` / `fill()` / `flush()` | Framebuffer et I2C |

Ce crate **n'en duplique aucun**. Il ajoute uniquement les primitives que le driver ne propose pas :

| Fonction | Algorithme | Apport |
|---|---|---|
| `line()` | Bresenham integer-only | Lignes obliques quelconques |
| `circle()` | Midpoint integer-only | Cercle contour |
| `fill_circle()` | Midpoint + hlines | Disque plein |
| `triangle()` | 3 × `line()` | Triangle contour |
| `ellipse()` | Midpoint généralisé | Contour d'ellipse |
| `bezier_quad()` | De Casteljau integer-only | Courbe de Bézier quadratique |
| `fill_triangle()` | Scanline integer-only | Triangle plein |

---

## Philosophie

| Principe | Détail |
|---|---|
| Zéro duplication | Délègue tout ce que le driver sait faire |
| Zéro allocation | Pas de `Vec`, pas de `Box` |
| Integer-only | Aucun flottant dans les algorithmes |
| `#![forbid(unsafe_code)]` | Garanti à la compilation |

---

## Installation

```toml
[dependencies]
embassy-ssd1306          = "0.6.0"
embassy-ssd1306-graphics = "0.1.0"
```

---

## Démarrage rapide

```rust
#![no_std]
#![no_main]

use embassy_ssd1306::Ssd1306;
use embassy_ssd1306_graphics::{Graphics, circle, ellipse, fill_triangle, line, triangle, bezier_quad};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let i2c = /* initialisation I2C selon votre MCU */;

    let mut oled = Ssd1306::new(i2c, 0x3C);
    oled.init().await.unwrap();

    loop {
        oled.clear();

        {
            let mut gfx = Graphics::new(&mut oled);

            // Primitives de ce crate
            line(&mut gfx, 0, 0, 127, 63, true);
            circle(&mut gfx, 64, 32, 20, true);
            ellipse(&mut gfx, 64, 32, 40, 16, true);
            fill_triangle(&mut gfx, 64, 4, 20, 59, 108, 59, true);
            bezier_quad(&mut gfx, 10, 50, 64, 5, 118, 50, 24, true);
        }
        // ↑ borrow libéré oled à nouveau accessible

        // Texte et flush via le driver directement
        oled.draw_str(40, 7, b"Hello!");
        oled.flush().await.unwrap();
    }
}
```

> **⚠ Borrow** : `Graphics` tient un borrow mutable exclusif sur `oled`.
> Encadrez-le dans un bloc `{}` pour pouvoir appeler `oled.draw_str()`,
> `oled.clear()` et `oled.flush()` en dehors.

---

## Architecture

```
┌──────────────────────────────────────────┐
│            Votre application             │
│  line() / circle() / triangle()          │  ← ce crate
│  oled.draw_str() / oled.draw_i16() …     │  ← driver direct
└──────────┬───────────────────────────────┘
           │ &mut Graphics
┌──────────▼──────────────┐
│  Graphics (ce crate)    │
│  clipping i32 · pixel() │
└──────────┬──────────────┘
           │ draw_pixel(u8, u8)
┌──────────▼──────────────────────────────┐
│       embassy-ssd1306 (driver)          │
│  framebuffer · I2C · flush()            │
└─────────────────────────────────────────┘
```

---

## API

### `Graphics<'a, I>`

```rust
/// Crée le contexte graphique.
pub fn new(display: &'a mut Ssd1306<I>) -> Self

/// Pixel individuel avec clipping i32 automatique.
/// Coordonnées hors [0,128[ × [0,64[ silencieusement ignorées.
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

Trace une ligne entre deux points quelconques.  
Algorithme de **Bresenham** integer-only  aucune division flottante, safe sans FPU.

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

Trace le contour d'un cercle.  
Algorithme **midpoint** 8-octants 8 pixels symétriques par itération.

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

Remplit un disque par balayage de lignes horizontales symétriques.

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

Trace le contour d'un triangle via trois appels à `line()`.

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

Trace le contour d'une ellipse. Algorithme midpoint généralisé, integer-only.

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

Trace une courbe de Bézier quadratique via De Casteljau integer-only.

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

Remplit un triangle plein par scanline integer-only.

---

## Exemples

### Animation bras rotatif (avec `embedded-trig-f32`)

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

### Barre de progression (driver seul, pas besoin de ce crate)

```rust
// draw_filled_rect est déjà dans le driver
oled.draw_rect(10, 28, 108, 8, true);
oled.draw_filled_rect(11, 29, filled, 6, true);
```

---

## Compatibilité

| Crate | Version |
|---|---|
| `embassy-ssd1306` | 0.6.0 |
| `embedded-hal-async` | 1.0 |
| Rust edition | 2024 |

Testé sur : **RP2040**, **RP2350**, (via Embassy).


---

## Licence

Distribué sous licence **GNU GPL v2.0 ou ultérieure**.  
Voir [LICENSE](LICENSE) pour le texte complet.

---

## Auteur

**Jorge Andre Castro** 