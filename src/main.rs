use std::fs::OpenOptions;
use std::io::{self, IsTerminal, Write};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};

use selective::app::{Action, App};
use selective::input::read_lines;

#[derive(Parser, Debug)]
#[command(name = "selective", version, about = "Interactive single-select filter")]
struct Cli {
    /// Header line shown above the list
    #[arg(short, long, default_value = "Select:")]
    prompt: String,

    /// Max rows for the list viewport
    #[arg(long, default_value_t = 10)]
    height: u16,

    /// Use the minimal rendering (1-line header + plain list)
    #[arg(long)]
    simple: bool,
}

enum Outcome {
    Selected(String),
    Cancelled,
}

fn main() -> ExitCode {
    match run() {
        Ok(Outcome::Selected(s)) => {
            let _ = writeln!(io::stdout().lock(), "{s}");
            ExitCode::SUCCESS
        }
        Ok(Outcome::Cancelled) => ExitCode::from(130),
        Err(e) => {
            eprintln!("selective: {e:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<Outcome> {
    let cli = Cli::parse();
    let mut items = read_lines(io::stdin().lock()).context("reading stdin")?;
    if items.is_empty() {
        anyhow::bail!("no candidates on stdin");
    }
    if items.len() == 1 {
        return Ok(Outcome::Selected(items.swap_remove(0)));
    }

    // Replace fd 0 with a dup of an inherited tty fd. crossterm's
    // tty_fd() then uses fd 0 directly. A freshly-opened /dev/tty cannot
    // be registered with kqueue on macOS (EINVAL); a dup'd one can.
    install_tty_on_stdin().context("no controlling terminal available")?;

    // Redirect fd 1 to /dev/tty for the lifetime of the TUI. crossterm's
    // cursor::position() (called by Viewport::Inline) writes the DSR
    // query (ESC[6n) unconditionally to io::stdout(); if stdout is the
    // downstream pipe the query never reaches the terminal and the
    // program hangs waiting for a response.
    //
    // Drop order is load-bearing: `_raw_guard` must drop before
    // `_stdout_guard` so that disable_raw_mode's terminal writes still
    // reach /dev/tty (not the downstream pipe). Rust drops locals in
    // reverse declaration order — keep them in this sequence.
    let _stdout_guard =
        redirect_stdout_to_tty().context("redirecting stdout to /dev/tty")?;

    enable_raw_mode().context("enabling raw mode")?;
    let _raw_guard = RawGuard;

    let list_rows = cli.height.min(items.len() as u16);
    let extra = if cli.simple { 1 } else { 7 };
    let viewport_h = list_rows.saturating_add(extra).max(extra + 1);
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions { viewport: Viewport::Inline(viewport_h) },
    )?;
    execute!(terminal.backend_mut(), Hide)?;

    let mut app = App::new(items, cli.prompt, cli.height);
    let outcome = event_loop(&mut terminal, &mut app, cli.simple);

    // `terminal.clear()` in Inline mode rewinds the cursor to viewport top
    // and clears from there to end of screen, so the shell prompt resumes
    // exactly where our command line was — no menu artifacts remain.
    let _ = terminal.clear();
    let _ = execute!(terminal.backend_mut(), Show);

    outcome
}

fn install_tty_on_stdin() -> io::Result<()> {
    if io::stdin().is_terminal() {
        return Ok(());
    }
    for src in [io::stderr().as_fd(), io::stdout().as_fd()] {
        if !src.is_terminal() {
            continue;
        }
        let dup = src.try_clone_to_owned()?;
        if unsafe { libc::dup2(dup.as_raw_fd(), 0) } < 0 {
            return Err(io::Error::last_os_error());
        }
        return Ok(());
    }
    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    if unsafe { libc::dup2(tty.as_raw_fd(), 0) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn redirect_stdout_to_tty() -> io::Result<StdoutGuard> {
    let saved = io::stdout().as_fd().try_clone_to_owned()?;
    let tty = OpenOptions::new().write(true).open("/dev/tty")?;
    if unsafe { libc::dup2(tty.as_raw_fd(), 1) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(StdoutGuard { saved })
}

struct StdoutGuard {
    saved: OwnedFd,
}

impl Drop for StdoutGuard {
    fn drop(&mut self) {
        unsafe { libc::dup2(self.saved.as_raw_fd(), 1) };
    }
}

struct RawGuard;

impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    simple: bool,
) -> Result<Outcome> {
    let mut dirty = true;
    loop {
        if dirty {
            terminal.draw(|f| selective::ui::draw(f, app, simple))?;
            dirty = false;
        }
        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(k) if k.kind == event::KeyEventKind::Press => match app.handle_key(k) {
                    Action::Continue => dirty = true,
                    Action::Confirm(s) => return Ok(Outcome::Selected(s)),
                    Action::Cancel => return Ok(Outcome::Cancelled),
                },
                Event::Resize(_, _) => dirty = true,
                _ => {}
            }
        }
    }
}

