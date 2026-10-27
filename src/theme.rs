//! Colour palettes and the bar gradient used by the meters.

use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub faint: Color,
    pub cpu: Color,
    pub gpu: Color,
    pub npu: Color,
    pub mem: Color,
    pub warn: Color,
    pub bad: Color,
}

pub const AURORA: Palette = Palette {
    bg: Color::Rgb(11, 13, 18),
    fg: Color::Rgb(214, 222, 235),
    dim: Color::Rgb(120, 132, 150),
    faint: Color::Rgb(45, 52, 64),
    cpu: Color::Rgb(64, 224, 208),
    gpu: Color::Rgb(167, 139, 250),
    npu: Color::Rgb(251, 191, 36),
    mem: Color::Rgb(96, 165, 250),
    warn: Color::Rgb(251, 146, 60),
    bad: Color::Rgb(248, 113, 113),
};

pub const NEBULA: Palette = Palette {
    bg: Color::Rgb(14, 11, 20),
    fg: Color::Rgb(232, 224, 245),
    dim: Color::Rgb(140, 130, 160),
    faint: Color::Rgb(48, 42, 62),
    cpu: Color::Rgb(96, 205, 255),
    gpu: Color::Rgb(244, 114, 182),
    npu: Color::Rgb(163, 230, 53),
    mem: Color::Rgb(129, 140, 248),
    warn: Color::Rgb(251, 191, 36),
    bad: Color::Rgb(251, 113, 133),
};

pub const MONO: Palette = Palette {
    bg: Color::Rgb(12, 12, 12),
    fg: Color::Rgb(226, 226, 226),
    dim: Color::Rgb(128, 128, 128),
    faint: Color::Rgb(44, 44, 44),
    cpu: Color::Rgb(235, 235, 235),
    gpu: Color::Rgb(200, 200, 200),
    npu: Color::Rgb(255, 255, 255),
    mem: Color::Rgb(180, 180, 180),
    warn: Color::Rgb(230, 230, 230),
    bad: Color::Rgb(255, 255, 255),
};

pub const PALETTES: [Palette; 3] = [AURORA, NEBULA, MONO];

/// Colour for a utilisation value, independent of the subsystem accent.
pub fn heat(p: &Palette, v: f32) -> Color {
    if v >= 90.0 {
        p.bad
    } else if v >= 70.0 {
        p.warn
    } else {
        p.fg
    }
}

fn mix(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color::Rgb(f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}

/// A left-to-right brightening gradient, so a full bar looks lit rather than flat.
pub fn gradient(accent: Color, t: f32, p: &Palette) -> Color {
    let base = rgb(accent);
    let lift = rgb(p.fg);
    mix(base, lift, 0.40 + 0.45 * t.clamp(0.0, 1.0))
}
