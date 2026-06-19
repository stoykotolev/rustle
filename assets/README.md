# Word-list assets

Rustle validates guesses using the same two-list model as the original NYT
Wordle:

- **`answers.txt`** — the canonical *answers* list (historically `La`). These
  are the ~2,300 words that can ever be the solution.
- **`allowed.txt`** — the set of words accepted as *guesses*. This is the union
  of the canonical *extra allowed guesses* list (historically `Ta`), the answers
  list, and the project's previous combined `words.txt`. A guess is valid if it
  appears in this set; the loader additionally unions the answers into the
  allowed set at runtime, so any answer is always a legal guess.

A guess is accepted iff it is in `answers ∪ allowed` (case-insensitive, exactly
five ASCII letters). These bundled lists are a frozen snapshot and can lag the
live NYT answer rotation, so the solution itself is not guaranteed to appear in
them. The game loop (`Game::apply_turn`) therefore keeps a small guard that
always accepts a guess equal to the current solution, which keeps any day's
game winnable regardless of list staleness.

## Provenance

Sourced from cfreshman's faithful mirrors of the original Wordle lists:

| File | Source | Original list |
|---|---|---|
| `answers.txt` | <https://gist.githubusercontent.com/cfreshman/a03ef2cba789d8cf00c08f767e0fad7b/raw> | `La` (answers) |
| `allowed.txt` (extras) | <https://gist.githubusercontent.com/cfreshman/cdcdf777450c5b5301e439061d29694c/raw> | `Ta` (allowed guesses / "herrings") |

Fetched: 2026-06-19.

`allowed.txt` was additionally unioned with the project's previous
`assets/words.txt` (12,970 words) so that the new two-list model never rejects a
guess the old single-list model would have accepted (no guess-acceptance
regression). `words.txt` has been removed; its contents live on inside
`allowed.txt`.

### Counts

- `answers.txt`: 2,315 words
- `allowed.txt`: 12,972 words

## Regenerating

```sh
# Fetch canonical lists
curl -sL "https://gist.githubusercontent.com/cfreshman/a03ef2cba789d8cf00c08f767e0fad7b/raw" -o answers_raw.txt
curl -sL "https://gist.githubusercontent.com/cfreshman/cdcdf777450c5b5301e439061d29694c/raw" -o allowed_raw.txt

# Normalize: lowercase, keep only 5-letter ascii words, sort, dedupe.
norm() { tr 'A-Z' 'a-z' | tr ',' '\n' | sed 's/[[:space:]]//g' | grep -E '^[a-z]{5}$'; }

# answers.txt = canonical answers only
norm < answers_raw.txt | sort -u > answers.txt

# allowed.txt = extras ∪ answers (∪ any legacy words.txt to preserve coverage)
{ norm < allowed_raw.txt; echo; norm < answers_raw.txt; } | sort -u > allowed.txt
```

Each line in both files is exactly five lowercase ASCII letters; both files are
sorted and free of duplicates. The asset-integrity unit tests in
`src/dictionary.rs` enforce these invariants at build time.
