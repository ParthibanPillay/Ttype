use rand::seq::IndexedRandom;
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum Mode {
    Time(u64),
    Words(usize),
}

pub const MODES: [Mode; 6] = [
    Mode::Time(15),
    Mode::Time(30),
    Mode::Time(60),
    Mode::Words(10),
    Mode::Words(25),
    Mode::Words(50),
];

impl Mode {
    pub fn label(self) -> String {
        match self {
            Mode::Time(s) => format!("time {s}s"),
            Mode::Words(n) => format!("words {n}"),
        }
    }
}

pub struct Test {
    pub target: Vec<char>,
    pub typed: Vec<char>,
    pub mode: Mode,
    started: Option<Instant>,
    pub keys: u32,
    pub wrong_keys: u32,
    pub samples: Vec<f64>,
    last_correct: usize,
    pub done: Option<f64>, // elapsed secs when finished
}

pub struct Stats {
    pub wpm: f64,
    pub raw: f64,
    pub acc: f64,
    pub consistency: f64,
}

impl Test {
    pub fn new(mode: Mode) -> Self {
        let words: Vec<&str> = include_str!("../words.txt").split_whitespace().collect();
        let n = match mode {
            Mode::Time(_) => 300, // ponytail: fixed pool, 300 words outlasts any realistic 60s run
            Mode::Words(n) => n,
        };
        let mut rng = rand::rng();
        let target = (0..n).map(|_| *words.choose(&mut rng).unwrap()).collect::<Vec<_>>().join(" ");
        Test {
            target: target.chars().collect(),
            typed: vec![],
            mode,
            started: None,
            keys: 0,
            wrong_keys: 0,
            samples: vec![],
            last_correct: 0,
            done: None,
        }
    }

    pub fn correct(&self) -> usize {
        self.typed.iter().zip(&self.target).filter(|(a, b)| a == b).count()
    }

    /// Seconds since first keypress (0 before start).
    pub fn elapsed(&self, now: Instant) -> f64 {
        self.done.unwrap_or_else(|| self.started.map_or(0.0, |s| now.duration_since(s).as_secs_f64()))
    }

    pub fn press(&mut self, c: char, now: Instant) {
        if self.done.is_some() {
            return;
        }
        self.started.get_or_insert(now);
        self.keys += 1;
        if self.typed.len() < self.target.len() {
            if self.target[self.typed.len()] != c {
                self.wrong_keys += 1;
            }
            self.typed.push(c);
        }
        // ponytail: no extra chars past a word end; a wrong key just consumes the next slot
        if matches!(self.mode, Mode::Words(_)) && self.typed.len() == self.target.len() {
            self.done = Some(self.elapsed(now));
        }
    }

    pub fn backspace(&mut self) {
        if self.done.is_none() {
            self.typed.pop();
        }
    }

    /// Call every frame: records per-second samples and ends time mode.
    pub fn tick(&mut self, now: Instant) {
        if self.done.is_some() || self.started.is_none() {
            return;
        }
        let mut el = self.elapsed(now);
        if let Mode::Time(s) = self.mode {
            el = el.min(s as f64);
        }
        while (self.samples.len() as f64) < el.floor() {
            let c = self.correct();
            self.samples.push(c.saturating_sub(self.last_correct) as f64 * 12.0); // chars/s -> wpm
            self.last_correct = c;
        }
        if let Mode::Time(s) = self.mode {
            if el >= s as f64 {
                self.done = Some(s as f64);
            }
        }
    }

    pub fn stats(&self, now: Instant) -> Stats {
        let mins = (self.elapsed(now) / 60.0).max(1e-9);
        let wpm = self.correct() as f64 / 5.0 / mins;
        let raw = self.typed.len() as f64 / 5.0 / mins;
        let acc = if self.keys == 0 { 100.0 } else { 100.0 * (self.keys - self.wrong_keys) as f64 / self.keys as f64 };
        let n = self.samples.len() as f64;
        let mean = self.samples.iter().sum::<f64>() / n.max(1.0);
        let var = self.samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n.max(1.0);
        let consistency = if mean > 0.0 { (100.0 - 100.0 * var.sqrt() / mean).max(0.0) } else { 0.0 };
        Stats { wpm, raw, acc, consistency }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn words_mode_metrics() {
        let mut t = Test::new(Mode::Words(10));
        let t0 = Instant::now();
        t.target = "hello world".chars().collect(); // 11 chars
        for (i, c) in "hello world".chars().enumerate() {
            t.press(c, t0 + Duration::from_millis(i as u64 * 100));
        }
        let end = t0 + Duration::from_secs(60);
        assert!(t.done.is_some());
        let s = t.stats(end);
        // 11 chars in 1.0s -> (11/5)/(1/60) = 132 wpm
        assert!((s.wpm - 132.0).abs() < 1.0, "{}", s.wpm);
        assert_eq!(s.acc, 100.0);
    }

    #[test]
    fn mistake_hurts_accuracy_even_if_fixed() {
        let mut t = Test::new(Mode::Time(15));
        let t0 = Instant::now();
        t.target = "ab".chars().collect();
        t.press('x', t0);
        t.backspace();
        t.press('a', t0);
        assert_eq!(t.stats(t0).acc, 50.0);
        assert_eq!(t.correct(), 1);
    }

    #[test]
    fn time_mode_ends() {
        let mut t = Test::new(Mode::Time(15));
        let t0 = Instant::now();
        t.press('a', t0);
        t.tick(t0 + Duration::from_secs(16));
        assert_eq!(t.done, Some(15.0));
        assert_eq!(t.samples.len(), 15);
    }
}
