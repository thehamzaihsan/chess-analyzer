# Bundled Lichess Puzzles

`lichess_puzzles.tsv.gz` is a curated, balanced subset of ~10,000 tactical puzzles sampled from the Lichess Puzzle Database:

- Source: <https://database.lichess.org/#puzzles> (`lichess_db_puzzle.csv.zst`)
- License: **CC0 1.0 Universal (Public Domain Dedication)** — Lichess publishes its puzzle database under CC0.
- Generation script: [`scripts/sample-lichess-puzzles.py`](../../scripts/sample-lichess-puzzles.py)
- Filtering & sampling criteria:
  - Rating: `600` to `2800` (sampled across 200-point rating bands and balanced across tactical themes)
  - Popularity: `> 80`
  - Plays (`NbPlays`): `> 500`
  - Move sequence: at least 2 UCI moves (`moves[0]` is the opponent's setup move applied to `FEN`, `moves[1..]` is the solver's solution line).

To regenerate the bundled asset from the live Lichess database stream:

```bash
python3 scripts/sample-lichess-puzzles.py
```
