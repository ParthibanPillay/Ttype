mod history;
mod test;
mod ui;

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::{execute, terminal::supports_keyboard_enhancement};
use std::time::{Duration, Instant};
use test::{MODES, Test};

fn main() -> std::io::Result<()> {
    let mut term = ratatui::init(); // also installs a panic hook that restores the terminal
    // Kitty keyboard protocol: lets us tell Ctrl+Enter from Enter. Terminals without it still get Ctrl+J.
    let enhanced = supports_keyboard_enhancement().unwrap_or(false);
    if enhanced {
        execute!(std::io::stdout(), PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES))?;
    }
    let res = run(&mut term);
    if enhanced {
        let _ = execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
    }
    ratatui::restore();
    res
}

fn run(term: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut mode_idx = 0;
    let mut t = Test::new(MODES[mode_idx]);
    let mut saved = false;
    let mut hist: Option<Vec<history::Run>> = None; // Some = history screen open
    let mut prev_best = None; // best wpm for this mode before the finished run
    loop {
        term.draw(|f| ui::draw(f, &t, prev_best, hist.as_deref()))?;
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                let now = Instant::now();
                let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
                if hist.is_some() && !(ctrl && k.code == KeyCode::Char('c')) {
                    if matches!(k.code, KeyCode::Esc) || (ctrl && k.code == KeyCode::Char('l')) {
                        hist = None;
                    }
                    continue;
                }
                match k.code {
                    KeyCode::Char('l') if ctrl => hist = Some(history::load()),
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                    KeyCode::Char('t') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        mode_idx = (mode_idx + 1) % MODES.len();
                        t = Test::new(MODES[mode_idx]);
                        saved = false;
                    }
                    // Ctrl+Enter (plain terminals send it as Ctrl+J), or Enter on the results screen
                    KeyCode::Tab | KeyCode::Enter | KeyCode::Char('j') if k.code == KeyCode::Tab || ctrl || (k.code == KeyCode::Enter && t.done.is_some()) => {
                        t = Test::new(MODES[mode_idx]);
                        saved = false;
                    }
                    KeyCode::Backspace if ctrl || k.modifiers.contains(KeyModifiers::ALT) => t.backspace_word(),
                    KeyCode::Char('w' | 'h') if ctrl => t.backspace_word(),
                    KeyCode::Backspace => t.backspace(),
                    KeyCode::Char(c) => t.press(c, now),
                    _ => {}
                }
            }
        }
        t.tick(Instant::now());
        if t.done.is_some() && !saved {
            saved = true;
            let s = t.stats(Instant::now());
            prev_best = history::best(&t.mode.label());
            let _ = history::append(&t.mode.label(), s.wpm, s.raw, s.acc);
        }
    }
}
