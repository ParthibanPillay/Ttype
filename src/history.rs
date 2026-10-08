use std::{fs::OpenOptions, io::Write, time::SystemTime};

pub fn append(mode: &str, wpm: f64, raw: f64, acc: f64) -> std::io::Result<()> {
    let dir = dirs::data_dir().unwrap_or_default().join("ttest");
    std::fs::create_dir_all(&dir)?;
    let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let line = serde_json::json!({ "ts": ts, "mode": mode, "wpm": wpm, "raw": raw, "acc": acc });
    writeln!(OpenOptions::new().create(true).append(true).open(dir.join("history.jsonl"))?, "{line}")
}
