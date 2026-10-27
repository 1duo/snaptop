//! Palettes and the bar gradient.

use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub faint: Color,
    pub border: Color,
    pub cpu: Color,
    pub gpu: Color,
    pub npu: Color,
    pub mem: Color,
    pub warn: Color,
    pub bad: Color,
    pub light: bool,
}

pub const AURORA: Palette = Palette {
    bg: Color::Rgb(10, 12, 17),
    fg: Color::Rgb(216, 224, 238),
    dim: Color::Rgb(138, 150, 168),
    faint: Color::Rgb(58, 66, 80),
    border: Color::Rgb(112, 126, 148),
    cpu: Color::Rgb(72, 220, 198),
    gpu: Color::Rgb(176, 148, 255),
    npu: Color::Rgb(250, 192, 70),
    mem: Color::Rgb(108, 170, 255),
    warn: Color::Rgb(250, 152, 72),
    bad: Color::Rgb(246, 112, 112),
    light: false,
};

pub const DAYLIGHT: Palette = Palette {
    bg: Color::Rgb(248, 249, 251),
    fg: Color::Rgb(26, 30, 38),
    dim: Color::Rgb(88, 98, 114),
    faint: Color::Rgb(206, 212, 222),
    border: Color::Rgb(158, 167, 182),
    cpu: Color::Rgb(0, 112, 104),
    gpu: Color::Rgb(92, 58, 188),
    npu: Color::Rgb(150, 92, 0),
    mem: Color::Rgb(28, 88, 188),
    warn: Color::Rgb(176, 88, 0),
    bad: Color::Rgb(184, 36, 44),
    light: true,
};

pub const MONO: Palette = Palette {
    bg: Color::Rgb(10, 10, 10),
    fg: Color::Rgb(232, 232, 232),
    dim: Color::Rgb(150, 150, 150),
    faint: Color::Rgb(52, 52, 52),
    border: Color::Rgb(96, 96, 96),
    cpu: Color::Rgb(232, 232, 232),
    gpu: Color::Rgb(200, 200, 200),
    npu: Color::Rgb(255, 255, 255),
    mem: Color::Rgb(180, 180, 180),
    warn: Color::Rgb(232, 232, 232),
    bad: Color::Rgb(255, 255, 255),
    light: false,
};

pub const PALETTES: [Palette; 3] = [AURORA, DAYLIGHT, MONO];
pub const NAMES: [&str; 3] = ["aurora", "daylight", "mono"];

pub fn palette(index: usize) -> Palette {
    PALETTES[index.min(PALETTES.len() - 1)]
}

pub fn name(index: usize) -> &'static str {
    NAMES[index.min(NAMES.len() - 1)]
}

pub fn index_of(value: &str) -> Option<usize> {
    match value.to_ascii_lowercase().as_str() {
        "aurora" | "dark" => Some(0),
        "daylight" | "light" => Some(1),
        "mono" => Some(2),
        _ => None,
    }
}

/// Pick a palette for the current terminal: explicit env override, then the OS
/// app theme (Windows Terminal follows it by default), then `COLORFGBG`.
pub fn auto_index() -> usize {
    if let Ok(value) = std::env::var("SNAPTOP_THEME")
        && let Some(index) = index_of(&value)
    {
        return index;
    }
    if let Some(light) = os_prefers_light() {
        return if light { 1 } else { 0 };
    }
    if let Ok(value) = std::env::var("COLORFGBG")
        && let Some(bg) = value
            .rsplit(';')
            .next()
            .and_then(|n| n.trim().parse::<u8>().ok())
    {
        return if bg >= 7 { 1 } else { 0 };
    }
    0
}

#[cfg(windows)]
fn os_prefers_light() -> Option<bool> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .ok()?;
    let value: u32 = key.get_value("AppsUseLightTheme").ok()?;
    Some(value != 0)
}

#[cfg(not(windows))]
fn os_prefers_light() -> Option<bool> {
    None
}

fn rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}

fn mix(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color::Rgb(f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

/// Bars get slightly denser toward the tip: lighter on dark, darker on light.
pub fn gradient(accent: Color, t: f32, p: &Palette) -> Color {
    let base = rgb(accent);
    if p.light {
        mix(base, (0, 0, 0), 0.12 + 0.22 * t.clamp(0.0, 1.0))
    } else {
        mix(base, rgb(p.fg), 0.35 + 0.45 * t.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(color: Color) -> f32 {
        let (r, g, b) = rgb(color);
        let channel = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
    }

    fn contrast(a: Color, b: Color) -> f32 {
        let (l1, l2) = (luminance(a), luminance(b));
        (l1.max(l2) + 0.05) / (l1.min(l2) + 0.05)
    }

    #[test]
    fn every_palette_is_legible() {
        for (index, p) in PALETTES.iter().enumerate() {
            assert!(
                contrast(p.fg, p.bg) >= 7.0,
                "{} body text contrast {:.1}",
                name(index),
                contrast(p.fg, p.bg)
            );
            assert!(
                contrast(p.dim, p.bg) >= 4.5,
                "{} dim text contrast {:.1}",
                name(index),
                contrast(p.dim, p.bg)
            );
            for accent in [p.cpu, p.gpu, p.npu, p.mem, p.warn, p.bad] {
                assert!(
                    contrast(accent, p.bg) >= 3.0,
                    "{} accent contrast {:.1}",
                    name(index),
                    contrast(accent, p.bg)
                );
            }
        }
    }

    #[test]
    fn light_and_dark_are_classified() {
        assert!(luminance(DAYLIGHT.bg) > 0.8);
        assert!(luminance(AURORA.bg) < 0.05);
    }

    #[test]
    fn names_and_aliases_resolve() {
        assert_eq!(index_of("dark"), Some(0));
        assert_eq!(index_of("light"), Some(1));
        assert_eq!(index_of("daylight"), Some(1));
        assert_eq!(index_of("mono"), Some(2));
        assert_eq!(index_of("nope"), None);
    }
}
