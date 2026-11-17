#!/usr/bin/env python3
"""Stream the CC0 Lichess puzzle database and sample a balanced subset.

Streams https://database.lichess.org/lichess_db_puzzle.csv.zst via curl + zstd
(without saving the ~300 MB archive to disk) and produces a compact gzipped TSV
in assets/puzzles/lichess_puzzles.tsv.gz.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import io
import os
import random
import subprocess
import sys
from collections import Counter, defaultdict

LICHESS_PUZZLE_URL = "https://database.lichess.org/lichess_db_puzzle.csv.zst"

# 11 bands across 600..=2800 (200-point increments)
BANDS = [
    (600, 799),
    (800, 999),
    (1000, 1199),
    (1200, 1399),
    (1400, 1599),
    (1600, 1799),
    (1800, 1999),
    (2000, 2199),
    (2200, 2399),
    (2400, 2599),
    (2600, 2800),
]

# Specific tactical/endgame themes prioritized before generic ones so every
# common theme gets representation within each rating band.
THEME_PRIORITY = [
    "smotheredMate",
    "backRankMate",
    "mateIn1",
    "mateIn2",
    "mateIn3",
    "fork",
    "pin",
    "skewer",
    "discoveredAttack",
    "doubleCheck",
    "deflection",
    "attraction",
    "clearance",
    "interference",
    "xRayAttack",
    "sacrifice",
    "hangingPiece",
    "trappedPiece",
    "promotion",
    "underPromotion",
    "zugzwang",
    "quietMove",
    "exposedKing",
    "kingsideAttack",
    "queensideAttack",
    "rookEndgame",
    "pawnEndgame",
    "endgame",
]


def rating_band(rating: int) -> int | None:
    for idx, (lo, hi) in enumerate(BANDS):
        if lo <= rating <= hi:
            return idx
    return None


def primary_theme(themes_str: str) -> str:
    theme_set = set(themes_str.split())
    for t in THEME_PRIORITY:
        if t in theme_set:
            return t
    return "other"


def reservoir_insert(
    reservoir: list[tuple[str, ...]],
    seen_count: int,
    item: tuple[str, ...],
    cap: int,
    rng: random.Random,
) -> int:
    seen_count += 1
    if len(reservoir) < cap:
        reservoir.append(item)
    else:
        j = rng.randrange(seen_count)
        if j < cap:
            reservoir[j] = item
    return seen_count


def pick_balanced_from_band(
    buckets: dict[str, list[tuple[str, ...]]],
    quota: int,
    rng: random.Random,
) -> list[tuple[str, ...]]:
    """Round-robin across theme buckets in a band so rare themes aren't drowned out."""
    keys = sorted(buckets.keys())
    queues = {}
    for k in keys:
        items = list(buckets[k])
        rng.shuffle(items)
        queues[k] = items

    selected: list[tuple[str, ...]] = []
    while len(selected) < quota:
        progressed = False
        for k in keys:
            if queues[k]:
                selected.append(queues[k].pop())
                progressed = True
                if len(selected) >= quota:
                    break
        if not progressed:
            break
    return selected


def iter_csv_lines(url: str, input_path: str | None):
    if input_path:
        if input_path.endswith(".zst"):
            proc = subprocess.Popen(
                ["zstd", "-dc", input_path],
                stdout=subprocess.PIPE,
                bufsize=1024 * 1024,
            )
            assert proc.stdout is not None
            try:
                for line in io.TextIOWrapper(proc.stdout, encoding="utf-8", errors="replace"):
                    yield line
            finally:
                proc.wait()
                if proc.returncode != 0:
                    raise RuntimeError(f"zstd exited with {proc.returncode}")
        else:
            with open(input_path, "r", encoding="utf-8", errors="replace") as f:
                for line in f:
                    yield line
        return

    curl_proc = subprocess.Popen(
        ["curl", "-fsSL", url],
        stdout=subprocess.PIPE,
        bufsize=1024 * 1024,
    )
    zstd_proc = subprocess.Popen(
        ["zstd", "-dc"],
        stdin=curl_proc.stdout,
        stdout=subprocess.PIPE,
        bufsize=1024 * 1024,
    )
    assert curl_proc.stdout is not None
    curl_proc.stdout.close()
    assert zstd_proc.stdout is not None

    try:
        for line in io.TextIOWrapper(zstd_proc.stdout, encoding="utf-8", errors="replace"):
            yield line
    finally:
        zstd_rc = zstd_proc.wait()
        curl_rc = curl_proc.wait()
        if zstd_rc != 0 or curl_rc != 0:
            raise RuntimeError(f"stream failed (curl={curl_rc}, zstd={zstd_rc})")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default=LICHESS_PUZZLE_URL)
    parser.add_argument("--input", default=None, help="Optional local .csv or .csv.zst")
    parser.add_argument(
        "--output",
        default=os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "assets",
            "puzzles",
            "lichess_puzzles.tsv.gz",
        ),
    )
    parser.add_argument("--target", type=int, default=10000)
    parser.add_argument("--min-popularity", type=int, default=81)
    parser.add_argument("--min-plays", type=int, default=501)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    rng = random.Random(args.seed)
    per_cell_cap = 80
    reservoirs: list[dict[str, list[tuple[str, ...]]]] = [
        defaultdict(list) for _ in BANDS
    ]
    seen_counts: list[dict[str, int]] = [defaultdict(int) for _ in BANDS]

    total_rows = 0
    matched_rows = 0

    reader = csv.reader(iter_csv_lines(args.url, args.input))
    header = next(reader, None)
    if header is None:
        raise RuntimeError("Empty input stream")

    col_idx = {name: i for i, name in enumerate(header)}
    idx_id = col_idx["PuzzleId"]
    idx_fen = col_idx["FEN"]
    idx_moves = col_idx["Moves"]
    idx_rating = col_idx["Rating"]
    idx_rd = col_idx["RatingDeviation"]
    idx_pop = col_idx["Popularity"]
    idx_plays = col_idx["NbPlays"]
    idx_themes = col_idx["Themes"]
    idx_opening = col_idx.get("OpeningTags")

    for row in reader:
        total_rows += 1
        if len(row) <= max(idx_id, idx_fen, idx_moves, idx_rating, idx_rd, idx_pop, idx_plays, idx_themes):
            continue

        try:
            rating = int(row[idx_rating])
            rd = int(row[idx_rd])
            pop = int(row[idx_pop])
            plays = int(row[idx_plays])
        except ValueError:
            continue

        if pop < args.min_popularity or plays < args.min_plays:
            continue

        band = rating_band(rating)
        if band is None:
            continue

        moves = row[idx_moves].strip()
        if len(moves.split()) < 2:
            continue

        fen = row[idx_fen].strip()
        pid = row[idx_id].strip()
        themes = row[idx_themes].strip()
        opening_tags = (
            row[idx_opening].strip()
            if idx_opening is not None and idx_opening < len(row)
            else ""
        )

        # Strip any accidental tabs/newlines
        record = (
            pid.replace("\t", " "),
            fen.replace("\t", " "),
            moves.replace("\t", " "),
            str(rating),
            str(rd),
            str(pop),
            str(plays),
            themes.replace("\t", " "),
            opening_tags.replace("\t", " "),
        )
        matched_rows += 1
        theme_key = primary_theme(themes)
        seen_counts[band][theme_key] = reservoir_insert(
            reservoirs[band][theme_key],
            seen_counts[band][theme_key],
            record,
            per_cell_cap,
            rng,
        )

    # First pass: give every band an equal base share, redistributing shortfall
    # if any extreme band has fewer total candidates than its share.
    base_quota = args.target // len(BANDS)
    rem = args.target % len(BANDS)
    band_quotas = [base_quota + (1 if i < rem else 0) for i in range(len(BANDS))]

    band_pools = [
        pick_balanced_from_band(reservoirs[i], sum(len(v) for v in reservoirs[i].values()), rng)
        for i in range(len(BANDS))
    ]

    selected_by_band: list[list[tuple[str, ...]]] = [[] for _ in BANDS]
    shortfall = 0
    for i in range(len(BANDS)):
        take = min(band_quotas[i], len(band_pools[i]))
        selected_by_band[i] = band_pools[i][:take]
        band_pools[i] = band_pools[i][take:]
        shortfall += band_quotas[i] - take

    # Redistribute any shortfall across bands that still have candidates
    while shortfall > 0:
        progressed = False
        for i in range(len(BANDS)):
            if band_pools[i]:
                selected_by_band[i].append(band_pools[i].pop(0))
                shortfall -= 1
                progressed = True
                if shortfall == 0:
                    break
        if not progressed:
            break

    final_rows: list[tuple[str, ...]] = []
    for band_rows in selected_by_band:
        final_rows.extend(band_rows)

    # Sort deterministically by rating then id
    final_rows.sort(key=lambda r: (int(r[3]), r[0]))

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with gzip.open(args.output, "wt", encoding="utf-8", compresslevel=9) as gz:
        gz.write("id\tfen\tmoves\trating\trating_deviation\tpopularity\tnb_plays\tthemes\topening_tags\n")
        for rec in final_rows:
            gz.write("\t".join(rec) + "\n")

    size_bytes = os.path.getsize(args.output)
    print(f"Streamed {total_rows:,} CSV rows; {matched_rows:,} matched filters (pop>={args.min_popularity}, plays>={args.min_plays}, rating 600-2800).")
    print(f"Sampled {len(final_rows):,} puzzles -> {args.output} ({size_bytes:,} bytes, {size_bytes / 1024:.1f} KiB)")

    print("\nRating distribution:")
    for i, (lo, hi) in enumerate(BANDS):
        print(f"  {lo:4d}-{hi:4d}: {len(selected_by_band[i]):5d}")

    theme_counts: Counter[str] = Counter()
    for rec in final_rows:
        for t in rec[7].split():
            theme_counts[t] += 1
    print("\nTop 15 themes:")
    for t, c in theme_counts.most_common(15):
        print(f"  {t:20s}: {c:5d}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
