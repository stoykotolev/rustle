# Rustle

An amazing, bleeding edge cli you can use to play Wordle in your terminal, without having to open the browser

## IT IS STILL WIP THOUGH

There are a couple of features missing, but I am getting to them.

## How to build

The usual stuff

```bash
cargo run
```

will build the project and start it

## To Do

- [x] Add a function to check if the word has repeatable letters and detect their position/color
- [ ] Add a check to see if you have already solved the word for this day
- [ ] Maybe add a tracker of how much correct guesses, etc. like with the original game
- [x] Fix a bug where if you pass duplicate letters in a word that doesn't have them, it still colors them.
