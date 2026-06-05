# selective

A small CLI utility that drops an **interactive selection UI** into a shell pipeline.
It reads candidate lines from stdin, lets the user pick one (or several, with `-m`), and writes the selected line(s) **as-is to stdout**.

## Examples

Delete a branch picked from `git branch`:

```sh
git branch | sed 's/^[* ] //' | selective --prompt "delete branch:" | xargs -r git branch -D
```

Delete several branches in one pass with multi-select:

```sh
git branch | sed 's/^[* ] //' | selective -m --prompt "delete:" | xargs -r git branch -D
```

`cd` into a worktree picked from `git worktree list`:

```sh
cd "$(git worktree list | awk '{print $1}' | selective --prompt "cd worktree:")"
```

## Features

- **Pure filter**: candidates from stdin, result(s) on stdout. Fits naturally into shell pipelines.
- **Single- or multi-select**: single by default; pass `-m` / `--multi` to toggle items with Space and confirm a set with Enter.
- **Cancel-safe**: Esc / `Ctrl-C` / `q` emits nothing on stdout and exits with **exit code 130**. Combined with `xargs -r`, no downstream command fires.
- **TTY isolation**: even when stdin is a pipe, key input is read directly from `/dev/tty`, so `cmd | selective | cmd2` just works.
- Inline TUI built with **ratatui + crossterm** (does not switch the terminal to fullscreen).
- **macOS / Linux supported** (Windows is out of scope).

## Usage

```
Usage: selective [OPTIONS]

Options:
  -p, --prompt <PROMPT>         Header text [default: "Select:"]
      --height <HEIGHT>         Maximum number of list rows [default: 10]
      --simple                  Use the minimal rendering (1-line header + plain list)
  -m, --multi                   Enable multi-select (space toggles; Ctrl-A all; Ctrl-D/Ctrl-U clear)
      --border-color <COLOR>    Border + embedded prompt color [default: reset]
      --cursor-color <COLOR>    Cursor arrow + selected row color [default: cyan]
  -h, --help                    Help
  -V, --version                 Version
```

By default `selective` renders a rich inline TUI: a rounded border in the
terminal's default text color with the prompt embedded on the top edge, an
`n/total` counter on the right, and a dimmed key-hint footer below. Pass
`--simple` to fall back to the original 1-line header + reverse-video cursor
list, which is lighter and uses fewer rows.

`--border-color` and `--cursor-color` accept either a color name
(`reset` / `default`, `black`, `red`, `green`, `yellow`, `blue`, `magenta`,
`cyan`, `gray`, `dark-gray`, `light-red`, `light-green`, `light-yellow`,
`light-blue`, `light-magenta`, `light-cyan`, `white`) or a hex literal in the
form `#RRGGBB` (e.g. `--cursor-color '#ff8800'`). `reset` (the default for
`--border-color`) tracks the terminal's current foreground color.

### Exit codes

| code | meaning                                              |
| ---- | ---------------------------------------------------- |
| 0    | Selection made; the chosen line has been written to stdout |
| 1    | Input error (empty stdin / cannot open `/dev/tty`, etc.) |
| 130  | User cancelled                                       |

### Input handling

- Candidates are **one per line**.
- Empty lines are ignored.
- CRLF (`\r\n`) is treated as LF.
- If there are **0 candidates**, the TUI is not launched and the program exits with 1.
- If there is **exactly 1 candidate**, the TUI is not launched; that line is written to stdout and the program exits with 0 (auto-confirm).

### Multi-select (`-m` / `--multi`)

In multi-select mode the UI gains a checkbox column and these extra keys:

| key                | action                                                       |
| ------------------ | ------------------------------------------------------------ |
| `Space`            | Toggle the item under the cursor                             |
| `Ctrl-A`           | Select all                                                   |
| `Ctrl-D` / `Ctrl-U`| Clear all selections                                         |
| `Enter`            | Confirm — emit every selected line, **in input order**       |

Pressing `Enter` with nothing toggled is a no-op. Movement and cancel keys are unchanged.

## Development

Assumes a standard Rust toolchain (stable `rustc` / `cargo`, e.g. installed via [rustup](https://rustup.rs/)) is already set up.

### Commands

```sh
# All tests
cargo test --lib

# A single test
cargo test --lib enter_confirms_current_item

# Lint
cargo clippy --all-targets -- -D warnings

# Dev build + run
cargo run -- --prompt "pick:"            # ※ no stdin, so it reports an input error
printf 'a\nb\nc\n' | cargo run -- --prompt "pick:"

# Release build
cargo build --release
```

### Acceptance tests (manual)

Logic is covered by unit tests, but the look of the TUI and its key handling
should be verified on an actual terminal.

```sh
# 1. Normal selection
printf 'alpha\nbeta\ngamma\n' | ./target/release/selective
# → arrow keys + Enter writes the selected line to stdout

# 2. Cancel
printf 'alpha\nbeta\n' | ./target/release/selective; echo "exit=$?"
# → Esc / Ctrl-C / q yields exit=130 with empty stdout

# 3. Single line auto-confirms
printf 'only\n' | ./target/release/selective
# → immediately outputs "only" and exits 0

# 4. Empty input is an error
: | ./target/release/selective; echo "exit=$?"
# → "selective: no candidates on stdin" / exit=1

# 5. Resize the terminal width and confirm the rendering does not break
```
