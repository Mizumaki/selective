# selective

A small CLI that drops an interactive single-select UI into a shell pipeline:
it reads candidate lines from stdin, lets the user pick one, and writes the
selected line as-is to stdout. See `README.md` for user-facing usage.

## Keeping docs in sync

These two docs have distinct ownership — keep them aligned with the code as
you work:

- **User-facing spec changes -> update `README.md`.** CLI flags, exit codes,
  input handling rules, examples, and anything else a user can observe must
  be reflected in `README.md` in the same change.
- **Architecture changes -> update this `CLAUDE.md`.** Module layout,
  layering between logic and I/O, the stdin/TUI separation strategy, the
  testing policy, and similar internal design facts must be reflected here
  in the same change.

If a change touches both (e.g. a new flag that also reshapes a module), update
both files in the same change.

## Architecture

```
src/
├── main.rs    # CLI arg parsing, stdin reading, event loop, exit-code control
├── lib.rs     # module re-exports
├── app.rs     # selection state machine (the main unit-test target)
├── input.rs   # stdin -> Vec<String> parser
└── ui.rs      # ratatui rendering
```

The crucial design point is the **separation of stdin and TUI I/O**:

- stdin is used only to read the candidate list from the pipe, and is closed
  once reading is done.
- Key input is read by crossterm, which opens `/dev/tty` itself (its standard
  behavior when stdin is not a tty).
- TUI rendering output goes to a write handle opened via
  `OpenOptions::new().write(true).open("/dev/tty")`, which is passed to the
  ratatui backend. This prevents the piped stdout and TUI rendering from
  colliding.
- Only the selection result is written to the process's stdout (i.e. the
  downstream pipe).

ratatui uses `Viewport::Inline`, so the terminal is **not** switched to the
alternate screen; only the inline region is rendered as a TUI.

**Caveat:** `Viewport::Inline` issues a cursor-position query (`ESC[6n`), so
`enable_raw_mode()` must be called **before** constructing the `Terminal`.

## Development workflow

This repository is built using **TDD**.

### Cycle

Following the `tdd` skill, repeat one cycle at a time:

1. **RED**: write one test that expresses a single behavior, and make it fail.
2. **GREEN**: add the minimum implementation needed to pass that test.
3. **REFACTOR**: while everything is green, clean up if needed.

The "write all the tests up front, then implement" horizontal-slice approach
is forbidden. Stack **one test -> one implementation** vertically
(tracer-bullet style).

### Testing policy

- **Verify behavior through the public API.** Do not poke at private functions
  or internal state from tests.
- Terminal I/O is inherently untestable, so the codebase splits the logic
  layer (`app.rs`, `input.rs`) from the I/O layer (`ui.rs`, `main.rs`).
  The logic layer is covered exhaustively by unit tests; the I/O layer is
  covered by manual / acceptance tests (see `README.md`).
- Key-event-to-action state transitions are verified by feeding `KeyEvent`
  values directly into `App`, so the tests stay in lock-step with crossterm's
  semantics.

## Commands

```sh
cargo test --lib                              # all tests
cargo test --lib enter_confirms_current_item  # a single test
cargo clippy --all-targets -- -D warnings     # lint
cargo build --release                         # release build
```
