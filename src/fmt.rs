//! Small, dependency-free formatting helpers. Everything is tuned for the
//! narrow columns of a terminal, so values stay short and stable-width.

pub fn pct(v: f32) -> String {
    format!("{:.0}%", v.clamp(0.0, 999.0))
}

pub fn watts(v: Option<f32>) -> String {
    match v {
        Some(w) if w >= 0.0 => {
            if w >= 100.0 {
                format!("{w:.0} W")
            } else {
                format!("{w:.1} W")
            }
        }
        _ => "—".to_string(),
    }
}

pub fn mhz(v: Option<u32>) -> String {
    match v {
        Some(m) if m >= 1000 => format!("{:.2} GHz", m as f32 / 1000.0),
        Some(m) if m > 0 => format!("{m} MHz"),
        _ => "—".to_string(),
    }
}

pub fn mhz_pair(cur: Option<u32>, max: Option<u32>) -> String {
    match (cur, max) {
        (Some(c), Some(m)) if c > 0 && m > 0 => {
            if m >= 1000 {
                format!("{:.2}/{:.2} GHz", c as f32 / 1000.0, m as f32 / 1000.0)
            } else {
                format!("{c}/{m} MHz")
            }
        }
        (Some(c), _) if c > 0 => mhz(Some(c)),
        _ => "—".to_string(),
    }
}

pub fn gib_pair(used: u64, total: u64) -> String {
    format!(
        "{:.1} / {:.1} GiB",
        used as f64 / 1073741824.0,
        total as f64 / 1073741824.0
    )
}

pub fn temp(c: Option<f32>) -> String {
    match c {
        Some(c) => format!("{c:.0} °C"),
        None => "—".to_string(),
    }
}

/// Fit `s` into `width` columns, marking a cut with an ellipsis.
pub fn truncate(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if s.chars().count() <= width {
        return s.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }
    let mut out: String = s.chars().take(width - 1).collect();
    out.push('…');
    out
}

/// Snapdragon marketing strings carry a lot of noise. Keep the part a human
/// would actually say: "Snapdragon X Elite X1E78100".
pub fn pretty_cpu_name(raw: &str) -> String {
    let cleaned = raw
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("(C)", "")
        .replace("CPU", "");
    let mut head = cleaned
        .split(" - Qualcomm")
        .next()
        .unwrap_or(&cleaned)
        .to_string();
    head = head.replace(" - ", " ");
    head.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Hexagon(TM) NPU" -> "Hexagon NPU"
pub fn pretty_npu_name(raw: &str) -> String {
    let cleaned = raw
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("(C)", "");
    let tail = cleaned.rsplit(" - ").next().unwrap_or(&cleaned);
    let mut name = tail
        .trim()
        .strip_prefix("Qualcomm ")
        .unwrap_or(tail.trim())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if !name.to_ascii_uppercase().contains("NPU") {
        name.push_str(" NPU");
    }
    name
}

/// "Qualcomm(R) Adreno(TM) X1-85 GPU" -> "Adreno X1-85"; other vendors keep
/// their own name.
pub fn pretty_gpu_name(raw: &str) -> String {
    let cleaned = raw
        .replace("Qualcomm(R)", "")
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("(C)", "");
    let mut name = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    let lower = name.to_ascii_lowercase();
    if let Some(at) = lower.find("adreno") {
        let rest = name[at + "adreno".len()..].trim();
        let rest = rest.trim_end_matches(" GPU").trim();
        return if rest.is_empty() {
            "Adreno GPU".to_string()
        } else {
            format!("Adreno {rest}")
        };
    }

    name = name.trim_end_matches(" GPU").trim().to_string();
    if name.is_empty() {
        "GPU".to_string()
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_names_are_tidy() {
        let raw = "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Oryon(TM) CPU";
        assert_eq!(pretty_cpu_name(raw), "Snapdragon X Elite X1E78100");
    }

    #[test]
    fn npu_names_are_tidy() {
        let raw = "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Hexagon(TM) NPU";
        assert_eq!(pretty_npu_name(raw), "Hexagon NPU");
    }

    #[test]
    fn gpu_names_are_tidy() {
        assert_eq!(
            pretty_gpu_name("Qualcomm(R) Adreno(TM) X1-85 GPU"),
            "Adreno X1-85"
        );
        assert_eq!(
            pretty_gpu_name("NVIDIA GeForce RTX 4090"),
            "NVIDIA GeForce RTX 4090"
        );
        assert_eq!(
            pretty_gpu_name("Intel(R) Arc(TM) Graphics GPU"),
            "Intel Arc Graphics"
        );
    }

    #[test]
    fn truncation_marks_the_cut() {
        assert_eq!(truncate("Hexagon NPU", 8), "Hexagon…");
        assert_eq!(truncate("short", 8), "short");
    }
}
