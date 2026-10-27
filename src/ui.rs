//! Rendering. Hand-painted on top of ratatui's Buffer: meters, braille
//! sparklines and per-core bars all need per-cell colour, which widgets do
//! not expose cleanly.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use crate::app::App;
use crate::fmt;
use crate::model::CoreKind;
use crate::theme::{self, Palette};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let palette = theme::PALETTES[app.theme.min(theme::PALETTES.len() - 1)];

    f.render_widget(
        Block::new().style(Style::default().bg(palette.bg).fg(palette.fg)),
        area,
    );

    if area.width < 48 || area.height < 14 {
        draw_too_small(f.buffer_mut(), area, &palette);
        return;
    }

    let compact = area.height < 21;
    let header_h = if compact { 2 } else { 3 };

    draw_header(
        f.buffer_mut(),
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: header_h,
        },
        app,
        &palette,
    );

    let body = Rect {
        x: area.x,
        y: area.y + header_h,
        width: area.width,
        height: area.height.saturating_sub(header_h + 1),
    };
    draw_body(f.buffer_mut(), body, app, &palette, compact);

    draw_footer(
        f.buffer_mut(),
        Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        },
        app,
        &palette,
    );

    if app.help {
        draw_help(f, area, &palette);
    }
}

fn draw_too_small(buf: &mut Buffer, area: Rect, p: &Palette) {
    let msg = "snaptop needs at least 48 × 14";
    let x = area.x + area.width.saturating_sub(msg.chars().count() as u16) / 2;
    let y = area.y + area.height / 2;
    buf.set_string(x, y, msg, Style::default().fg(p.dim).bg(p.bg));
}

fn draw_header(buf: &mut Buffer, area: Rect, app: &App, p: &Palette) {
    let y = area.y;
    let wordmark = "snaptop";
    buf.set_string(
        area.x,
        y,
        wordmark,
        Style::default()
            .fg(p.cpu)
            .bg(p.bg)
            .add_modifier(Modifier::BOLD),
    );
    buf.set_string(
        area.x + wordmark.len() as u16 + 1,
        y,
        concat!("v", env!("CARGO_PKG_VERSION")),
        Style::default().fg(p.faint).bg(p.bg),
    );

    let right = fmt::truncate(&app.device.cpu_name, (area.width / 2) as usize);
    put_right(buf, area, y, &right, Style::default().fg(p.fg).bg(p.bg));

    let mut detail = format!(
        "{} · build {} · {}",
        app.device.os_name,
        app.device.os_build,
        app.device.core_summary()
    );
    if app.paused {
        detail.push_str("  ·  PAUSED");
    }
    buf.set_string(
        area.x,
        y + 1,
        fmt::truncate(&detail, area.width as usize),
        Style::default()
            .fg(if app.paused { p.warn } else { p.dim })
            .bg(p.bg),
    );

    let cadence = if app.paused {
        format!("{:.1}s", app.interval_ms as f64 / 1000.0)
    } else if app.stale() {
        "stale".to_string()
    } else {
        format!("{:.1}s", app.interval_ms as f64 / 1000.0)
    };
    put_right(
        buf,
        area,
        y + 1,
        &cadence,
        Style::default()
            .fg(if app.stale() && !app.paused {
                p.warn
            } else {
                p.faint
            })
            .bg(p.bg),
    );
}

fn draw_body(buf: &mut Buffer, body: Rect, app: &App, p: &Palette, compact: bool) {
    let gap = if compact { 0 } else { 1 };
    let mem_h = if compact { 1 } else { 2 };
    let spark_h = if compact {
        1
    } else {
        ((body.height.saturating_sub(11)) / 3).clamp(1, 4)
    };
    let accel_h = 2 + spark_h;

    let mut y = body.y;
    let snap = &app.snapshot;

    // CPU
    let cpu_right = format!(
        "{} · {}",
        fmt::mhz(Some(snap.cpu.cur_mhz)),
        fmt::watts(snap.cpu.watts)
    );
    let cpu_sub = cpu_subline(app);
    draw_accel(
        buf,
        Rect {
            x: body.x,
            y,
            width: body.width,
            height: accel_h,
        },
        p,
        "CPU",
        p.cpu,
        snap.cpu.util,
        &cpu_right,
        &app.cpu.as_slice(),
        &cpu_sub,
        spark_h,
    );
    y += accel_h + gap;

    // GPU
    let gpu = snap.gpu();
    let gpu_util = gpu.map(|g| g.util).unwrap_or(0.0);
    let gpu_right = match gpu {
        Some(g) => {
            let clock = fmt::mhz_pair(g.cur_mhz, g.max_mhz);
            if g.watts.is_some() {
                format!("{} · {}", clock, fmt::watts(g.watts))
            } else {
                clock
            }
        }
        None => "no GPU".to_string(),
    };
    let gpu_sub = gpu_subline(app);
    draw_accel(
        buf,
        Rect {
            x: body.x,
            y,
            width: body.width,
            height: accel_h,
        },
        p,
        "GPU",
        p.gpu,
        gpu_util,
        &gpu_right,
        &app.gpu.as_slice(),
        &gpu_sub,
        spark_h,
    );
    y += accel_h + gap;

    // NPU
    let npu = snap.npu();
    let npu_util = npu.map(|n| n.util).unwrap_or(0.0);
    let npu_right = npu
        .map(|n| match n.watts {
            Some(w) => fmt::watts(Some(w)),
            None => {
                if n.util < 0.5 {
                    "idle".to_string()
                } else {
                    "no rail".to_string()
                }
            }
        })
        .unwrap_or_else(|| "not found".to_string());
    let npu_sub = npu_subline(app);
    draw_accel(
        buf,
        Rect {
            x: body.x,
            y,
            width: body.width,
            height: accel_h,
        },
        p,
        "NPU",
        p.npu,
        npu_util,
        &npu_right,
        &app.npu.as_slice(),
        &npu_sub,
        spark_h,
    );
    y += accel_h + gap;

    // Memory
    let mem_pct = snap.mem.pct();
    let mem_sub = fmt::gib_pair(snap.mem.used, snap.mem.total);
    draw_mem(
        buf,
        Rect {
            x: body.x,
            y,
            width: body.width,
            height: mem_h,
        },
        p,
        mem_pct,
        &app.mem.as_slice(),
        &mem_sub,
        compact,
    );
}

fn cpu_subline(app: &App) -> String {
    let cpu = &app.snapshot.cpu;
    let mut parts = Vec::new();
    if let Some(v) = cpu.util_of(CoreKind::Performance) {
        parts.push(format!(
            "P {}× {}",
            cpu.count(CoreKind::Performance),
            fmt::pct(v)
        ));
    }
    if let Some(v) = cpu.util_of(CoreKind::Efficiency) {
        parts.push(format!(
            "E {}× {}",
            cpu.count(CoreKind::Efficiency),
            fmt::pct(v)
        ));
    }
    let mut line = parts.join("   ");
    if !line.is_empty() {
        line.push_str("   ");
    }
    line.push_str(&core_bars(cpu, 24));
    line
}

fn gpu_subline(app: &App) -> String {
    let Some(gpu) = app.snapshot.gpu() else {
        return String::new();
    };
    let mut parts = vec![gpu.name.clone()];
    if let (Some(used), Some(total)) = (gpu.mem_used, gpu.mem_total) {
        parts.push(format!(
            "{:.1}/{:.1} GiB",
            used as f64 / 1073741824.0,
            total as f64 / 1073741824.0
        ));
    }
    let engines: Vec<String> = gpu
        .engines
        .iter()
        .filter(|e| e.util >= 0.5 || e.name == "3D")
        .take(3)
        .map(|e| format!("{} {}", e.name, fmt::pct(e.util)))
        .collect();
    if !engines.is_empty() {
        parts.push(engines.join(" · "));
    }
    parts.join(" · ")
}

fn npu_subline(app: &App) -> String {
    let Some(npu) = app.snapshot.npu() else {
        return "no compute accelerator found".to_string();
    };
    let mut parts = vec![npu.name.clone()];
    if let Some(engine) = npu.engines.first() {
        parts.push(format!("engine {} {}", engine.name, fmt::pct(engine.util)));
    }
    parts.join(" · ")
}

/// 12 cores in 24 columns: one eight-level block per core, grouped P | E.
fn core_bars(cpu: &crate::model::Cpu, max: usize) -> String {
    const LEVELS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let mut out = String::new();
    let mut last_kind = None;
    for core in cpu.cores.iter().take(max) {
        if let Some(prev) = last_kind
            && prev != core.kind
            && !out.is_empty()
        {
            out.push(' ');
        }
        last_kind = Some(core.kind);
        let level = ((core.util / 100.0) * 8.0).round() as usize;
        let ch = if core.util < 1.0 {
            '·'
        } else {
            LEVELS[level.clamp(1, 8) - 1]
        };
        out.push(ch);
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn draw_accel(
    buf: &mut Buffer,
    rect: Rect,
    p: &Palette,
    label: &str,
    accent: Color,
    value: f32,
    right: &str,
    spark: &[f32],
    subline: &str,
    spark_h: u16,
) {
    if rect.height < 2 {
        return;
    }
    let y = rect.y;
    let width = rect.width as usize;

    buf.set_string(
        rect.x,
        y,
        label,
        Style::default()
            .fg(accent)
            .bg(p.bg)
            .add_modifier(Modifier::BOLD),
    );

    let pct_text = format!("{:>3.0}%", value.clamp(0.0, 100.0));
    buf.set_string(
        rect.x + 4,
        y,
        &pct_text,
        Style::default()
            .fg(theme::heat(p, value))
            .bg(p.bg)
            .add_modifier(Modifier::BOLD),
    );

    let right = fmt::truncate(right, (width / 3).clamp(6, 24));
    let detail_w = (width / 3).clamp(6, 24);
    let meter_x = rect.x + 9;
    let meter_w = width.saturating_sub(9 + detail_w + 2);
    if meter_w >= 4 {
        draw_meter(buf, meter_x, y, meter_w as u16, value, accent, p);
    }
    put_right(buf, rect, y, &right, Style::default().fg(p.dim).bg(p.bg));

    if spark_h > 0 {
        let spark_rect = Rect {
            x: rect.x,
            y: y + 1,
            width: rect.width,
            height: spark_h,
        };
        draw_sparkline(buf, spark_rect, spark, accent, p);
    }

    let sub_y = y + 1 + spark_h;
    if sub_y < rect.y + rect.height {
        buf.set_string(
            rect.x,
            sub_y,
            fmt::truncate(subline, width),
            Style::default().fg(p.dim).bg(p.bg),
        );
    }
}

fn draw_mem(
    buf: &mut Buffer,
    rect: Rect,
    p: &Palette,
    value: f32,
    spark: &[f32],
    subline: &str,
    compact: bool,
) {
    if rect.height == 0 {
        return;
    }
    let y = rect.y;
    let width = rect.width as usize;
    buf.set_string(
        rect.x,
        y,
        "MEM",
        Style::default()
            .fg(p.mem)
            .bg(p.bg)
            .add_modifier(Modifier::BOLD),
    );
    let pct_text = format!("{:>3.0}%", value.clamp(0.0, 100.0));
    buf.set_string(
        rect.x + 4,
        y,
        &pct_text,
        Style::default()
            .fg(theme::heat(p, value))
            .bg(p.bg)
            .add_modifier(Modifier::BOLD),
    );

    let detail_w = (width / 3).clamp(6, 24);
    let meter_x = rect.x + 9;
    let meter_w = width.saturating_sub(9 + detail_w + 2);
    if meter_w >= 4 {
        draw_meter(buf, meter_x, y, meter_w as u16, value, p.mem, p);
    }
    put_right(
        buf,
        rect,
        y,
        &fmt::truncate(subline, detail_w),
        Style::default().fg(p.dim).bg(p.bg),
    );

    if !compact && rect.height > 1 {
        draw_sparkline(
            buf,
            Rect {
                x: rect.x,
                y: y + 1,
                width: rect.width,
                height: 1,
            },
            spark,
            p.mem,
            p,
        );
    }
}

fn draw_meter(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    value: f32,
    accent: Color,
    p: &Palette,
) {
    let width = width as usize;
    let filled = ((value.clamp(0.0, 100.0) / 100.0) * width as f32).round() as usize;
    for i in 0..width {
        let cell = &mut buf[(x + i as u16, y)];
        if i < filled {
            cell.set_char('█');
            cell.set_fg(theme::gradient(accent, i as f32 / width.max(1) as f32, p));
        } else {
            cell.set_char('░');
            cell.set_fg(p.faint);
        }
        cell.set_bg(p.bg);
    }
}

/// Braille sparkline: two sample columns and four dot rows per character.
fn draw_sparkline(buf: &mut Buffer, rect: Rect, data: &[f32], accent: Color, p: &Palette) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    const LEFT: [u16; 4] = [0x01, 0x02, 0x04, 0x08];
    const RIGHT: [u16; 4] = [0x10, 0x20, 0x40, 0x80];

    let cols = rect.width as usize;
    let rows = rect.height as usize;
    let dots = rows * 4;
    let want = cols * 2;

    let mut samples = vec![0.0f32; want];
    if !data.is_empty() {
        let take = data.len().min(want);
        let tail = &data[data.len() - take..];
        let offset = want - take;
        // Before the trace has filled the width, hold the oldest sample so the
        // line starts flat rather than in a misleading zero corner.
        for value in samples[..offset].iter_mut() {
            *value = tail[0];
        }
        samples[offset..].copy_from_slice(tail);
    }

    let color = match accent {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * 0.82) as u8,
            (g as f32 * 0.82) as u8,
            (b as f32 * 0.82) as u8,
        ),
        other => other,
    };

    for cx in 0..cols {
        for cy in 0..rows {
            let mut bits = 0u16;
            for dx in 0..2 {
                let value = samples[(cx * 2 + dx).min(want - 1)].clamp(0.0, 100.0);
                let lit = ((value / 100.0) * dots as f32).round() as usize;
                for g in 0..dots {
                    if dots - g > lit {
                        continue;
                    }
                    if g / 4 != cy {
                        continue;
                    }
                    let within = g % 4;
                    bits |= if dx == 0 { LEFT[within] } else { RIGHT[within] };
                }
            }
            let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');
            let cell = &mut buf[(rect.x + cx as u16, rect.y + cy as u16)];
            cell.set_char(ch);
            cell.set_fg(if bits == 0 { p.faint } else { color });
            cell.set_bg(p.bg);
        }
    }
}

fn draw_footer(buf: &mut Buffer, area: Rect, app: &App, p: &Palette) {
    let power = &app.snapshot.power;
    let mut spans: Vec<Span> = Vec::new();

    let mut push = |label: &str, value: String, color: Color| {
        if !spans.is_empty() {
            spans.push(Span::styled("  ·  ", Style::default().fg(p.faint)));
        }
        spans.push(Span::styled(
            format!("{label} "),
            Style::default().fg(p.dim),
        ));
        spans.push(Span::styled(value, Style::default().fg(color)));
    };

    if let Some(w) = power.sys_w {
        push("SYS", fmt::watts(Some(w)), p.fg);
    }
    if let Some(t) = app.snapshot.thermal_c {
        push(
            "SoC",
            fmt::temp(Some(t)),
            if t > 85.0 { p.bad } else { p.fg },
        );
    }
    if let Some(b) = power.battery_pct {
        let suffix = match power.ac {
            Some(true) => " AC",
            Some(false) => "",
            None => "",
        };
        push("BAT", format!("{b}%{suffix}"), p.fg);
    }

    let right = if app.help {
        "? close".to_string()
    } else {
        "q quit · space pause · ? help".to_string()
    };
    let right_w = right.chars().count() as u16;
    let left_w = area.width.saturating_sub(right_w + 2) as usize;

    // Paint spans manually so the right-hand hint always survives.
    let mut x = area.x;
    for span in spans {
        let used = (x - area.x) as usize;
        let content = fmt::truncate(span.content.as_ref(), left_w.saturating_sub(used));
        if content.is_empty() {
            break;
        }
        buf.set_string(x, area.y, &content, span.style.bg(p.bg));
        x += content.chars().count() as u16;
        if x >= area.x + left_w as u16 {
            break;
        }
    }
    put_right(
        buf,
        area,
        area.y,
        &right,
        Style::default().fg(p.faint).bg(p.bg),
    );
}

fn draw_help(f: &mut Frame, area: Rect, p: &Palette) {
    let width = 48.min(area.width);
    let height = 9.min(area.height);
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let rect = Rect {
        x,
        y,
        width,
        height,
    };

    f.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(p.cpu).bg(p.bg))
        .title(Span::styled(
            " keys ",
            Style::default().fg(p.cpu).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(p.bg));
    f.render_widget(block, rect);

    let rows = [
        ("q / esc", "quit"),
        ("space", "pause / resume"),
        ("+ / −", "slower / faster"),
        ("r", "reset history"),
        ("t", "cycle theme"),
        ("?", "close this"),
    ];
    let mut lines = Vec::new();
    for (key, action) in rows {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<8}"), Style::default().fg(p.cpu)),
            Span::styled(action, Style::default().fg(p.fg)),
        ]));
    }
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };
    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(p.bg)),
        inner,
    );
}

fn put_right(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    let len = text.chars().count() as u16;
    if len > area.width {
        return;
    }
    let x = area.x + area.width - len;
    buf.set_string(x, y, text, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{self, DeviceInfo, Engine, Gpu, Mem, Npu, Power};

    fn demo() -> (DeviceInfo, crate::model::Snapshot) {
        let device = DeviceInfo {
            cpu_name: "Snapdragon X Elite X1E78100".into(),
            os_name: "Windows 11 Enterprise".into(),
            os_build: "26100".into(),
            cpu_count: 12,
            p_cores: 8,
            e_cores: 4,
            gpu_name: "Adreno X1-85".into(),
            npu_name: "Hexagon NPU".into(),
        };
        let mut cpu = model::Cpu {
            util: 31.0,
            cur_mhz: 3418,
            max_mhz: 3418,
            watts: Some(16.8),
            ..Default::default()
        };
        for i in 0..12 {
            let p = i < 8;
            cpu.cores.push(model::Core {
                id: i,
                kind: if p {
                    CoreKind::Performance
                } else {
                    CoreKind::Efficiency
                },
                util: if p { 18.0 } else { 57.0 },
                cur_mhz: 3418,
                max_mhz: 3418,
            });
        }
        let snapshot = crate::model::Snapshot {
            cpu,
            gpus: vec![Gpu {
                name: "Adreno X1-85".into(),
                util: 14.0,
                engines: vec![
                    Engine {
                        name: "3D".into(),
                        util: 13.6,
                    },
                    Engine {
                        name: "Compute".into(),
                        util: 0.2,
                    },
                ],
                cur_mhz: Some(720),
                max_mhz: Some(1443),
                mem_used: Some(880 << 20),
                mem_total: Some(16287 << 20),
                watts: Some(0.1),
            }],
            npus: vec![Npu {
                name: "Hexagon NPU".into(),
                util: 0.0,
                engines: vec![Engine {
                    name: "Compute".into(),
                    util: 0.0,
                }],
                watts: None,
            }],
            mem: Mem {
                used: 20 << 30,
                total: 32 << 30,
            },
            power: Power {
                sys_w: Some(33.2),
                psu_w: Some(33.2),
                cpu_w: Some(16.8),
                gpu_w: Some(0.1),
                battery_pct: Some(87),
                ac: Some(true),
            },
            thermal_c: Some(71.0),
        };
        (device, snapshot)
    }

    #[test]
    fn draws_a_full_frame() {
        let (device, snapshot) = demo();
        let backend = ratatui::backend::TestBackend::new(100, 28);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut app = App::new(device, 1000, 0);
        app.update(snapshot);
        terminal.draw(|frame| draw(frame, &app)).unwrap();

        let buffer = terminal.backend().buffer();
        let mut plain = String::new();
        for y in 0..28 {
            for x in 0..100 {
                plain.push_str(buffer[(x, y)].symbol());
            }
            plain.push('\n');
        }
        println!("{plain}");
        assert!(plain.contains("Snapdragon X Elite X1E78100"));
        assert!(plain.contains("Hexagon NPU"));
        assert!(plain.contains("31%"));
        assert!(plain.contains("33.2 W"));
    }

    #[test]
    fn survives_a_tiny_terminal() {
        let (device, snapshot) = demo();
        let backend = ratatui::backend::TestBackend::new(40, 10);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut app = App::new(device, 1000, 0);
        app.update(snapshot);
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }
}
