# ttype

A Monkeytype-style typing test for the terminal. Local only: no accounts, no server.

## Website

The terminal-style project site lives at [parthibanpillay.github.io/Ttype](https://parthibanpillay.github.io/Ttype/). To publish it, open the repository's **Settings → Pages**, select **GitHub Actions** as the source, and push to `main`. The workflow deploys the static files in `docs/`.

## Install

Clone the repo, build, and install. Needs Rust 1.85 or newer.

### Ubuntu / Debian

```
sudo apt install git build-essential rustup
rustup default stable
git clone https://github.com/ParthibanPillay/Ttype.git
cd Ttype
cargo install --path .
ttype
```

### Arch Linux

```
sudo pacman -S git base-devel rust
git clone https://github.com/ParthibanPillay/Ttype.git
cd Ttype
cargo install --path .
ttype
```

### Windows

In PowerShell (also install the "Desktop development with C++" workload from Visual Studio Build Tools):

```
winget install Git.Git Rustlang.Rustup
git clone https://github.com/ParthibanPillay/Ttype.git
cd Ttype
cargo install --path .
ttype
```

If `ttype` is not found, add `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on Windows) to your `PATH`.

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

Each finished test appends a JSON line to `~/.local/share/ttype/history.jsonl`:

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
