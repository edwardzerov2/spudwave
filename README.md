<div align="center">

# Spudwave

**A 3D potato spinning in a microwave, right in your terminal.**
Written in Rust, with zero external dependencies.

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)
![Dependencies](https://img.shields.io/badge/dependencies-0-brightgreen)
![Terminal](https://img.shields.io/badge/terminal-truecolor-blueviolet)
![License](https://img.shields.io/badge/license-MIT-blue)

</div>

---

## Features

- **A proper 3D potato**: egg-shaped surface with bumps, dimples ("eyes") and dark specks on the skin
- **Lighting**: per-point diffuse and specular (Blinn-Phong) shading
- **Perspective camera**: slightly top-down view with a z-buffer
- **Rotating turntable**: the potato sits off-center, so it *orbits* while it spins, and casts a shadow on the glass plate
- **Cooking**: the potato turns golden over time and gives off steam
- **Working control panel**: countdown timer, blinking lamp and a *DING!* at the end
- **`std` only**: no crates, not even for Ctrl+C

## What the scene looks like

```
╔════════════════════════════════════════════════════╦══════════════════╗
║ ┌────────────────────────────────────────────────┐ ║ MICRO-ONDAS 3000 ║
║ │                      o ~                       │ ║ ┌──────────────┐ ║
║ │                  .:+*##%+:.                    │ ║ │    00:27     │ ║
║ │                .+#%@@@@@%#*:                   │ ║ └──────────────┘ ║
║ │                 :*#%@@@%#+.                    │ ║ MODO: BATATA     ║
║ │            :-=+=-:..:+*+=-:.                   │ ║ [1][2][3]        ║
║ │         .:-==++==-----=+++==-:.                │ ║ (*) COZINHANDO   ║
║ └────────────────────────────────────────────────┘ ║ [  START  ]      ║
╚════════════════════════════════════════════════════╩══════════════════╝
```

> Simplified illustration. In the terminal everything is rendered in 24-bit color. The control panel text is in Portuguese.

## Getting started

### Requirements

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or newer)
- A terminal with **truecolor** (24-bit) support. Works well in GNOME Terminal, Konsole, Alacritty, kitty, WezTerm, iTerm2 and Windows Terminal
- Minimum size: **74 columns x 26 rows**

### Running

```bash
git clone https://github.com/edwardzerov2/spudwave.git
cd spudwave

cargo run --release            # 30 seconds (default)
cargo run --release -- 60      # 60 seconds
```

Press **Ctrl+C** to cancel at any time. If the cursor stays hidden afterwards, run `tput cnorm`.

## How it works

| Stage | What happens |
| --- | --- |
| **Geometry** | The potato is a unit sphere deformed by sine waves (bumps), with dimples at the "eyes" and an egg-shaped taper. Points and normals are computed **once** at startup |
| **Transform** | Every frame, the points rotate around the plate's center, get recentered, and are tilted by the camera |
| **Projection** | Simple perspective, corrected for terminal cell proportions (cells are ~2x taller than wide) |
| **Visibility** | Per-cell z-buffer, with back-face culling |
| **Shading** | Diffuse + specular; brightness picks the character (`.:+*#%@`) and the color becomes the cell background |
| **Output** | The whole frame is built as one `String` of ANSI escapes and written at once, only switching colors when needed |

## Customizing

Everything lives in [`src/main.rs`](src/main.rs):

| To change... | Edit... |
| --- | --- |
| Potato shape and bumpiness | `potato_surface` |
| Position and number of eyes | the `eyes` list in `build_potato` |
| Skin color / "cooked" color | `build_potato` and the `cooked` variable in `render` |
| Turntable speed | the `0.9` factor in the `angle` variable in `render` |
| Camera angle | the `TILT` constant |
| Scene size | the `SCALE` constant |
| Light position | the vector in `shade` |
| Panel text and buttons | `base_frame` |

## Project structure

```
spudwave/
├── Cargo.toml
├── README.md
└── src/
    └── main.rs
```

## License

Distributed under the MIT license. Do whatever you want with the potato.

---

<div align="center">
<sub>Made with Rust and a little bit of hunger.</sub>
</div>
