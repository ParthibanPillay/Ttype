# ttest

A Monkeytype-style typing test for the terminal. Local only: no accounts, no server.

## Install

### Prebuilt (no Rust needed)

Once a release exists, on Linux/macOS:

```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ParthibanPillay/Ttype/releases/latest/download/ttest-installer.sh | sh
```

On Windows (PowerShell):

```
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ParthibanPillay/Ttype/releases/latest/download/ttest-installer.ps1 | iex"
```

Or download a binary from the [Releases page](https://github.com/ParthibanPillay/Ttype/releases).

### From source

1. Install Rust 1.85 or newer (the project uses edition 2024) via [rustup](https://rustup.rs):

   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   On Windows, download `rustup-init.exe` from the same site. Check with `rustc --version`; if it is older than 1.85, run `rustup update`.

2. Clone and install:

   ```
   git clone https://github.com/ParthibanPillay/Ttype.git
   cd Ttype
   cargo install --path .
   ```

3. Run it from anywhere:

   ```
   ttest
   ```

   If the command is not found, add `~/.cargo/bin` to your `PATH`.

To try it without installing, run `cargo run --release` inside the cloned folder.

To uninstall: `cargo uninstall ttest`.

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

## Releasing (maintainer)

```
# bump version in Cargo.toml, commit, then:
git tag v0.1.0
git push origin main --tags
```

The tag triggers `.github/workflows/release.yml`, which builds binaries and installers and publishes the GitHub Release.
