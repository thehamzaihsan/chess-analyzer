use chess_analyzer::analysis::{analyze_game, AnalyzedMove, GameAnalysis, GamePhase, MoveClassification};
use chess_analyzer::db::Database;
use chess_analyzer::engine::UciEval;
use chess_analyzer::pgn::parse_pgn_games;
use chess_analyzer::puzzle_mining::{backfill_puzzles_from_db, mine_puzzles_from_analysis};
use chess_analyzer::puzzles_seed::{
    needs_lichess_seed, seed_lichess_from_tsv_if_needed, LICHESS_SEED_VERSION_KEY,
};
use shakmaty::{fen::Fen, uci::UciMove, CastlingMode, Chess, Position};
use std::collections::HashSet;
use std::io::Cursor;

const TINY_LICHESS_FIXTURE: &str = "\
id\tfen\tmoves\trating\trating_deviation\tpopularity\tnb_plays\tthemes\topening_tags
00008\tr6k/pp2r2p/4Rp1Q/3p4/8/1N1P2R1/PqP2bPP/7K b - - 0 24\tf2g3 e6e7 b2b1 b3c1 b1c1 h6c1\t1819\t77\t95\t10446\tcrushing hangingPiece long middlegame\t
0000H\tr4rk1/pp3ppp/8/2bqp2b/3N4/P2Bn2P/1PPN1P2/R1BQ1RK1 w - - 1 15\td1h5 d5g2\t1032\t78\t100\t92\tmate mateIn1 middlegame oneMove\tFrench_Defense French_Defense_Tarrasch_Variation
0009B\tr2qr1k1/b1p2ppp/pp4n1/P1P1p3/4P1n1/B2P2Pb/3NBP1P/RN1QR1K1 b - - 1 16\tb6c5 e2g4 h3g4 d1g4\t1061\t75\t88\t617\tadvantage middlegame short\tKings_Pawn_Game Kings_Pawn_Game_Leonardis_Variation
";

#[test]
fn seed_import_from_tiny_fixture_and_selection_avoids_repeats() {
    let db = Database::memory().unwrap();
    assert!(needs_lichess_seed(&db, "1").unwrap());

    let inserted =
        seed_lichess_from_tsv_if_needed(&db, Cursor::new(TINY_LICHESS_FIXTURE), "1").unwrap();
    assert_eq!(inserted, 3);
    assert_eq!(db.count_puzzles_by_source("lichess").unwrap(), 3);
    assert_eq!(
        db.get_session_value(LICHESS_SEED_VERSION_KEY)
            .unwrap()
            .as_deref(),
        Some("1")
    );
    assert!(!needs_lichess_seed(&db, "1").unwrap());

    // Selection avoids repeats across all 3 unseen puzzles.
    let mut seen_ids = HashSet::new();
    for _ in 0..3 {
        let p = db.get_next_puzzle(Some("lichess"), None).unwrap().unwrap();
        assert_eq!(p.source, "lichess");
        // Verify FEN is before moves[0] (setup) and moves[1] is legal after moves[0]
        let pos: Chess = p
            .fen
            .parse::<Fen>()
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap();
        let moves: Vec<&str> = p.moves.split_whitespace().collect();
        let setup = moves[0].parse::<UciMove>().unwrap().to_move(&pos).unwrap();
        let after_setup = pos.play(setup).unwrap();
        let _solver = moves[1]
            .parse::<UciMove>()
            .unwrap()
            .to_move(&after_setup)
            .unwrap();
        assert!(seen_ids.insert(p.id));
    }
    assert_eq!(seen_ids.len(), 3);

    // Record attempts: solve two of them, fail one -> next selection prefers the failed one.
    let mut ids: Vec<String> = seen_ids.into_iter().collect();
    ids.sort();
    db.record_puzzle_attempt(&ids[0], true).unwrap();
    db.record_puzzle_attempt(&ids[1], true).unwrap();
    db.record_puzzle_attempt(&ids[2], false).unwrap();

    let next = db.get_next_puzzle(Some("lichess"), None).unwrap().unwrap();
    assert_eq!(next.id, ids[2]);
}

#[test]
fn target_rating_selection_prefers_matching_band() {
    let db = Database::memory().unwrap();
    seed_lichess_from_tsv_if_needed(&db, Cursor::new(TINY_LICHESS_FIXTURE), "1").unwrap();

    // Target 1800 should pick the 1819 puzzle ("lichess_00008")
    let high = db
        .get_next_puzzle(Some("lichess"), Some(1800))
        .unwrap()
        .unwrap();
    assert_eq!(high.id, "lichess_00008");
    assert_eq!(high.rating, 1819);

    // Target 1050 should pick one of the ~1032/1061 puzzles with opening tags preserved
    let low = db
        .get_next_puzzle(Some("lichess"), Some(1050))
        .unwrap()
        .unwrap();
    assert!(low.rating < 1100);
    assert!(!low.opening_tags.is_empty());
}

#[test]
fn mining_and_startup_backfill_only_mine_tracked_player_moves() {
    let db = Database::memory().unwrap();
    let pgn = "1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0\n";
    let parsed = &parse_pgn_games(pgn)[0];

    let game_id = db
        .insert_game(
            "HeroPlayer",
            "OpponentPlayer",
            "1-0",
            Some(1400),
            Some(1450),
            "2026.02.01",
            "600",
            "C20",
            "King's Pawn Game",
            pgn,
        )
        .unwrap();

    // Construct an analysis with:
    // - HeroPlayer (White) ply 3: Mistake with legal best move g1f3
    // - HeroPlayer (White) ply 5: Blunder with "(none)" -> skipped
    // - OpponentPlayer (Black) ply 6: Blunder with legal best move g7g6 -> ignored for HeroPlayer
    let mut analysis: GameAnalysis = analyze_game(&parsed.moves, &[]);
    analysis.moves = vec![
        AnalyzedMove {
            ply: 3,
            move_number: 2,
            is_white: true,
            san: parsed.moves[2].san.clone(),
            uci: parsed.moves[2].uci.clone(),
            fen_before: parsed.moves[2].fen_before.clone(),
            fen_after: parsed.moves[2].fen_after.clone(),
            eval_before: Some(UciEval {
                depth: 12,
                score_cp: Some(25),
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
            ply: 5,
            move_number: 3,
            is_white: true,
            san: parsed.moves[4].san.clone(),
            uci: parsed.moves[4].uci.clone(),
            fen_before: parsed.moves[4].fen_before.clone(),
            fen_after: parsed.moves[4].fen_after.clone(),
            eval_before: Some(UciEval {
                depth: 12,
                score_cp: Some(25),
                mate_in: None,
                best_move: Some("(none)".to_string()),
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
            san: parsed.moves[5].san.clone(),
            uci: parsed.moves[5].uci.clone(),
            fen_before: parsed.moves[5].fen_before.clone(),
            fen_after: parsed.moves[5].fen_after.clone(),
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

    db.save_analysis(game_id, &analysis).unwrap();

    // Startup backfill from DB should mine only HeroPlayer's valid mistake (ply 3)
    let backfilled = backfill_puzzles_from_db(&db, "HeroPlayer");
    assert_eq!(backfilled, 1);
    assert_eq!(db.get_puzzle_count().unwrap(), (1, 1));

    let mined_puzzle = db.get_random_puzzle(Some("blunder")).unwrap().unwrap();
    assert_eq!(mined_puzzle.id, format!("blunder_{}_3", game_id));
    assert_eq!(mined_puzzle.moves, "g1f3");

    // Calling mine_puzzles_from_analysis again is idempotent (0 new)
    let stored = db.get_game(game_id).unwrap().unwrap();
    assert_eq!(
        mine_puzzles_from_analysis(&db, &stored, &analysis, "HeroPlayer"),
        0
    );
}
