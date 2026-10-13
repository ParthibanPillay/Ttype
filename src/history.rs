use std::{fs::OpenOptions, io::Write, time::SystemTime};

pub fn append(mode: &str, wpm: f64, raw: f64, acc: f64) -> std::io::Result<()> {
    let dir = dirs::data_dir().unwrap_or_default().join("ttype");
    std::fs::create_dir_all(&dir)?;
    let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let line = serde_json::json!({ "ts": ts, "mode": mode, "wpm": wpm, "raw": raw, "acc": acc });
    writeln!(OpenOptions::new().create(true).append(true).open(dir.join("history.jsonl"))?, "{line}")
}

#[derive(serde::Deserialize)]
pub struct Run {
    pub mode: String,
    pub wpm: f64,
    pub acc: f64,
}

/// All saved runs, oldest first.
pub fn load() -> Vec<Run> {
    let Some(p) = dirs::data_dir().map(|d| d.join("ttype/history.jsonl")) else { return vec![] };
    let s = std::fs::read_to_string(p).unwrap_or_default();
    s.lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
}

/// Highest wpm saved for this mode, if any.
pub fn best(mode: &str) -> Option<f64> {
    load().iter().filter(|r| r.mode == mode).map(|r| r.wpm).max_by(f64::total_cmp)
}
