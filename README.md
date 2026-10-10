# Chess Analyzer (native Rust)

A local, native desktop chess analyzer: import your Chess.com history (or paste PGN), review
games move-by-move against a Stockfish eval bar, get a skill-score dashboard across ratings,
openings, tactics, endgames, time management and habits, train on puzzles (including ones mined
from your own blunders), play Elo-limited bots, and ask a fully local Gemma model to explain a
mistake in plain language. No webview, no cloud dependency beyond the Chess.com API calls you
trigger and the optional local coach model.

A previous Next.js/TypeScript prototype of this app lives under [`web/`](web/) for reference; the
Rust app at the repo root is the active implementation.

## Stack

- [`iced`](https://github.com/iced-rs/iced) — native GUI, Elm-style architecture
- [`shakmaty`](https://github.com/niklasf/shakmaty) — chess rules/legality/FEN/SAN
- [`pgn-reader`](https://github.com/niklasf/rust-pgn-reader) — PGN parsing
- [`rusqlite`](https://github.com/rusqlite/rusqlite) (bundled) — games, eval cache, puzzles
- Stockfish, spoken to over UCI as a subprocess — engine analysis and Elo-limited bots
- A local Gemma GGUF model run via a bundled llama.cpp runtime — the mistake-explainer coach
- Piece art: Lichess's cburnett set (CC BY-SA, see `assets/pieces/cburnett/LICENSE`). Nav icons:
  [Lucide](https://lucide.dev/) (ISC, see `assets/icons/lucide/LICENSE`)

## Setup

```bash
./scripts/fetch-assets.sh   # downloads Stockfish, the llama.cpp runtime, and the Gemma model
cargo run
```

These are downloaded rather than committed to git since together they're ~1.75GB. You can fetch
just one component (e.g. if you only want engine analysis, not the coach) with
`./scripts/fetch-assets.sh stockfish`.

> **Note:** if your system `rustc` is broken (seen on some rolling-release distros due to a
> libstdc++ ABI mismatch), use a rustup-managed toolchain instead, e.g.
> `PATH="$HOME/.cargo/bin:$PATH" cargo run`.

## Test

```bash
cargo test
```

## Build a release binary

```bash
cargo build --release
```

The binary looks for Stockfish/the coach runtime/model at paths relative to its **working
directory** (`assets/stockfish/linux/...`, `assets/coach/linux/...`, `assets/models/gemma/...`),
so run it from the project root (or, for a packaged release, from inside the extracted release
folder — see the `Release` GitHub Actions workflow, which bundles Stockfish alongside the binary).

## CI / Releases

- `.github/workflows/ci.yml` runs `cargo check`/`cargo test` on every push and PR.
- `.github/workflows/release.yml` builds a release binary and publishes a Linux x86-64 tarball
  (binary + bundled Stockfish + `fetch-assets.sh`) to GitHub Releases whenever a tag matching
  `v*.*.*` is pushed. The llama.cpp runtime and Gemma model are left out of the release archive
  (large, optional) — run `fetch-assets.sh` after extracting if you want the coach feature.
