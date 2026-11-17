use crate::analysis::{GameAnalysis, MoveClassification};
use crate::db::{Database, Puzzle, StoredGame};
use shakmaty::{uci::UciMove, CastlingMode, Chess, fen::Fen};
use std::collections::HashMap;

/// Mines blunder and mistake puzzles for `player_username` from a single analyzed game.
///
/// Only moves played by `player_username` (case-insensitive match against `game.white`
/// or `game.black`) classified as `Blunder` or `Mistake` are mined. The solution is
/// `eval_before.best_move`; moves without a legal UCI best move in `fen_before` (or with
/// `"(none)"` / `"0000"`) are skipped. Uses `Database::insert_puzzle_if_new` for
/// deterministic deduplication.
pub fn mine_puzzles_from_analysis(
    db: &Database,
    game: &StoredGame,
    analysis: &GameAnalysis,
    player_username: &str,
) -> usize {
    let uname = player_username.trim();
    if uname.is_empty() {
        return 0;
    }

    let mine_white = game.white.eq_ignore_ascii_case(uname);
    let mine_black = game.black.eq_ignore_ascii_case(uname);
    if !mine_white && !mine_black {
        return 0;
    }

    let mut mined = 0usize;

    for m in &analysis.moves {
        if m.is_white && !mine_white {
            continue;
        }
        if !m.is_white && !mine_black {
            continue;
        }
        if !matches!(
            m.classification,
            MoveClassification::Blunder | MoveClassification::Mistake
        ) {
            continue;
        }

        let Some(best_raw) = m.eval_before.as_ref().and_then(|e| e.best_move.as_deref()) else {
            continue;
        };
        let best_trimmed = best_raw.trim();
        if best_trimmed.is_empty() || best_trimmed == "(none)" || best_trimmed == "0000" {
            continue;
        }

        let is_legal = m
            .fen_before
            .parse::<Fen>()
            .ok()
            .and_then(|f| f.into_position::<Chess>(CastlingMode::Standard).ok())
            .and_then(|pos| {
                best_trimmed
                    .parse::<UciMove>()
                    .ok()
                    .and_then(|u| u.to_move(&pos).ok())
            })
            .is_some();

        if !is_legal {
            continue;
        }

        let rating = if m.is_white {
            game.white_elo
        } else {
            game.black_elo
        }
        .unwrap_or(1500)
        .clamp(400, 3000);

        let theme = match m.classification {
            MoveClassification::Blunder => "blunder-recovery",
            MoveClassification::Mistake => "mistake-recovery",
            _ => "blunder-recovery",
        };

        let opening_tags = if !game.opening.trim().is_empty() {
            game.opening.trim().to_string()
        } else {
            analysis.opening_name.clone()
        };

        let puzzle = Puzzle {
            id: format!("blunder_{}_{}", game.id, m.ply),
            fen: m.fen_before.clone(),
            moves: best_trimmed.to_string(),
            rating,
            rating_deviation: 100,
            popularity: 100,
            nb_plays: 1,
            themes: theme.to_string(),
            opening_tags,
            source: "blunder".to_string(),
        };

        if let Ok(true) = db.insert_puzzle_if_new(&puzzle) {
            mined += 1;
        }
    }

    mined
}

/// Mines blunder and mistake puzzles across a slice of games and their in-memory analyses.
pub fn mine_puzzles_from_games(
    db: &Database,
    games: &[StoredGame],
    analyses: &HashMap<i64, GameAnalysis>,
    player_username: &str,
) -> usize {
    let mut total_mined = 0usize;
    for g in games {
        if let Some(an) = analyses.get(&g.id) {
            total_mined += mine_puzzles_from_analysis(db, g, an, player_username);
        }
    }
    total_mined
}

/// Backfills blunder and mistake puzzles from all analyzed games stored in `db`
/// for `player_username`.
pub fn backfill_puzzles_from_db(db: &Database, player_username: &str) -> usize {
    let uname = player_username.trim();
    if uname.is_empty() {
        return 0;
    }

    let Ok(games) = db.list_games() else {
        return 0;
    };

    let mut total_mined = 0usize;
    for g in &games {
        if !g.white.eq_ignore_ascii_case(uname) && !g.black.eq_ignore_ascii_case(uname) {
            continue;
        }
        if let Some(ref json) = g.analysis_json {
            if let Ok(an) = serde_json::from_str::<GameAnalysis>(json) {
                total_mined += mine_puzzles_from_analysis(db, g, &an, uname);
            }
        }
    }
    total_mined
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{analyze_game, AnalyzedMove, GamePhase};
    use crate::engine::UciEval;
    use crate::pgn::parse_pgn_games;

    fn sample_stored_game(id: i64, white: &str, black: &str) -> StoredGame {
        StoredGame {
            id,
            pgn: "1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7#".to_string(),
            white: white.to_string(),
            black: black.to_string(),
            white_elo: Some(1350),
            black_elo: Some(1420),
            result: "1-0".to_string(),
            date: "2026.01.01".to_string(),
            time_control: "600".to_string(),
            eco: "C20".to_string(),
            opening: "King's Pawn Game".to_string(),
            accuracy_white: Some(85.0),
            accuracy_black: Some(60.0),
            analysis_json: None,
        }
    }

    #[test]
    fn mining_only_takes_tracked_players_blunders_and_mistakes() {
        let db = Database::memory().unwrap();
        let pgn = "1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0\n";
        let moves = &parse_pgn_games(pgn)[0].moves;

        // Build synthetic analyzed moves:
        // - ply 3 (White, move 2): Mistake with legal best_move "g1f3"
        // - ply 6 (Black, move 3... Nf6): Blunder with legal best_move "g7g6"
        // - ply 4 (Black, move 2... Nc6): Blunder with "(none)" -> skipped
        // - ply 5 (White, move 3. Bc4): Blunder with illegal best_move "a1h8" -> skipped
        let analyzed_moves = vec![
            AnalyzedMove {
                ply: 3,
                move_number: 2,
                is_white: true,
                san: moves[2].san.clone(),
                uci: moves[2].uci.clone(),
                fen_before: moves[2].fen_before.clone(),
                fen_after: moves[2].fen_after.clone(),
                eval_before: Some(UciEval {
                    depth: 12,
                    score_cp: Some(20),
                    mate_in: None,
                    best_move: Some("g1f3".to_string()),
                    pv: vec!["g1f3".to_string()],
                }),
                eval_after: None,
                win_prob_before: 55.0,
                win_prob_after: 35.0,
                win_prob_loss: 20.0,
                classification: MoveClassification::Mistake,
                phase: GamePhase::Opening,
                clock_seconds: None,
            },
            AnalyzedMove {
                ply: 4,
                move_number: 2,
                is_white: false,
                san: moves[3].san.clone(),
                uci: moves[3].uci.clone(),
                fen_before: moves[3].fen_before.clone(),
                fen_after: moves[3].fen_after.clone(),
                eval_before: Some(UciEval {
                    depth: 12,
                    score_cp: Some(0),
                    mate_in: None,
                    best_move: Some("(none)".to_string()),
                    pv: vec![],
                }),
                eval_after: None,
                win_prob_before: 50.0,
                win_prob_after: 80.0,
                win_prob_loss: 30.0,
                classification: MoveClassification::Blunder,
                phase: GamePhase::Opening,
                clock_seconds: None,
            },
            AnalyzedMove {
                ply: 5,
                move_number: 3,
                is_white: true,
                san: moves[4].san.clone(),
                uci: moves[4].uci.clone(),
                fen_before: moves[4].fen_before.clone(),
                fen_after: moves[4].fen_after.clone(),
                eval_before: Some(UciEval {
                    depth: 12,
                    score_cp: Some(20),
                    mate_in: None,
                    best_move: Some("a1h8".to_string()), // illegal!
                    pv: vec![],
                }),
                eval_after: None,
                win_prob_before: 55.0,
                win_prob_after: 20.0,
                win_prob_loss: 35.0,
                classification: MoveClassification::Blunder,
                phase: GamePhase::Opening,
                clock_seconds: None,
            },
            AnalyzedMove {
                ply: 6,
                move_number: 3,
                is_white: false,
                san: moves[5].san.clone(),
                uci: moves[5].uci.clone(),
                fen_before: moves[5].fen_before.clone(),
                fen_after: moves[5].fen_after.clone(),
                eval_before: Some(UciEval {
                    depth: 12,
                    score_cp: Some(15),
                    mate_in: None,
                    best_move: Some("g7g6".to_string()),
                    pv: vec!["g7g6".to_string()],
                }),
                eval_after: None,
                win_prob_before: 48.0,
                win_prob_after: 0.0,
                win_prob_loss: 48.0,
                classification: MoveClassification::Blunder,
                phase: GamePhase::Opening,
                clock_seconds: None,
            },
        ];

        let mut analysis = analyze_game(moves, &[]);
        analysis.moves = analyzed_moves;

        let game = sample_stored_game(10, "Alice", "Bob");

        // Mining for Alice (White) must only take Alice's legal Mistake (g1f3),
        // skipping Alice's illegal best_move (a1h8) and ignoring Bob's moves.
        let count_alice = mine_puzzles_from_analysis(&db, &game, &analysis, "alice");
        assert_eq!(count_alice, 1);
        let p = db.get_random_puzzle(Some("blunder")).unwrap().unwrap();
        assert_eq!(p.id, "blunder_10_3");
        assert_eq!(p.moves, "g1f3");
        assert_eq!(p.themes, "mistake-recovery");
        assert_eq!(p.rating, 1350);

        // Re-mining for Alice produces 0 new puzzles due to insert_puzzle_if_new.
        assert_eq!(mine_puzzles_from_analysis(&db, &game, &analysis, "Alice"), 0);

        // Mining for Bob (Black) takes only Bob's valid blunder (g7g6), skipping "(none)".
        let count_bob = mine_puzzles_from_analysis(&db, &game, &analysis, "BOB");
        assert_eq!(count_bob, 1);
        assert_eq!(db.get_puzzle_count().unwrap(), (2, 2));

        // Mining for an unrelated player yields 0.
        assert_eq!(mine_puzzles_from_analysis(&db, &game, &analysis, "Charlie"), 0);
    }
}
