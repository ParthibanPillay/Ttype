use crate::test::Test;
use ratatui::{prelude::*, widgets::*};
use std::time::Instant;

pub fn draw(f: &mut Frame, t: &Test, prev_best: Option<f64>, hist: Option<&[crate::history::Run]>) {
    if let Some(h) = hist {
        return draw_history(f, h);
    }
    let now = Instant::now();
    let area = f.area().centered(Constraint::Max(80), Constraint::Max(20));
    let s = t.stats(now);
    if t.done.is_some() {
        let [top, chart] =
            Layout::vertical([Constraint::Length(8), Constraint::Min(3)]).areas(area);
        let pb = match prev_best {
            Some(b) if s.wpm <= b => format!("best {b:.0}"),
            _ => "new PB!".into(),
        };
        let text = format!(
            "{}  {pb}\n\nwpm {:.0}\nraw {:.0}\nacc {:.1}%\nconsistency {:.0}%\n\nTab/Enter: restart   Esc: quit",
            t.mode.label(),
            s.wpm,
            s.raw,
            s.acc,
            s.consistency
        );
        f.render_widget(Paragraph::new(text), top);
        let data: Vec<u64> = t.samples.iter().map(|&x| x as u64).collect();
        f.render_widget(
            Sparkline::default()
                .data(&data)
                .block(Block::bordered().title("wpm / s")),
            chart,
        );
        return;
    }
    let [head, body] = Layout::vertical([Constraint::Length(2), Constraint::Min(3)]).areas(area);
    let left = match t.mode {
        crate::test::Mode::Time(sec) => format!("{:.0}", (sec as f64 - t.elapsed(now)).ceil()),
        crate::test::Mode::Words(n) => {
            format!("{}/{n}", t.typed.len() - 1)
        }
    };
    let [title, bar] = Layout::vertical([Constraint::Length(1); 2]).areas(head);
    f.render_widget(
        Paragraph::new(format!("{}  {left}  {:.0} wpm   (Ctrl+T mode, Ctrl+L history, Tab/Ctrl+Enter restart, Esc quit)", t.mode.label(), s.wpm))
            .style(Style::new().fg(Color::Yellow)),
        title,
    );
    let progress = match t.mode {
        crate::test::Mode::Time(sec) => t.elapsed(now) / sec as f64,
        crate::test::Mode::Words(_) => (t.typed.len() - 1) as f64 / t.words.len() as f64,
    };
    f.render_widget(
        LineGauge::default().label("").ratio(progress.clamp(0.0, 1.0)).filled_style(Color::Yellow).unfilled_style(Color::DarkGray),
        bar,
    );
    // Monkeytype-style: show 3 lines, keep the cursor on the 2nd, scroll as lines are finished.
    // `starts` holds the index of the first word on each line; a word is as wide as its extras.
    let cur = t.typed.len() - 1;
    let width = |i: usize| t.words[i].len().max(t.typed.get(i).map_or(0, Vec::len));
    let (n, w) = (t.words.len(), body.width as usize);
    let mut starts = vec![0];
    let mut col = 0;
    for i in 0..n {
        if col > 0 && col + width(i) > w {
            starts.push(i);
            col = 0;
        }
        col += width(i) + 1;
    }
    starts.push(n);
    let cur_line = starts.partition_point(|&s| s <= cur) - 1;
    let top = cur_line.saturating_sub(1);
    let lines: Vec<Line> = starts[top..]
        .windows(2)
        .take(3)
        .flat_map(|r| {
            let spans: Vec<Span> = (r[0]..r[1]).flat_map(|i| word_spans(t, i)).collect();
            [Line::from(spans), Line::default()] // blank row between lines
        })
        .collect();
    f.render_widget(Paragraph::new(lines), body);
}

/// One word plus the space after it. Extras show in red after the word; the cursor is a reversed cell.
fn word_spans(t: &Test, i: usize) -> Vec<Span<'static>> {
    let cur = t.typed.len() - 1;
    let (word, typed) = (&t.words[i], t.typed.get(i).map_or(&[][..], Vec::as_slice));
    let mut out: Vec<Span> = (0..word.len().max(typed.len()))
        .map(|j| match (word.get(j), typed.get(j)) {
            (Some(&c), Some(&k)) if k == c => Span::styled(c.to_string(), Color::Green),
            (Some(&c), Some(_)) => Span::styled(c.to_string(), Style::new().fg(Color::Red).underlined()),
            (None, Some(&k)) => Span::styled(k.to_string(), Color::Red),
            (Some(&c), None) if i == cur && j == typed.len() => Span::styled(c.to_string(), Style::new().reversed()),
            (Some(&c), None) if i < cur => Span::styled(c.to_string(), Style::new().fg(Color::DarkGray).underlined()), // skipped
            (Some(&c), None) => Span::styled(c.to_string(), Color::DarkGray),
            (None, None) => unreachable!(),
        })
        .collect();
    out.push(if i == cur && typed.len() >= word.len() { Span::styled(" ", Style::new().reversed()) } else { Span::raw(" ") });
    out
}

fn draw_history(f: &mut Frame, runs: &[crate::history::Run]) {
    let area = f.area().centered(Constraint::Max(80), Constraint::Max(24));
    let [list, chart] = Layout::vertical([Constraint::Min(5), Constraint::Length(6)]).areas(area);
    let rows: Vec<String> = runs.iter().rev().map(|r| format!("{:<14} {:>4.0} wpm  {:>5.1}%", r.mode, r.wpm, r.acc)).collect();
    let text = if rows.is_empty() { "no runs yet".into() } else { rows.join("\n") }; // Paragraph clips what doesn't fit
    f.render_widget(Paragraph::new(text).block(Block::bordered().title("history (newest first) - Esc/Ctrl+L close")), list);
    let data: Vec<u64> = runs.iter().rev().take(60).rev().map(|r| r.wpm as u64).collect();
    f.render_widget(Sparkline::default().data(&data).block(Block::bordered().title("wpm over last runs")), chart);
}
