//! Rendering: mactop-style panels, meters, per-core bars and braille history.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget};

use crate::app::App;
use crate::fmt;
use crate::model::{CoreKind, Gpu, Npu};
use crate::theme::{self, Palette};

pub fn draw(f: &mut Frame, app: &App) {
    let outer = f.area();
    let p = theme::palette(app.theme);

    let frame = Block::bordered()
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(p.border).bg(p.bg))
        .title(Line::from(vec![
            Span::styled(
                " snaptop ",
                Style::default().fg(p.cpu).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                concat!("v", env!("CARGO_PKG_VERSION"), " "),
                Style::default().fg(p.faint),
            ),
        ]))
        .style(Style::default().bg(p.bg).fg(p.fg));

    if outer.width < 62 || outer.height < 20 {
        draw_too_small(f.buffer_mut(), outer, &p);
        return;
    }

    let area = frame.inner(outer);
    frame.render(outer, f.buffer_mut());

    let header = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    let footer = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    let body = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height - 2,
    };

    draw_header(f.buffer_mut(), header, app, &p);
    if area.width >= 94 && body.height >= 18 {
        draw_grid(f.buffer_mut(), body, app, &p);
    } else {
        draw_stack(f.buffer_mut(), body, app, &p);
    }
    draw_footer(f.buffer_mut(), footer, app, &p);

    if app.help {
        draw_help(f, outer, &p);
    }
}

fn draw_too_small(buf: &mut Buffer, area: Rect, p: &Palette) {
    let msg = "snaptop needs 62 × 20";
    let x = area.x + area.width.saturating_sub(msg.chars().count() as u16) / 2;
    let y = area.y + area.height / 2;
    buf.set_string(x, y, msg, Style::default().fg(p.dim).bg(p.bg));
}

fn draw_header(buf: &mut Buffer, area: Rect, app: &App, p: &Palette) {
    let right = format!("{} · {}", app.device.os_name, app.device.core_summary());
    let right_w = right.chars().count() as u16;
    let left_max = area.width.saturating_sub(right_w + 2);
    buf.set_string(
        area.x,
        area.y,
        fmt::truncate(&app.device.cpu_name, left_max as usize),
        Style::default()
            .fg(p.fg)
            .bg(p.bg)
            .add_modifier(Modifier::BOLD),
    );
    put_right(
        buf,
        area,
        area.y,
        &right,
        Style::default().fg(p.dim).bg(p.bg),
    );
}

fn draw_footer(buf: &mut Buffer, area: Rect, app: &App, p: &Palette) {
    let right = format!(
        "{} · {:.1}s",
        theme::name(app.theme),
        app.interval_ms as f64 / 1000.0
    );
    let right_len = right.chars().count() as u16;
    let left_width = area.width.saturating_sub(right_len + 2);

    let (left, color) = if app.paused {
        ("PAUSED", p.warn)
    } else if app.stale() {
        ("STALE", p.warn)
    } else {
        ("q quit · space pause · t theme · ? help", p.dim)
    };
    buf.set_string(
        area.x,
        area.y,
        fmt::truncate(left, left_width as usize),
        Style::default().fg(color).bg(p.bg),
    );

    put_right(
        buf,
        area,
        area.y,
        &right,
        Style::default().fg(p.faint).bg(p.bg),
    );
}

fn draw_grid(buf: &mut Buffer, body: Rect, app: &App, p: &Palette) {
    let rows = Layout::vertical([Constraint::Min(10), Constraint::Length(8)]).split(body);
    let top =
        Layout::horizontal([Constraint::Percentage(64), Constraint::Percentage(36)]).split(rows[0]);
    let right =
        Layout::vertical([Constraint::Percentage(56), Constraint::Percentage(44)]).split(top[1]);
    let bottom =
        Layout::horizontal([Constraint::Percentage(56), Constraint::Percentage(44)]).split(rows[1]);

    draw_cpu(buf, top[0], app, p);
    draw_gpu(buf, right[0], app, p);
    draw_npu(buf, right[1], app, p);
    draw_mem(buf, bottom[0], app, p);
    draw_power(buf, bottom[1], app, p);
}

fn draw_stack(buf: &mut Buffer, body: Rect, app: &App, p: &Palette) {
    let count = app.snapshot.cpu.cores.len().max(1) as u16;
    let bar = if body.width >= 80 { 10u16 } else { 8 };
    let cell = bar + 13;
    let max_cols = (body.width.saturating_sub(2) / cell).max(1);
    let mut rows = count.div_ceil(max_cols).max(1);

    let mut cpu_h = (rows + 3).clamp(5, 9);
    let mut gpu_h = 4u16;
    let mut npu_h = 3u16;
    let mut bottom_h = 4u16;
    while cpu_h + gpu_h + npu_h + bottom_h > body.height {
        if bottom_h > 3 {
            bottom_h -= 1;
        } else if gpu_h > 3 {
            gpu_h -= 1;
        } else if npu_h > 2 {
            npu_h -= 1;
        } else if rows > 1 && cpu_h > 5 {
            rows -= 1;
            cpu_h -= 1;
        } else {
            break;
        }
    }

    // Use any leftover rows for the panels that were squeezed first.
    let mut spare = body.height.saturating_sub(cpu_h + gpu_h + npu_h + bottom_h);
    while spare > 0 {
        if bottom_h < 5 {
            bottom_h += 1;
        } else if gpu_h < 5 {
            gpu_h += 1;
        } else if npu_h < 4 {
            npu_h += 1;
        } else if cpu_h < 9 {
            cpu_h += 1;
        } else {
            break;
        }
        spare -= 1;
    }

    let chunks = Layout::vertical([
        Constraint::Length(cpu_h),
        Constraint::Length(gpu_h),
        Constraint::Length(npu_h),
        Constraint::Length(bottom_h),
        Constraint::Min(0),
    ])
    .split(body);
    draw_cpu(buf, chunks[0], app, p);
    draw_gpu(buf, chunks[1], app, p);
    draw_npu(buf, chunks[2], app, p);

    let bottom = Layout::horizontal([Constraint::Percentage(56), Constraint::Percentage(44)])
        .split(chunks[3]);
    draw_mem(buf, bottom[0], app, p);
    draw_power(buf, bottom[1], app, p);
}

fn panel(
    buf: &mut Buffer,
    rect: Rect,
    p: &Palette,
    accent: Color,
    title: &str,
    detail: &str,
) -> Option<Rect> {
    if rect.width < 12 || rect.height < 3 {
        return None;
    }
    let block = Block::bordered()
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(p.border).bg(p.bg))
        .title(Line::from(vec![
            Span::styled(
                format!("-{title} "),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{detail} "), Style::default().fg(p.dim)),
        ]))
        .style(Style::default().bg(p.bg));
    let inner = block.inner(rect);
    block.render(rect, buf);
    Some(inner)
}

fn draw_cpu(buf: &mut Buffer, rect: Rect, app: &App, p: &Palette) {
    let cpu = &app.snapshot.cpu;
    let detail = format!("{} · {} ", app.device.core_summary(), fmt::pct(cpu.util));
    let Some(inner) = panel(buf, rect, p, p.cpu, "CPU", &detail) else {
        return;
    };
    if inner.height == 0 {
        return;
    }

    let mut left = String::new();
    if let Some(value) = cpu.util_of(CoreKind::Performance) {
        left.push_str(&format!(
            "P {}× {}   ",
            cpu.count(CoreKind::Performance),
            fmt::pct(value)
        ));
    }
    if let Some(value) = cpu.util_of(CoreKind::Efficiency) {
        left.push_str(&format!(
            "E {}× {}",
            cpu.count(CoreKind::Efficiency),
            fmt::pct(value)
        ));
    }
    buf.set_string(
        inner.x,
        inner.y,
        fmt::truncate(&left, inner.width as usize),
        Style::default().fg(p.fg).bg(p.bg),
    );
    let right = format!(
        "{} · {}",
        fmt::mhz(Some(cpu.cur_mhz)),
        fmt::watts(cpu.watts)
    );
    put_right(
        buf,
        inner,
        inner.y,
        &right,
        Style::default().fg(p.dim).bg(p.bg),
    );

    let rows_available = inner.height.saturating_sub(1);
    if rows_available == 0 {
        return;
    }

    let bar = if inner.width >= 64 { 8 } else { 6 };
    let cell = bar + 13;
    let max_cols = (inner.width / cell).clamp(1, 4);
    let mut rows = (cpu.cores.len() as u16).div_ceil(max_cols).max(1);
    rows = rows.min(rows_available);
    let cols = (cpu.cores.len() as u16).div_ceil(rows.max(1)).min(max_cols);

    for (index, core) in cpu.cores.iter().enumerate() {
        let index = index as u16;
        let col = index / rows;
        let row = index % rows;
        if col >= cols {
            break;
        }
        draw_core(buf, inner.x + col * cell, inner.y + 1 + row, core, bar, p);
    }

    let used = 1 + rows.min(rows_available);
    let spark_h = inner.height.saturating_sub(used).min(2);
    if spark_h > 0 {
        draw_sparkline(
            buf,
            Rect {
                x: inner.x,
                y: inner.y + used,
                width: inner.width,
                height: spark_h,
            },
            &app.cpu.as_slice(),
            100.0,
            p.cpu,
            p,
        );
    }
}

fn draw_core(buf: &mut Buffer, x: u16, y: u16, core: &crate::model::Core, bar: u16, p: &Palette) {
    if x + bar + 12 > buf.area.right() || y >= buf.area.bottom() {
        return;
    }
    buf.set_string(
        x,
        y,
        format!("{:>2} [", core.id),
        Style::default().fg(p.dim).bg(p.bg),
    );
    let filled = ((core.util / 100.0) * bar as f32).round() as u16;
    for i in 0..bar {
        let cell = &mut buf[(x + 4 + i, y)];
        if i < filled {
            cell.set_char('█');
            cell.set_fg(core_color(core.util, p));
        } else {
            cell.set_char('░');
            cell.set_fg(p.faint);
        }
        cell.set_bg(p.bg);
    }
    buf.set_string(
        x + 4 + bar,
        y,
        format!("] {:>5.1}%", core.util),
        Style::default().fg(p.fg).bg(p.bg),
    );
}

fn core_color(value: f32, p: &Palette) -> Color {
    if value >= 90.0 {
        p.bad
    } else if value >= 70.0 {
        p.warn
    } else {
        p.cpu
    }
}
fn draw_gpu(buf: &mut Buffer, rect: Rect, app: &App, p: &Palette) {
    let Some(gpu) = app.snapshot.gpu() else {
        panel(buf, rect, p, p.gpu, "GPU", "not found ");
        return;
    };
    let detail = format!("{} · {} ", gpu.name, fmt::pct(gpu.util));
    let Some(inner) = panel(buf, rect, p, p.gpu, "GPU", &detail) else {
        return;
    };
    draw_accelerator(buf, inner, gpu, p, p.gpu, &app.gpu.as_slice());
}

fn draw_npu(buf: &mut Buffer, rect: Rect, app: &App, p: &Palette) {
    let Some(npu) = app.snapshot.npu() else {
        panel(buf, rect, p, p.npu, "NPU", "not found ");
        return;
    };
    let detail = format!(
        "{} · {} ",
        npu.name.trim_end_matches(" NPU"),
        fmt::pct(npu.util)
    );
    let Some(inner) = panel(buf, rect, p, p.npu, "NPU", &detail) else {
        return;
    };
    draw_accelerator(buf, inner, npu, p, p.npu, &app.npu.as_slice());
}

fn draw_accelerator<T: Gauge>(
    buf: &mut Buffer,
    inner: Rect,
    device: &T,
    p: &Palette,
    accent: Color,
    history: &[f32],
) {
    if inner.height == 0 || inner.width < 12 {
        return;
    }

    if inner.height >= 5 && inner.width >= 28 {
        let gauge_w = 6.min(inner.width / 5).max(3);
        draw_vmeter(
            buf,
            Rect {
                x: inner.x,
                y: inner.y,
                width: gauge_w,
                height: inner.height,
            },
            device.util(),
            accent,
            p,
        );

        let text = Rect {
            x: inner.x + gauge_w + 2,
            y: inner.y,
            width: inner.width.saturating_sub(gauge_w + 2),
            height: inner.height,
        };
        draw_big_pct(buf, text.x, text.y, device.util(), p, accent);
        put_right(
            buf,
            text,
            text.y,
            &device.summary(),
            Style::default().fg(p.dim).bg(p.bg),
        );
        if text.height > 1 {
            buf.set_string(
                text.x,
                text.y + 1,
                fmt::truncate(&device.details(), text.width as usize),
                Style::default().fg(p.dim).bg(p.bg),
            );
        }
        let spark_h = text.height.saturating_sub(2).min(2);
        if spark_h > 0 {
            draw_sparkline(
                buf,
                Rect {
                    x: text.x,
                    y: text.y + text.height - spark_h,
                    width: text.width,
                    height: spark_h,
                },
                history,
                100.0,
                accent,
                p,
            );
        }
        return;
    }

    draw_big_pct(buf, inner.x, inner.y, device.util(), p, accent);
    put_right(
        buf,
        inner,
        inner.y,
        &device.summary(),
        Style::default().fg(p.dim).bg(p.bg),
    );

    if inner.height > 1 {
        draw_meter(
            buf,
            inner.x,
            inner.y + 1,
            inner.width,
            device.util(),
            accent,
            p,
        );
    }
    if inner.height > 2 {
        buf.set_string(
            inner.x,
            inner.y + 2,
            fmt::truncate(&device.details(), inner.width as usize),
            Style::default().fg(p.dim).bg(p.bg),
        );
    }

    let spark_h = inner.height.saturating_sub(3).min(2);
    if spark_h > 0 {
        draw_sparkline(
            buf,
            Rect {
                x: inner.x,
                y: inner.y + inner.height - spark_h,
                width: inner.width,
                height: spark_h,
            },
            history,
            100.0,
            accent,
            p,
        );
    }
}

trait Gauge {
    fn util(&self) -> f32;
    fn summary(&self) -> String;
    fn details(&self) -> String;
}

impl Gauge for Gpu {
    fn util(&self) -> f32 {
        self.util
    }
    fn summary(&self) -> String {
        fmt::mhz_pair(self.cur_mhz, self.max_mhz)
    }
    fn details(&self) -> String {
        let memory = match (self.mem_used, self.mem_total) {
            (Some(used), Some(total)) => format!(
                "{:.1}/{:.1} GiB · ",
                used as f64 / 1_073_741_824.0,
                total as f64 / 1_073_741_824.0
            ),
            _ => String::new(),
        };
        let engines = self
            .engines
            .iter()
            .filter(|e| e.util >= 0.5)
            .take(3)
            .map(|e| format!("{} {}", e.name, fmt::pct(e.util)))
            .collect::<Vec<_>>()
            .join(" · ");
        if engines.is_empty() {
            format!("{memory}idle")
        } else {
            format!("{memory}{engines}")
        }
    }
}

impl Gauge for Npu {
    fn util(&self) -> f32 {
        self.util
    }
    fn summary(&self) -> String {
        match self.watts {
            Some(watts) => fmt::watts(Some(watts)),
            None if self.util < 0.5 => "idle".to_string(),
            None => "no rail".to_string(),
        }
    }
    fn details(&self) -> String {
        self.engines
            .first()
            .map(|e| format!("engine {} {}", e.name, fmt::pct(e.util)))
            .unwrap_or_else(|| "no engine counter".to_string())
    }
}

fn draw_mem(buf: &mut Buffer, rect: Rect, app: &App, p: &Palette) {
    let mem = app.snapshot.mem;
    let detail = format!("{} ", fmt::gib_pair(mem.used, mem.total));
    let Some(inner) = panel(buf, rect, p, p.mem, "MEMORY", &detail) else {
        return;
    };
    if inner.height == 0 {
        return;
    }

    draw_big_pct(buf, inner.x, inner.y, mem.pct(), p, p.mem);
    put_right(
        buf,
        inner,
        inner.y,
        &format!("avail {}", fmt::gib(mem.available)),
        Style::default().fg(p.dim).bg(p.bg),
    );

    if inner.height > 1 {
        draw_meter(buf, inner.x, inner.y + 1, inner.width, mem.pct(), p.mem, p);
    }
    if inner.height > 2 && mem.commit_total > 0 {
        buf.set_string(
            inner.x,
            inner.y + 2,
            fmt::truncate(
                &format!(
                    "commit {} · {}",
                    fmt::gib_pair(mem.commit_used, mem.commit_total),
                    fmt::pct(mem.commit_pct())
                ),
                inner.width as usize,
            ),
            Style::default().fg(p.dim).bg(p.bg),
        );
    }

    let spark_h = inner.height.saturating_sub(3).min(2);
    if spark_h > 0 {
        draw_sparkline(
            buf,
            Rect {
                x: inner.x,
                y: inner.y + inner.height - spark_h,
                width: inner.width,
                height: spark_h,
            },
            &app.mem.as_slice(),
            100.0,
            p.mem,
            p,
        );
    }
}

fn draw_power(buf: &mut Buffer, rect: Rect, app: &App, p: &Palette) {
    let power = &app.snapshot.power;
    let detail = match power.sys_w {
        Some(watts) => format!("SYS {} ", fmt::watts(Some(watts))),
        None => "no rails ".to_string(),
    };
    let Some(inner) = panel(buf, rect, p, p.warn, "POWER", &detail) else {
        return;
    };
    if inner.height == 0 {
        return;
    }

    let rails = [
        ("CPU", power.cpu_w),
        ("GPU", power.gpu_w),
        ("PSU", power.psu_w),
    ]
    .into_iter()
    .filter_map(|(label, value)| value.map(|w| format!("{label} {}", fmt::watts(Some(w)))))
    .collect::<Vec<_>>()
    .join(" · ");
    buf.set_string(
        inner.x,
        inner.y,
        fmt::truncate(&rails, inner.width as usize),
        Style::default().fg(p.fg).bg(p.bg),
    );

    if inner.height > 1 {
        let mut second = Vec::new();
        if let Some(battery) = power.battery_pct {
            let ac = match power.ac {
                Some(true) => " AC",
                _ => "",
            };
            second.push(format!("BAT {battery}%{ac}"));
        }
        if let Some(temp) = app.snapshot.thermal_c {
            second.push(format!("SoC {}", fmt::temp(Some(temp))));
        }
        buf.set_string(
            inner.x,
            inner.y + 1,
            fmt::truncate(&second.join(" · "), inner.width as usize),
            Style::default().fg(p.dim).bg(p.bg),
        );
    }

    let bars_h = inner.height.saturating_sub(2);
    if bars_h > 0 {
        let peak = app.power.as_slice().iter().copied().fold(0.0f32, f32::max);
        draw_bars(
            buf,
            Rect {
                x: inner.x,
                y: inner.y + 2,
                width: inner.width,
                height: bars_h,
            },
            &app.power.as_slice(),
            (peak * 1.2).clamp(30.0, 150.0),
            p.warn,
            p,
        );
    }
}

fn draw_vmeter(buf: &mut Buffer, rect: Rect, value: f32, accent: Color, p: &Palette) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    let rows = rect.height as usize;
    let filled = ((value.clamp(0.0, 100.0) / 100.0) * rows as f32).round() as usize;
    for row in 0..rows {
        let lit = rows - row <= filled;
        for x in rect.x..rect.x + rect.width {
            let cell = &mut buf[(x, rect.y + row as u16)];
            if lit {
                cell.set_char('█');
                cell.set_fg(theme::gradient(accent, 1.0 - row as f32 / rows as f32, p));
            } else {
                cell.set_char('░');
                cell.set_fg(p.faint);
            }
            cell.set_bg(p.bg);
        }
    }
}

fn draw_bars(buf: &mut Buffer, rect: Rect, data: &[f32], max: f32, accent: Color, p: &Palette) {
    let cols = rect.width as usize;
    let rows = rect.height as usize;
    if cols == 0 || rows == 0 {
        return;
    }
    let max = max.max(1.0);
    let mut samples = vec![0.0f32; cols];
    if !data.is_empty() {
        let take = data.len().min(cols);
        let tail = &data[data.len() - take..];
        let offset = cols - take;
        for value in samples[..offset].iter_mut() {
            *value = tail[0];
        }
        samples[offset..].copy_from_slice(tail);
    }

    for (x, value) in samples.iter().enumerate() {
        let height = ((value / max).clamp(0.0, 1.0) * rows as f32).round() as usize;
        for row in 0..rows {
            let cell = &mut buf[(rect.x + x as u16, rect.y + row as u16)];
            if rows - row <= height {
                cell.set_char('█');
                cell.set_fg(accent);
            } else {
                cell.set_char(' ');
                cell.set_fg(p.faint);
            }
            cell.set_bg(p.bg);
        }
    }
}

fn draw_big_pct(buf: &mut Buffer, x: u16, y: u16, value: f32, p: &Palette, accent: Color) {
    buf.set_string(
        x,
        y,
        format!("{:.0}%", value.clamp(0.0, 999.0)),
        Style::default()
            .fg(accent)
            .bg(p.bg)
            .add_modifier(Modifier::BOLD),
    );
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
    if width == 0 {
        return;
    }
    let filled = ((value.clamp(0.0, 100.0) / 100.0) * width as f32).round() as usize;
    for i in 0..width {
        let cell = &mut buf[(x + i as u16, y)];
        if i < filled {
            cell.set_char('█');
            cell.set_fg(theme::gradient(accent, i as f32 / width as f32, p));
        } else {
            cell.set_char('░');
            cell.set_fg(p.faint);
        }
        cell.set_bg(p.bg);
    }
}

fn draw_sparkline(
    buf: &mut Buffer,
    rect: Rect,
    data: &[f32],
    max: f32,
    accent: Color,
    p: &Palette,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    const LEFT: [u16; 4] = [0x01, 0x02, 0x04, 0x08];
    const RIGHT: [u16; 4] = [0x10, 0x20, 0x40, 0x80];

    let cols = rect.width as usize;
    let rows = rect.height as usize;
    let dots = rows * 4;
    let want = cols;
    let max = max.max(1.0);

    let mut samples = vec![0.0f32; want];
    if !data.is_empty() {
        let take = data.len().min(want);
        let tail = &data[data.len() - take..];
        let offset = want - take;
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
                let value = samples[cx.min(want - 1)] / max;
                let lit = (value.clamp(0.0, 1.0) * dots as f32).round() as usize;
                for g in 0..dots {
                    if dots - g > lit || g / 4 != cy {
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

fn draw_help(f: &mut Frame, area: Rect, p: &Palette) {
    let width = 52.min(area.width);
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
        .border_type(BorderType::Plain)
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
    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(p.bg)),
        Rect {
            x: rect.x + 1,
            y: rect.y + 1,
            width: rect.width.saturating_sub(2),
            height: rect.height.saturating_sub(2),
        },
    );
}

fn put_right(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    let len = text.chars().count() as u16;
    if len > area.width {
        return;
    }
    buf.set_string(area.x + area.width - len, y, text, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{self, DeviceInfo, Engine, Mem, Power};

    fn demo() -> (DeviceInfo, crate::model::Snapshot) {
        let device = DeviceInfo {
            cpu_name: "Snapdragon X Elite X1E78100".into(),
            os_name: "Windows 11 Enterprise 24H2".into(),
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
            let performance = i < 8;
            cpu.cores.push(model::Core {
                id: i,
                kind: if performance {
                    CoreKind::Performance
                } else {
                    CoreKind::Efficiency
                },
                util: if performance { 18.0 } else { 57.0 },
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
                available: 12 << 30,
                commit_used: 22 << 30,
                commit_total: 40 << 30,
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

    fn render(width: u16, height: u16, theme_index: usize) -> String {
        let (device, snapshot) = demo();
        let backend = ratatui::backend::TestBackend::new(width, height);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut app = App::new(device, 1000, Some(theme_index));
        app.update(snapshot);
        terminal.draw(|frame| draw(frame, &app)).unwrap();

        let buffer = terminal.backend().buffer();
        let mut plain = String::new();
        for y in 0..height {
            for x in 0..width {
                plain.push_str(buffer[(x, y)].symbol());
            }
            plain.push('\n');
        }
        plain
    }

    #[test]
    fn wide_layout_shows_every_panel() {
        let frame = render(100, 30, 0);
        println!("{frame}");
        for panel in ["CPU", "GPU", "NPU", "MEMORY", "POWER"] {
            assert!(frame.contains(panel), "missing {panel}");
        }
        assert!(frame.contains("commit"));
        assert!(frame.contains("avail"));
        assert!(frame.contains("31%"));
    }

    #[test]
    fn light_theme_renders_the_same_panels() {
        let frame = render(100, 30, 1);
        for panel in ["CPU", "GPU", "NPU", "MEMORY", "POWER"] {
            assert!(frame.contains(panel), "missing {panel}");
        }
    }

    #[test]
    fn palettes_reach_the_buffer() {
        for (index, palette) in crate::theme::PALETTES.iter().enumerate() {
            let (device, snapshot) = demo();
            let backend = ratatui::backend::TestBackend::new(80, 24);
            let mut terminal = ratatui::Terminal::new(backend).unwrap();
            let mut app = App::new(device, 1000, Some(index));
            app.update(snapshot);
            terminal.draw(|frame| draw(frame, &app)).unwrap();
            assert_eq!(terminal.backend().buffer()[(1, 1)].bg, palette.bg);
        }
    }

    #[test]
    fn narrow_layout_still_fits() {
        let frame = render(70, 24, 0);
        println!("{frame}");
        assert!(frame.contains("MEMORY"));
    }

    #[test]
    fn tiny_terminal_shows_a_hint() {
        let frame = render(40, 12, 0);
        assert!(frame.contains("needs 62"));
    }
}
