use rand::seq::IndexedRandom;
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum Mode {
    Time(u64),
    Words(usize),
}

/// Extra chars allowed past the end of a word.
const MAX_EXTRA: usize = 10;

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
    pub words: Vec<Vec<char>>,
    /// One entry per word reached; the last is the word being typed. May run past the word's length (extras).
    pub typed: Vec<Vec<char>>,
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
        Test {
            words: (0..n).map(|_| words.choose(&mut rng).unwrap().chars().collect()).collect(),
            typed: vec![vec![]],
            mode,
            started: None,
            keys: 0,
            wrong_keys: 0,
            samples: vec![],
            last_correct: 0,
            done: None,
        }
    }

    /// Correct chars, Monkeytype-style: fully correct finished words (+1 for the space), plus the correct prefix of the current word.
    pub fn correct(&self) -> usize {
        let cur = self.typed.len() - 1;
        let done: usize = (0..cur).filter(|&i| self.typed[i] == self.words[i]).map(|i| self.words[i].len() + 1).sum();
        done + if self.words[cur].starts_with(&self.typed[cur]) { self.typed[cur].len() } else { 0 }
    }

    /// Every char typed, wrong or right, plus a space per finished word.
    fn typed_chars(&self) -> usize {
        self.typed.iter().map(Vec::len).sum::<usize>() + self.typed.len() - 1
    }

    /// Seconds since first keypress (0 before start).
    pub fn elapsed(&self, now: Instant) -> f64 {
        self.done.unwrap_or_else(|| self.started.map_or(0.0, |s| now.duration_since(s).as_secs_f64()))
    }

    pub fn press(&mut self, c: char, now: Instant) {
        let i = self.typed.len() - 1;
        if self.done.is_some() || (c == ' ' && self.typed[i].is_empty()) {
            return;
        }
        if c != ' ' && self.typed[i].len() >= self.words[i].len() + MAX_EXTRA {
            return;
        }
        self.started.get_or_insert(now);
        self.keys += 1;
        let last = i + 1 == self.words.len();
        if c == ' ' {
            // space skips whatever is left of the word; it never counts as a mistake
            if last {
                self.done = Some(self.elapsed(now));
            } else {
                self.typed.push(vec![]);
            }
            return;
        }
        if self.words[i].get(self.typed[i].len()) != Some(&c) {
            self.wrong_keys += 1;
        }
        self.typed[i].push(c);
        if last && self.typed[i] == self.words[i] {
            self.done = Some(self.elapsed(now));
        }
    }

    pub fn backspace(&mut self) {
        let i = self.typed.len() - 1;
        if self.done.is_some() {
            return;
        }
        if self.typed[i].pop().is_none() && i > 0 && self.typed[i - 1] != self.words[i - 1] {
            self.typed.pop(); // step back into a wrong word; correct words are locked
        }
    }

    /// Ctrl/Alt+Backspace: clear the current word (stepping back into a wrong word first if it's empty).
    pub fn backspace_word(&mut self) {
        if self.typed.last().is_some_and(Vec::is_empty) {
            self.backspace();
        }
        if self.done.is_none() {
            self.typed.last_mut().unwrap().clear();
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
        let raw = self.typed_chars() as f64 / 5.0 / mins;
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

    fn set(t: &mut Test, text: &str) {
        t.words = text.split(' ').map(|w| w.chars().collect()).collect();
    }

    fn type_str(t: &mut Test, text: &str, t0: Instant) {
        for c in text.chars() {
            t.press(c, t0);
        }
    }

    #[test]
    fn words_mode_metrics() {
        let mut t = Test::new(Mode::Words(10));
        let t0 = Instant::now();
        set(&mut t, "hello world"); // 11 chars
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
        set(&mut t, "ab");
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

    #[test]
    fn space_skips_rest_of_word() {
        let mut t = Test::new(Mode::Time(15));
        let t0 = Instant::now();
        set(&mut t, "hello world again");
        type_str(&mut t, "he world", t0);
        assert_eq!(t.typed, vec![vec!['h', 'e'], "world".chars().collect()]);
        assert_eq!(t.correct(), 5); // "he" is wrong as a word, so only "world" counts
    }

    #[test]
    fn extras_stay_in_their_word() {
        let mut t = Test::new(Mode::Time(15));
        let t0 = Instant::now();
        set(&mut t, "hi there");
        type_str(&mut t, "hixx ", t0);
        assert_eq!(t.typed[0].len(), 4);
        assert_eq!(t.typed.len(), 2); // next word starts clean
        assert_eq!(t.stats(t0).acc, 60.0); // h i ok, 2 extras wrong, space ok
    }

    #[test]
    fn backspace_only_enters_wrong_words() {
        let mut t = Test::new(Mode::Time(15));
        let t0 = Instant::now();
        set(&mut t, "ab cd ef");
        type_str(&mut t, "ab ", t0);
        t.backspace(); // "ab" is correct: locked
        assert_eq!(t.typed.len(), 2);
        type_str(&mut t, "x ", t0);
        t.backspace();
        t.backspace(); // back into wrong "x" word, then delete it
        assert_eq!((t.typed.len(), t.typed[1].len()), (2, 0));
    }

    #[test]
    fn backspace_word_clears_word_then_steps_back() {
        let mut t = Test::new(Mode::Time(15));
        let t0 = Instant::now();
        set(&mut t, "ab cd ef");
        type_str(&mut t, "ab cx", t0);
        t.backspace_word();
        assert_eq!((t.typed.len(), t.typed[1].len()), (2, 0));
        t.backspace_word(); // "ab" is correct: stays locked
        assert_eq!(t.typed.len(), 2);
        type_str(&mut t, "xy ", t0);
        t.backspace_word(); // empty -> back into wrong "xy" and clear it
        assert_eq!((t.typed.len(), t.typed[1].len()), (2, 0));
    }

    #[test]
    fn empty_word_space_ignored_and_last_space_finishes() {
        let mut t = Test::new(Mode::Words(2));
        let t0 = Instant::now();
        set(&mut t, "ab cd");
        t.press(' ', t0);
        assert_eq!(t.keys, 0);
        type_str(&mut t, "ab c ", t0);
        assert!(t.done.is_some());
    }
}
