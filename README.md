# Rustle

A terminal Wordle clone that pulls the daily NYT solution and lets you play in a full-screen TUI without opening a browser.

## What it is

Rustle fetches today's word from the NYT Wordle API, then runs a [ratatui](https://ratatui.rs/)-based terminal UI. Type a five-letter guess, press Enter, and see each letter coloured to guide your next attempt. You have six tries. Alongside the board you get:

- an **on-screen keyboard** that colours each letter as you use it;
- a **history panel** of the past solutions you've already played; and
- a **streak counter** tracking how many days in a row you've guessed the word (plus your best-ever streak).

Finish the day's puzzle and Rustle remembers it: relaunching replays your board read-only instead of letting you play again.

## Requirements

- Rust stable toolchain (`rustup` recommended)
- An active internet connection (to fetch the daily word)

## Install & Run

```bash
cargo install strustle --locked
strustle
```

Or, from a clone:

```bash
cargo run
```

> Tip: `--locked` makes `cargo install` use the published `Cargo.lock`, building against the exact dependency versions this release was tested with.

## How to play

1. Type a five-letter word. Letters fill the current row; **Backspace** deletes, **Esc** quits.
2. Press **Enter** to submit. Each letter is coloured:
   - **Green** — correct letter, correct position.
   - **Yellow** — letter is in the word but in the wrong position.
   - **Gray** — letter does not appear in the word (or all occurrences are already accounted for by green/yellow slots).
3. The on-screen keyboard updates to show which letters you've used and their best-known state.
4. Duplicate-letter handling follows the standard Wordle rules: extra copies of a letter are only highlighted if the solution contains that many occurrences.
5. You win if you guess the word within six attempts. Win on consecutive days to grow your streak — a missed day or a loss resets it.

## Platform notes

Two optional celebration effects activate at the end of a game:

- **Confetti** (macOS + [Raycast](https://raycast.com/)): on a win, Rustle opens the `raycast://confetti` URL to trigger the confetti animation. If Raycast is not installed, or the `open` command fails, nothing happens.
- **Loss sound**: on a loss, Rustle plays a short bundled audio clip through the system's default audio device via `rodio`. If no audio device is available, the error is silently ignored.

Both effects are entirely non-fatal — the game works normally without them.

## Module layout

| Module | Role |
|---|---|
| `celebrate` | Confetti and loss-sound effects (non-fatal). |
| `dictionary` | Bundled word-list lookup. |
| `error` | Unified `RustleError` type. |
| `feedback` | Two-pass letter-state evaluation. |
| `game` | Core game state machine. |
| `nyt` | NYT Wordle API client. |
| `persistence` | Streak/history save & load, board replay. |
| `tui` | Terminal lifecycle and the event loops. |
| `ui` | Ratatui rendering of the board, history, stats, and keyboard. |

## Roadmap

- Guess-distribution histogram in the stats panel.
- Shareable emoji-grid summary of the day's result.
