# selective

A small CLI utility that drops an **interactive single-select UI** into a shell pipeline.
It reads candidate lines from stdin, lets the user pick one, and writes the selected line **as-is to stdout**.

## Examples

Delete a branch picked from `git branch`:

```sh
git branch | sed 's/^[* ] //' | selective --prompt "delete branch:" | xargs -r git branch -D
```

`cd` into a worktree picked from `git worktree list`:

```sh
cd "$(git worktree list | awk '{print $1}' | selective --prompt "cd worktree:")"
```

## Features

- **Pure filter**: candidates from stdin, result on stdout. Fits naturally into shell pipelines.
- **Single-select only**: confirm with Enter.
- **Cancel-safe**: Esc / `Ctrl-C` / `q` emits nothing on stdout and exits with **exit code 130**. Combined with `xargs -r`, no downstream command fires.
- **TTY isolation**: even when stdin is a pipe, key input is read directly from `/dev/tty`, so `cmd | selective | cmd2` just works.
- Inline TUI built with **ratatui + crossterm** (does not switch the terminal to fullscreen).
- **macOS / Linux supported** (Windows is out of scope).

## Usage

```
Usage: selective [OPTIONS]

Options:
  -p, --prompt <PROMPT>   Header text [default: "Select:"]
      --height <HEIGHT>   Maximum number of list rows [default: 10]
  -h, --help              Help
  -V, --version           Version
```

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
