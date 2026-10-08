mod history;
mod test;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};
use test::{MODES, Test};

fn main() -> std::io::Result<()> {
    let mut term = ratatui::init(); // also installs a panic hook that restores the terminal
    let res = run(&mut term);
    ratatui::restore();
    res
}

fn run(term: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut mode_idx = 0;
    let mut t = Test::new(MODES[mode_idx]);
    let mut saved = false;
    loop {
        term.draw(|f| ui::draw(f, &t))?;
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                let now = Instant::now();
                match k.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                    KeyCode::Char('t') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        mode_idx = (mode_idx + 1) % MODES.len();
                        t = Test::new(MODES[mode_idx]);
                        saved = false;
                    }
                    KeyCode::Tab => {
                        t = Test::new(MODES[mode_idx]);
                        saved = false;
                    }
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
            let _ = history::append(&t.mode.label(), s.wpm, s.raw, s.acc);
        }
    }
}
