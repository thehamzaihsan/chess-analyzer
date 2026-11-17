use crate::db::{Database, Puzzle};
use anyhow::Result;
use flate2::read::GzDecoder;
use std::io::{BufRead, BufReader};

pub const LICHESS_SEED_VERSION_KEY: &str = "lichess_puzzle_seed_version";
pub const BUNDLED_LICHESS_SEED_VERSION: &str = "1";

pub const BUNDLED_LICHESS_PUZZLES_GZ: &[u8] =
    include_bytes!("../assets/puzzles/lichess_puzzles.tsv.gz");

/// Parses Lichess puzzles from a TSV stream (`id\tfen\tmoves\trating\trating_deviation\tpopularity\tnb_plays\tthemes\topening_tags`).
///
/// Preserves Lichess puzzle semantics: `fen` is the position BEFORE the opponent's
/// setup move `moves[0]`, and `moves[1..]` is the solver's solution line.
pub fn parse_lichess_tsv<R: BufRead>(reader: R) -> Result<Vec<Puzzle>> {
    let mut puzzles = Vec::new();

    for (line_idx, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
        if trimmed.is_empty() {
            continue;
        }

        if line_idx == 0 && (trimmed.starts_with("id\t") || trimmed.starts_with("PuzzleId\t")) {
            continue;
        }

        let cols: Vec<&str> = trimmed.split('\t').collect();
        if cols.len() < 8 {
            continue;
        }

        let raw_id = cols[0].trim();
        let fen = cols[1].trim();
        let moves = cols[2].trim();
        if raw_id.is_empty() || fen.is_empty() || moves.split_whitespace().count() < 2 {
            continue;
        }

        let Ok(rating) = cols[3].trim().parse::<i32>() else {
            continue;
        };
        let Ok(rating_deviation) = cols[4].trim().parse::<i32>() else {
            continue;
        };
        let Ok(popularity) = cols[5].trim().parse::<i32>() else {
            continue;
        };
        let Ok(nb_plays) = cols[6].trim().parse::<i32>() else {
            continue;
        };

        let themes = cols[7].trim().to_string();
        let opening_tags = cols.get(8).map(|s| s.trim()).unwrap_or("").to_string();
        let id = if raw_id.starts_with("lichess_") {
            raw_id.to_string()
        } else {
            format!("lichess_{}", raw_id)
        };

        puzzles.push(Puzzle {
            id,
            fen: fen.to_string(),
            moves: moves.to_string(),
            rating,
            rating_deviation,
            popularity,
            nb_plays,
            themes,
            opening_tags,
            source: "lichess".to_string(),
        });
    }

    Ok(puzzles)
}

/// Decompresses a gzipped TSV buffer and parses its Lichess puzzles.
pub fn parse_lichess_tsv_gz(gz_bytes: &[u8]) -> Result<Vec<Puzzle>> {
    let decoder = GzDecoder::new(gz_bytes);
    let reader = BufReader::new(decoder);
    parse_lichess_tsv(reader)
}

/// Returns `true` if the database has no `source = 'lichess'` puzzles or
/// was seeded with an older seed version.
pub fn needs_lichess_seed(db: &Database, target_version: &str) -> Result<bool> {
    let lichess_count = db.count_puzzles_by_source("lichess")?;
    if lichess_count == 0 {
        return Ok(true);
    }
    let current_ver = db.get_session_value(LICHESS_SEED_VERSION_KEY)?;
    Ok(current_ver.as_deref() != Some(target_version))
}

/// Seeds Lichess puzzles from a TSV reader in a single transaction if needed.
pub fn seed_lichess_from_tsv_if_needed<R: BufRead>(
    db: &Database,
    reader: R,
    target_version: &str,
) -> Result<usize> {
    if !needs_lichess_seed(db, target_version)? {
        return Ok(0);
    }
    let puzzles = parse_lichess_tsv(reader)?;
    if puzzles.is_empty() {
        return Ok(0);
    }
    db.import_puzzles_batch(&puzzles, Some((LICHESS_SEED_VERSION_KEY, target_version)))
}

/// Seeds Lichess puzzles from gzipped TSV bytes in a single transaction if needed.
pub fn seed_lichess_from_gz_if_needed(
    db: &Database,
    gz_bytes: &[u8],
    target_version: &str,
) -> Result<usize> {
    if !needs_lichess_seed(db, target_version)? {
        return Ok(0);
    }
    let puzzles = parse_lichess_tsv_gz(gz_bytes)?;
    if puzzles.is_empty() {
        return Ok(0);
    }
    db.import_puzzles_batch(&puzzles, Some((LICHESS_SEED_VERSION_KEY, target_version)))
}

/// Seeds the bundled CC0 Lichess puzzle dataset on startup if the DB has no
/// `source = 'lichess'` puzzles or has an older seed version.
pub fn seed_bundled_lichess_puzzles_if_needed(db: &Database) -> Result<usize> {
    seed_lichess_from_gz_if_needed(
        db,
        BUNDLED_LICHESS_PUZZLES_GZ,
        BUNDLED_LICHESS_SEED_VERSION,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{fen::Fen, uci::UciMove, CastlingMode, Chess, Position};
    use std::io::Cursor;

    const TINY_FIXTURE_TSV: &str = "\
id\tfen\tmoves\trating\trating_deviation\tpopularity\tnb_plays\tthemes\topening_tags
0000H\tr4rk1/pp3ppp/8/2bqp2b/3N4/P2Bn2P/1PPN1P2/R1BQ1RK1 w - - 1 15\td1h5 d5g2\t1032\t78\t100\t92\tmate mateIn1 middlegame oneMove\tFrench_Defense French_Defense_Tarrasch_Variation
0009B\tr2qr1k1/b1p2ppp/pp4n1/P1P1p3/4P1n1/B2P2Pb/3NBP1P/RN1QR1K1 b - - 1 16\tb6c5 e2g4 h3g4 d1g4\t1061\t75\t88\t617\tadvantage middlegame short\tKings_Pawn_Game Kings_Pawn_Game_Leonardis_Variation
";

    #[test]
    fn seed_import_from_tiny_fixture_and_version_check() {
        let db = Database::memory().unwrap();
        assert!(needs_lichess_seed(&db, "1").unwrap());

        let imported =
            seed_lichess_from_tsv_if_needed(&db, Cursor::new(TINY_FIXTURE_TSV), "1").unwrap();
        assert_eq!(imported, 2);
        assert_eq!(db.count_puzzles_by_source("lichess").unwrap(), 2);
        assert!(!needs_lichess_seed(&db, "1").unwrap());

        // Subsequent run with same version is a no-op
        let second =
            seed_lichess_from_tsv_if_needed(&db, Cursor::new(TINY_FIXTURE_TSV), "1").unwrap();
        assert_eq!(second, 0);

        // Older/different version triggers re-seed
        assert!(needs_lichess_seed(&db, "2").unwrap());
        let upgraded =
            seed_lichess_from_tsv_if_needed(&db, Cursor::new(TINY_FIXTURE_TSV), "2").unwrap();
        assert_eq!(upgraded, 2);
        assert_eq!(
            db.get_session_value(LICHESS_SEED_VERSION_KEY).unwrap().as_deref(),
            Some("2")
        );

        // Verify Lichess semantics: fen is pre-setup, moves[0] setup is legal, moves[1] solver reply is legal
        let p = db.get_next_puzzle(Some("lichess"), Some(1030)).unwrap().unwrap();
        assert_eq!(p.source, "lichess");
        assert!(!p.themes.is_empty());
        assert!(!p.opening_tags.is_empty());
        let pos: Chess = p
            .fen
            .parse::<Fen>()
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap();
        let moves: Vec<&str> = p.moves.split_whitespace().collect();
        assert!(moves.len() >= 2);
        let setup_m = moves[0].parse::<UciMove>().unwrap().to_move(&pos).unwrap();
        let pos_after_setup = pos.play(setup_m).unwrap();
        let _solver_m = moves[1]
            .parse::<UciMove>()
            .unwrap()
            .to_move(&pos_after_setup)
            .unwrap();
    }

    #[test]
    fn bundled_lichess_asset_parses_and_validates() {
        let puzzles = parse_lichess_tsv_gz(BUNDLED_LICHESS_PUZZLES_GZ).unwrap();
        assert!(
            puzzles.len() >= 9000,
            "expected ~10,000 bundled puzzles, got {}",
            puzzles.len()
        );

        // Check rating spread & filter bounds
        let min_r = puzzles.iter().map(|p| p.rating).min().unwrap();
        let max_r = puzzles.iter().map(|p| p.rating).max().unwrap();
        assert!(min_r >= 600 && min_r < 800);
        assert!(max_r <= 2800 && max_r >= 2600);

        // Verify every bundled puzzle has legal setup & solver moves
        for p in &puzzles {
            assert!(p.popularity > 80);
            assert!(p.nb_plays > 500);
            assert_eq!(p.source, "lichess");
            let pos: Chess = p
                .fen
                .parse::<Fen>()
                .unwrap()
                .into_position(CastlingMode::Standard)
                .unwrap();
            let moves: Vec<&str> = p.moves.split_whitespace().collect();
            assert!(moves.len() >= 2);
            let setup = moves[0].parse::<UciMove>().unwrap().to_move(&pos).unwrap();
            let after_setup = pos.play(setup).unwrap();
            let _reply = moves[1]
                .parse::<UciMove>()
                .unwrap()
                .to_move(&after_setup)
                .unwrap();
        }
    }
}
