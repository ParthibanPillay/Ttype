use crate::test::Test;
use ratatui::{prelude::*, widgets::*};
use std::time::Instant;

pub fn draw(f: &mut Frame, t: &Test) {
    let now = Instant::now();
    let area = f.area().centered(Constraint::Max(80), Constraint::Max(20));
    let s = t.stats(now);
    if t.done.is_some() {
        let [top, chart] = Layout::vertical([Constraint::Length(8), Constraint::Min(3)]).areas(area);
        let text = format!(
            "{}\n\nwpm {:.0}\nraw {:.0}\nacc {:.1}%\nconsistency {:.0}%\n\nTab: restart   Esc: quit",
            t.mode.label(), s.wpm, s.raw, s.acc, s.consistency
        );
        f.render_widget(Paragraph::new(text), top);
        let data: Vec<u64> = t.samples.iter().map(|&x| x as u64).collect();
        f.render_widget(Sparkline::default().data(&data).block(Block::bordered().title("wpm / s")), chart);
        return;
    }
    let [head, body] = Layout::vertical([Constraint::Length(2), Constraint::Min(3)]).areas(area);
    let left = match t.mode {
        crate::test::Mode::Time(sec) => format!("{:.0}", (sec as f64 - t.elapsed(now)).ceil()),
        crate::test::Mode::Words(n) => format!("{}/{n}", t.typed.iter().filter(|&&c| c == ' ').count()),
    };
    f.render_widget(
        Paragraph::new(format!("{}  {left}  {:.0} wpm   (Ctrl+T mode, Tab restart, Esc quit)", t.mode.label(), s.wpm))
            .style(Style::new().fg(Color::Yellow)),
        head,
    );
    let spans: Vec<Span> = t
        .target
        .iter()
        .enumerate()
        .map(|(i, &c)| match t.typed.get(i) {
            Some(&k) if k == c => Span::styled(c.to_string(), Color::Green),
            Some(_) => Span::styled(c.to_string(), Style::new().fg(Color::Red).underlined()),
            None if i == t.typed.len() => Span::styled(c.to_string(), Style::new().reversed()),
            None => Span::styled(c.to_string(), Color::DarkGray),
        })
        .collect();
    f.render_widget(Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false }), body);
}
