use std::f64::consts::PI;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

const W: usize = 74;
const H: usize = 24;
const CX: f64 = 27.5;
const CY: f64 = 13.0;
const SCALE: f64 = 6.5;
const TILT: f64 = 0.45;
const PLATE_R: f64 = 1.55;

type V3 = [f64; 3];
type Rgb = (u8, u8, u8);
type Col = (f64, f64, f64);

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_sigint(_: i32) {
    STOP.store(true, Ordering::SeqCst);
}

extern "C" {
    fn signal(signum: i32, handler: extern "C" fn(i32)) -> usize;
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: V3) -> V3 {
    let l = dot(a, a).sqrt().max(1e-12);
    mul(a, 1.0 / l)
}
fn rot_y(p: V3, a: f64) -> V3 {
    let (s, c) = a.sin_cos();
    [p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c]
}
fn rot_x(p: V3, a: f64) -> V3 {
    let (s, c) = a.sin_cos();
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]
}
fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mulc(c: Col, k: f64) -> Col {
    (c.0 * k, c.1 * k, c.2 * k)
}
fn lerp3(a: Col, b: Col, t: f64) -> Col {
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
    )
}
fn to_rgb(c: Col) -> Rgb {
    (
        c.0.clamp(0.0, 255.0) as u8,
        c.1.clamp(0.0, 255.0) as u8,
        c.2.clamp(0.0, 255.0) as u8,
    )
}

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    fg: Rgb,
    bg: Rgb,
}

fn put(c: &mut [Cell], x: usize, y: usize, ch: char, fg: Rgb, bg: Rgb) {
    if x < W && y < H {
        c[y * W + x] = Cell { ch, fg, bg };
    }
}

fn text(c: &mut [Cell], x: usize, y: usize, s: &str, fg: Rgb, bg: Rgb) {
    for (i, ch) in s.chars().enumerate() {
        put(c, x + i, y, ch, fg, bg);
    }
}

fn fill(c: &mut [Cell], x0: usize, y0: usize, x1: usize, y1: usize, fg: Rgb, bg: Rgb) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            put(c, x, y, ' ', fg, bg);
        }
    }
}

const SINGLE: [char; 6] = ['┌', '─', '┐', '│', '└', '┘'];
const DOUBLE: [char; 6] = ['╔', '═', '╗', '║', '╚', '╝'];

fn boxd(c: &mut [Cell], x0: usize, y0: usize, x1: usize, y1: usize, s: [char; 6], fg: Rgb, bg: Rgb) {
    for x in x0 + 1..x1 {
        put(c, x, y0, s[1], fg, bg);
        put(c, x, y1, s[1], fg, bg);
    }
    for y in y0 + 1..y1 {
        put(c, x0, y, s[3], fg, bg);
        put(c, x1, y, s[3], fg, bg);
    }
    put(c, x0, y0, s[0], fg, bg);
    put(c, x1, y0, s[2], fg, bg);
    put(c, x0, y1, s[4], fg, bg);
    put(c, x1, y1, s[5], fg, bg);
}

fn base_frame() -> Vec<Cell> {
    let body_bg: Rgb = (84, 86, 92);
    let body_fg: Rgb = (170, 174, 184);
    let mut c = vec![
        Cell {
            ch: ' ',
            fg: body_fg,
            bg: body_bg
        };
        W * H
    ];
    boxd(&mut c, 0, 0, W - 1, H - 1, DOUBLE, body_fg, body_bg);
    let door_bg: Rgb = (66, 68, 75);
    fill(&mut c, 2, 1, 53, 22, body_fg, door_bg);
    boxd(&mut c, 2, 1, 53, 22, SINGLE, (110, 114, 124), door_bg);
    boxd(&mut c, 4, 2, 51, 21, SINGLE, (170, 180, 195), door_bg);

    for y in 3..=20usize {
        for x in 5..=50usize {
            let dx = (x as f64 - CX) / 23.0;
            let dy = (y as f64 - 11.5) / 9.0;
            let d = (dx * dx + dy * dy).min(1.0);
            let f = 1.0 - 0.5 * d;
            let bg = to_rgb((58.0 * f, 48.0 * f, 32.0 * f));
            put(&mut c, x, y, ' ', (255, 255, 255), bg);
        }
    }

    for y in 8..=15usize {
        put(&mut c, 52, y, '█', (205, 208, 218), door_bg);
    }

    let pbg: Rgb = (46, 48, 54);
    fill(&mut c, 55, 1, 72, 22, body_fg, pbg);
    boxd(&mut c, 55, 1, 72, 22, DOUBLE, (120, 124, 134), pbg);
    text(&mut c, 56, 2, "MICRO-ONDAS 3000", (225, 225, 235), pbg);
    fill(&mut c, 57, 3, 70, 7, (255, 255, 255), (8, 12, 8));
    boxd(&mut c, 57, 3, 70, 7, SINGLE, (90, 95, 105), (8, 12, 8));
    text(&mut c, 58, 9, "MODO: BATATA", (255, 200, 90), pbg);
    text(&mut c, 57, 10, "POTENCIA: 800W", (190, 195, 205), pbg);
    let btn: Rgb = (200, 204, 214);
    text(&mut c, 59, 12, "[1][2][3]", btn, pbg);
    text(&mut c, 59, 13, "[4][5][6]", btn, pbg);
    text(&mut c, 59, 14, "[7][8][9]", btn, pbg);
    text(&mut c, 62, 15, "[0]", btn, pbg);
    text(&mut c, 58, 19, "[  START  ]", (255, 255, 255), (30, 120, 60));
    text(&mut c, 58, 20, "[  STOP   ]", (255, 255, 255), (140, 40, 40));
    c
}

struct Pt {
    p: V3,
    n: V3,
    base: Col,
    eye: f64,
}

struct PlatePt {
    p: V3,
    base: Col,
}

fn potato_surface(n: V3, eyes: &[V3]) -> (V3, f64) {
    let (x, y, z) = (n[0], n[1], n[2]);
    let mut r = 1.0
        + 0.07 * (3.1 * x + 1.3).sin() * (2.7 * y + 0.5).sin()
        + 0.05 * (4.3 * z + 2.0 * x).sin()
        + 0.03 * (7.0 * x + 5.0 * y + 6.0 * z).sin();
    let mut eye = 0.0f64;
    for e in eyes {
        let t = smoothstep((dot(n, *e) - 0.95) / 0.05);
        r -= 0.09 * t;
        eye = eye.max(t);
    }
    let p = [
        x * (1.0 + 0.15 * x) * r,
        0.6 * y * r * (1.0 + 0.1 * x),
        0.72 * z * r * (1.0 + 0.1 * x),
    ];
    (p, eye)
}

fn build_potato() -> Vec<Pt> {
    let eyes: Vec<V3> = [
        [0.6, 0.7, 0.4],
        [-0.3, 0.8, -0.5],
        [0.2, 0.5, 0.85],
        [-0.8, 0.3, 0.5],
        [0.9, -0.2, -0.4],
        [0.1, -0.9, 0.4],
        [-0.5, 0.4, -0.8],
    ]
    .iter()
    .map(|v| norm(*v))
    .collect();

    let sph = |u: f64, v: f64| -> V3 { [u.sin() * v.cos(), u.cos(), u.sin() * v.sin()] };
    let (nu, nv) = (120usize, 240usize);
    let e = 1e-3;
    let mut pts = Vec::with_capacity(nu * nv);

    for i in 0..nu {
        let u = (i as f64 + 0.5) / nu as f64 * PI;
        for j in 0..nv {
            let v = j as f64 / nv as f64 * 2.0 * PI;
            let s = sph(u, v);
            let (p, eye) = potato_surface(s, &eyes);
            let pu = sub(potato_surface(sph(u + e, v), &eyes).0, p);
            let pv = sub(potato_surface(sph(u, v + e), &eyes).0, p);
            let mut n = norm(cross(pu, pv));
            if dot(n, p) < 0.0 {
                n = mul(n, -1.0);
            }

            let tone = 0.92 + 0.08 * (11.0 * s[0] + 3.0).sin() * (9.0 * s[1] + 7.0 * s[2]).sin();
            let speck = (40.0 * s[0]).sin() * (37.0 * s[1]).sin() * (43.0 * s[2]).sin() > 0.82;
            let mut base = mulc((176.0, 132.0, 80.0), tone);
            if speck {
                base = mulc(base, 0.7);
            }
            base = lerp3(base, (70.0, 44.0, 24.0), eye * 0.85);
            pts.push(Pt { p, n, base, eye });
        }
    }
    pts
}

fn build_plate() -> Vec<PlatePt> {
    let mut v = Vec::new();
    let rings = 60;
    for ri in 0..=rings {
        let r = ri as f64 / rings as f64 * PLATE_R;
        let na = ((r * 300.0) as usize).max(1);
        for ai in 0..na {
            let a = ai as f64 / na as f64 * 2.0 * PI;
            let sector = ((a / (2.0 * PI)) * 12.0) as usize;
            let mut base: Col = if sector % 2 == 0 {
                (112.0, 162.0, 178.0)
            } else {
                (88.0, 132.0, 150.0)
            };
            if r < 0.28 {
                base = (175.0, 185.0, 190.0);
            }
            if (r - 0.95).abs() < 0.03 {
                base = (70.0, 100.0, 115.0);
            }
            let mut y = 0.0;
            if r > PLATE_R - 0.08 {
                base = (200.0, 225.0, 235.0);
                y = 0.04;
            }
            v.push(PlatePt {
                p: [r * a.cos(), y, r * a.sin()],
                base,
            });
        }
    }
    v
}

fn project(p: V3) -> (f64, f64, f64) {
    let f = 9.0 / (9.0 - p[2]);
    (CX + p[0] * SCALE * f * 2.0, CY - p[1] * SCALE * f, p[2])
}

fn shade(base: Col, n: V3, spec_k: f64) -> Col {
    let l = norm([-0.45, 0.75, 0.65]);
    let diff = dot(n, l).max(0.0);
    let h = norm(add(l, [0.0, 0.0, 1.0]));
    let spec = dot(n, h).max(0.0).powf(28.0) * spec_k;
    let k = 0.28 + 0.85 * diff;
    (
        base.0 * k + 255.0 * spec,
        base.1 * k + 255.0 * spec,
        base.2 * k + 255.0 * spec,
    )
}

fn plot(
    cells: &mut [Cell],
    zb: &mut [f64],
    sx: f64,
    sy: f64,
    z: f64,
    col: Col,
    bg_k: f64,
    ramp: &[char],
) {
    let x = sx.round() as i32;
    let y = sy.round() as i32;
    if x < 5 || x > 50 || y < 3 || y > 20 {
        return;
    }
    let i = y as usize * W + x as usize;
    if z <= zb[i] {
        return;
    }
    zb[i] = z;
    let lum = (0.299 * col.0 + 0.587 * col.1 + 0.114 * col.2) / 255.0;
    let top = (ramp.len() - 1) as f64;
    let idx = ((lum * 1.25 * top).round().max(0.0) as usize).min(ramp.len() - 1);
    cells[i] = Cell {
        ch: ramp[idx],
        fg: to_rgb(mulc(col, 1.25)),
        bg: to_rgb(mulc(col, bg_k)),
    };
}

fn render(t: f64, total: f64, potato: &[Pt], plate: &[PlatePt], base: &[Cell], finished: bool) -> Vec<Cell> {
    let mut cells = base.to_vec();
    let mut zb = vec![f64::NEG_INFINITY; W * H];

    let prog = (t / total).min(1.0);
    let angle = 0.9 * (t - (1.0 - (-2.0 * t).exp()) / 2.0);

    let pivot: V3 = [0.0, 0.35, 0.0];
    let center: V3 = [0.4, 0.58, 0.0]; // batata fora do centro => orbita
    let cooked: Col = (205.0, 148.0, 62.0);

    let plate_n = rot_x(rot_y([0.0, 1.0, 0.0], angle), TILT);
    let plate_ramp = ['.', ':', '-', '=', '+'];
    for pp in plate {
        let p = rot_x(sub(rot_y(pp.p, angle), pivot), TILT);
        let (sx, sy, z) = project(p);
        let sdx = (pp.p[0] - center[0]) / 1.2;
        let sdz = pp.p[2] / 0.85;
        let s = sdx * sdx + sdz * sdz;
        let shadow = if s < 1.0 { 0.45 + 0.55 * s } else { 1.0 };
        let col = mulc(shade(pp.base, plate_n, 0.25), shadow);
        plot(&mut cells, &mut zb, sx, sy, z, col, 0.4, &plate_ramp);
    }

    let potato_ramp = ['.', ':', '+', '*', '#', '%', '@'];
    for pt in potato {
        let n = rot_x(rot_y(pt.n, angle), TILT);
        if n[2] < -0.05 {
            continue;
        }
        let p = rot_x(sub(rot_y(add(pt.p, center), angle), pivot), TILT);
        let (sx, sy, z) = project(p);
        let base_col = lerp3(pt.base, cooked, 0.55 * prog);
        let col = shade(base_col, n, 0.35 * (1.0 - pt.eye));
        plot(&mut cells, &mut zb, sx, sy, z, col, 0.55, &potato_ramp);
    }

    if !finished {
        let n = 3 + (prog * 12.0) as usize;
        for i in 0..n {
            let fi = i as f64;
            let phase = (t * 0.35 + fi * 0.173) % 1.0;
            let x = 27.5 + 9.0 * (fi * 2.399).sin() + 1.8 * (t * 2.0 + fi).sin();
            let y = 7.5 - phase * 4.5;
            let xi = x.round() as i32;
            let yi = y.round() as i32;
            if xi < 5 || xi > 50 || yi < 3 || yi > 20 {
                continue;
            }
            let idx = yi as usize * W + xi as usize;
            if zb[idx] != f64::NEG_INFINITY {
                continue;
            }
            let v = (230.0 * (1.0 - phase)).max(40.0) as u8;
            let ch = if phase < 0.35 {
                'o'
            } else if phase < 0.7 {
                '~'
            } else {
                '.'
            };
            let bg = cells[idx].bg;
            cells[idx] = Cell {
                ch,
                fg: (v, v, v.saturating_add(10)),
                bg,
            };
        }
    }

    let rem = (total - t).max(0.0).ceil() as u32;
    let colon = if finished || (t * 2.0) as u32 % 2 == 0 { ':' } else { ' ' };
    let tm = format!("{:02}{}{:02}", rem / 60, colon, rem % 60);
    text(&mut cells, 61, 5, &tm, (90, 255, 130), (8, 12, 8));

    let pbg: Rgb = (46, 48, 54);
    if finished {
        text(&mut cells, 57, 17, "   PRONTO!!!   ", (255, 240, 120), pbg);
    } else {
        let lamp = if (t * 2.0) as u32 % 2 == 0 { "(*)" } else { "( )" };
        text(&mut cells, 57, 17, &format!("{} COZINHANDO", lamp), (255, 170, 60), pbg);
    }
    cells
}

fn present(cells: &[Cell], out: &mut impl Write) -> io::Result<()> {
    let mut s = String::with_capacity(W * H * 24);
    s.push_str("\x1b[H");
    for y in 0..H {
        let mut last: Option<(Rgb, Rgb)> = None;
        for x in 0..W {
            let c = cells[y * W + x];
            if last != Some((c.fg, c.bg)) {
                s.push_str(&format!(
                    "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
                    c.fg.0, c.fg.1, c.fg.2, c.bg.0, c.bg.1, c.bg.2
                ));
                last = Some((c.fg, c.bg));
            }
            s.push(c.ch);
        }
        s.push_str("\x1b[0m\r\n");
    }
    s.push_str("Ctrl+C para cancelar\x1b[K");
    out.write_all(s.as_bytes())?;
    out.flush()
}

fn main() -> io::Result<()> {
    let total: f64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .filter(|v: &f64| *v > 0.0)
        .unwrap_or(30.0);

    unsafe {
        signal(2, on_sigint);
    }

    let potato = build_potato();
    let plate = build_plate();
    let base = base_frame();

    let stdout = io::stdout();
    let mut out = stdout.lock();
    out.write_all(b"\x1b[2J\x1b[?25l")?;

    let start = Instant::now();
    let frame_time = Duration::from_millis(33);
    let mut finished_normally = false;

    loop {
        if STOP.load(Ordering::SeqCst) {
            break;
        }
        let frame_start = Instant::now();
        let t = start.elapsed().as_secs_f64();
        let done = t >= total;

        let cells = render(t, total, &potato, &plate, &base, done);
        present(&cells, &mut out)?;

        if done {
            finished_normally = true;
            break;
        }
        let spent = frame_start.elapsed();
        if spent < frame_time {
            sleep(frame_time - spent);
        }
    }

    out.write_all(b"\x1b[0m\x1b[?25h\r\n")?;
    if finished_normally {
        out.write_all(b"\x07DING! Sua batata esta pronta.\r\n")?;
    } else {
        out.write_all(b"Cozimento cancelado.\r\n")?;
    }
    out.flush()
}
