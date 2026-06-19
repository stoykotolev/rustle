# Rustle

A terminal Wordle clone that pulls the daily NYT solution and lets you play without opening a browser.

## What it is

Rustle fetches today's word from the NYT Wordle API, then runs an interactive loop in your terminal. Type a five-letter guess, press Enter, and see each letter coloured to guide your next attempt. You have six tries.

## Requirements

- Rust stable toolchain (`rustup` recommended)
- An active internet connection (to fetch the daily word)

## Build & Run

```bash
cargo run
```

That's it. Cargo compiles everything and starts the game.

## How to play

1. Type any valid five-letter English word and press Enter.
2. Each letter is coloured after your guess:
   - **Green** — correct letter, correct position.
   - **Yellow** — letter is in the word but in the wrong position.
   - **Gray** — letter does not appear in the word (or all occurrences are already accounted for by green/yellow slots).
3. Duplicate-letter handling follows the standard Wordle rules: extra copies of a letter are only highlighted if the solution contains that many occurrences.
4. You win if you guess the word within six attempts.

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
| `game` | Core game loop and state machine. |
| `nyt` | NYT Wordle API client. |
| `render` | ANSI colour rendering. |

## Roadmap

- Persist today's result so the game can't be replayed after a win.
- Show a running score / guess-distribution histogram.
