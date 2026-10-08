# ttest

A Monkeytype-style typing test for the terminal. Local only: no accounts, no server.

## Run

```
cargo run --release
```

Requires Rust (stable).

## Keys

| Key      | Action                                                  |
|----------|---------------------------------------------------------|
| `Tab`    | Restart with new words                                  |
| `Ctrl+T` | Cycle mode: 15s, 30s, 60s, 10 words, 25 words, 50 words |
| `Esc`    | Quit                                                    |

The timer starts on your first keypress.

## Metrics

- **wpm**: correct characters / 5 per minute
- **raw**: all typed characters / 5 per minute
- **acc**: correct keystrokes / total keystrokes (fixed mistakes still count against you)
- **consistency**: 100 minus the coefficient of variation of per-second wpm

## History

Each finished test appends a JSON line to `~/.local/share/ttest/history.jsonl`:

```json
{"ts":1791460000,"mode":"time 15s","wpm":72.0,"raw":75.0,"acc":96.0}
```

## Layout

- `src/test.rs`: typing state machine and metrics (unit tested: `cargo test`)
- `src/ui.rs`: ratatui rendering
- `src/history.rs`: history file writer
- `words.txt`: embedded word list

## Known limits

- No typing past the end of a word; a wrong key just takes the next slot.
- Time mode uses a fixed pool of 300 words.
- Long lines may wrap mid-word.
