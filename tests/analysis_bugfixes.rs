use chess_analyzer::analysis::{
    analyze_game, centipawns_to_win_prob, matched_opening_ply_count, MoveClassification,
};
use chess_analyzer::db::{CachedEval, Database};
use chess_analyzer::engine::{AnalysisJob, EnginePool, PoolProgress, UciEngine, UciEval};
use chess_analyzer::pgn::parse_pgn_games;
use std::path::PathBuf;

#[test]
fn test_evals_alignment_no_off_by_one() {
    // 1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6? (blunder allowing Qxf7#) 4. Qxf7#
    let pgn = "[Event \"Test\"]\n\n1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0\n";
    let games = parse_pgn_games(pgn);
    assert_eq!(games.len(), 1);
    let moves = &games[0].moves;
    assert_eq!(moves.len(), 7);

    // Construct n + 1 = 8 evaluations:
    // evals[0] = startpos (before 1. e4)
    // evals[1] = after 1. e4 / before 1... e5
    // evals[2] = after 1... e5 / before 2. Qh5
    // evals[3] = after 2. Qh5 / before 2... Nc6
    // evals[4] = after 2... Nc6 / before 3. Bc4
    // evals[5] = after 3. Bc4 / before 3... Nf6 (still equal, best move is g7g6)
    // evals[6] = after 3... Nf6 / before 4. Qxf7# (White has mate in 1: h5f7)
    // evals[7] = after 4. Qxf7# (Black is checkmated -> White won)
    let evals = vec![
        Some(UciEval {
            depth: 12,
            score_cp: Some(20),
            mate_in: None,
            best_move: Some("e2e4".to_string()),
            pv: vec!["e2e4".to_string()],
        }),
        Some(UciEval {
            depth: 12,
            score_cp: Some(25),
            mate_in: None,
            best_move: Some("e7e5".to_string()),
            pv: vec!["e7e5".to_string()],
        }),
        Some(UciEval {
            depth: 12,
            score_cp: Some(15),
            mate_in: None,
            best_move: Some("g1f3".to_string()),
            pv: vec!["g1f3".to_string()],
        }),
        Some(UciEval {
            depth: 12,
            score_cp: Some(-10),
            mate_in: None,
            best_move: Some("b8c6".to_string()),
            pv: vec!["b8c6".to_string()],
        }),
        Some(UciEval {
            depth: 12,
            score_cp: Some(10),
            mate_in: None,
            best_move: Some("f1c4".to_string()),
            pv: vec!["f1c4".to_string()],
        }),
        Some(UciEval {
            depth: 12,
            score_cp: Some(15),
            mate_in: None,
            best_move: Some("g7g6".to_string()),
            pv: vec!["g7g6".to_string()],
        }),
        Some(UciEval {
            depth: 12,
            score_cp: None,
            mate_in: Some(1),
            best_move: Some("h5f7".to_string()),
            pv: vec!["h5f7".to_string()],
        }),
        Some(UciEval {
            depth: 0,
            score_cp: None,
            mate_in: Some(1),
            best_move: None,
            pv: vec![],
        }),
    ];

    let analysis = analyze_game(moves, &evals);
    assert_eq!(analysis.moves.len(), 7);

    // Every move i must have eval_before == evals[i] and eval_after == evals[i + 1].
    for (i, analyzed) in analysis.moves.iter().enumerate() {
        let expected_before = evals[i].as_ref().unwrap();
        let expected_after = evals[i + 1].as_ref().unwrap();
        assert_eq!(
            analyzed.eval_before.as_ref().unwrap().best_move,
            expected_before.best_move,
            "move {} ({}) eval_before.best_move must match position before move {}",
            i,
            analyzed.san,
            i
        );
        assert_eq!(
            analyzed.eval_before.as_ref().unwrap().score_cp,
            expected_before.score_cp
        );
        assert_eq!(
            analyzed.eval_after.as_ref().unwrap().score_cp,
            expected_after.score_cp
        );
        assert_eq!(
            analyzed.eval_after.as_ref().unwrap().mate_in,
            expected_after.mate_in
        );
    }

    // Ply 5 (index 5, "Nf6") allowed mate in 1 -> Blunder, with best_move "g7g6".
    assert_eq!(analysis.moves[5].san, "Nf6");
    assert_eq!(analysis.moves[5].classification, MoveClassification::Blunder);
    assert_eq!(
        analysis.moves[5]
            .eval_before
            .as_ref()
            .and_then(|e| e.best_move.as_deref()),
        Some("g7g6")
    );

    // Ply 6 (index 6, "Qxf7#") delivered mate -> Best (NOT Blunder), with best_move "h5f7".
    assert_eq!(analysis.moves[6].san, "Qxf7#");
    assert_eq!(analysis.moves[6].classification, MoveClassification::Best);
    assert_eq!(analysis.moves[6].win_prob_loss, 0.0);
}

#[test]
fn test_analyze_game_missing_evals_robustness() {
    let pgn = "1. h4 a5 2. Rh3 *\n";
    let moves = &parse_pgn_games(pgn)[0].moves;
    assert_eq!(moves.len(), 3);

    // Pass only 2 evals for 3 moves (missing evals[2] and evals[3]).
    let short_evals = vec![
        Some(UciEval {
            depth: 10,
            score_cp: Some(600),
            ..Default::default()
        }),
        Some(UciEval {
            depth: 10,
            score_cp: Some(600),
            ..Default::default()
        }),
    ];

    let analysis = analyze_game(moves, &short_evals);
    assert_eq!(analysis.moves.len(), 3);
    // Move 1 (index 1) has eval_before = +600 and eval_after = None.
    // Must not fabricate a blunder against a 50% fallback.
    assert_ne!(analysis.moves[1].classification, MoveClassification::Blunder);
    assert_eq!(analysis.moves[1].win_prob_loss, 0.0);
    assert_eq!(analysis.white_blunders, 0);
    assert_eq!(analysis.black_blunders, 0);
}

#[test]
fn test_uci_checkmate_mate0_and_bestmove_none() {
    // Black is to move and checkmated (White delivered mate): `score mate 0`
    let mut white_mated_black = UciEval::default();
    UciEngine::parse_info_line("info depth 0 score mate 0", false, &mut white_mated_black);
    assert!(
        white_mated_black.mate_in.unwrap() > 0,
        "White delivering mate must have positive White-perspective mate_in, got {:?}",
        white_mated_black.mate_in
    );
    assert_eq!(white_mated_black.score_cp, None);
    assert_eq!(
        centipawns_to_win_prob(white_mated_black.score_cp, white_mated_black.mate_in),
        100.0
    );

    // White is to move and checkmated (Black delivered mate): `score mate 0`
    let mut black_mated_white = UciEval::default();
    UciEngine::parse_info_line("info depth 0 score mate 0", true, &mut black_mated_white);
    assert!(
        black_mated_white.mate_in.unwrap() < 0,
        "Black delivering mate must have negative White-perspective mate_in, got {:?}",
        black_mated_white.mate_in
    );
    assert_eq!(black_mated_white.score_cp, None);
    assert_eq!(
        centipawns_to_win_prob(black_mated_white.score_cp, black_mated_white.mate_in),
        0.0
    );

    // Non-zero mate scores still flip properly by side to move
    let mut eval = UciEval::default();
    UciEngine::parse_info_line("info depth 12 score mate 2 pv h5f7", true, &mut eval);
    assert_eq!(eval.mate_in, Some(2));
    assert_eq!(centipawns_to_win_prob(eval.score_cp, eval.mate_in), 100.0);

    let mut eval_b = UciEval::default();
    UciEngine::parse_info_line("info depth 12 score mate 2 pv d8h4", false, &mut eval_b);
    assert_eq!(eval_b.mate_in, Some(-2));
    assert_eq!(centipawns_to_win_prob(eval_b.score_cp, eval_b.mate_in), 0.0);

    // `bestmove (none)` and `bestmove 0000` must yield None
    assert_eq!(UciEngine::parse_bestmove_line("bestmove (none)"), None);
    assert_eq!(UciEngine::parse_bestmove_line("bestmove 0000"), None);
    assert_eq!(
        UciEngine::parse_bestmove_line("bestmove e2e4 ponder e7e5"),
        Some("e2e4".to_string())
    );
}

#[test]
fn test_black_delivering_mate_classified_as_best() {
    // Fool's mate: 1. f3 e5 2. g4 Qh4#
    let pgn = "1. f3 e5 2. g4 Qh4# 0-1\n";
    let moves = &parse_pgn_games(pgn)[0].moves;
    assert_eq!(moves.len(), 4);

    let mut eval_after_qh4 = UciEval::default();
    // White is to move in the checkmated position (`is_white_turn = true`)
    UciEngine::parse_info_line("info depth 0 score mate 0", true, &mut eval_after_qh4);
    eval_after_qh4.best_move = UciEngine::parse_bestmove_line("bestmove (none)");

    let evals = vec![
        Some(UciEval {
            depth: 12,
            score_cp: Some(20),
            best_move: Some("e2e4".to_string()),
            ..Default::default()
        }),
        Some(UciEval {
            depth: 12,
            score_cp: Some(-50),
            best_move: Some("e7e5".to_string()),
            ..Default::default()
        }),
        Some(UciEval {
            depth: 12,
            score_cp: Some(-60),
            best_move: Some("d2d4".to_string()),
            ..Default::default()
        }),
        Some(UciEval {
            depth: 12,
            mate_in: Some(-1),
            best_move: Some("d8h4".to_string()),
            ..Default::default()
        }),
        Some(eval_after_qh4),
    ];

    let analysis = analyze_game(moves, &evals);
    assert_eq!(analysis.moves[3].san, "Qh4#");
    assert_eq!(analysis.moves[3].classification, MoveClassification::Best);
    assert_eq!(analysis.moves[3].win_prob_loss, 0.0);
    assert_eq!(analysis.black_blunders, 0);
}

#[test]
fn test_book_heuristic_only_marks_matched_opening_plies() {
    // 1. Unclassified opening -> 0 book plies
    let unclassified_pgn = "1. h4 a5 2. g4 h5 *\n";
    let unclassified_moves = &parse_pgn_games(unclassified_pgn)[0].moves;
    let dummy_eval = || {
        Some(UciEval {
            depth: 10,
            score_cp: Some(0),
            ..Default::default()
        })
    };
    let unclassified_evals = vec![dummy_eval(); unclassified_moves.len() + 1];
    let unclassified_analysis = analyze_game(unclassified_moves, &unclassified_evals);
    assert!(
        unclassified_analysis
            .moves
            .iter()
            .all(|m| m.classification != MoveClassification::Book),
        "Unclassified openings must not mark any moves as Book"
    );

    // 2. 2-ply opening ("King's Pawn Game": e4 e5), followed by non-book moves 2. a3 Bc5
    let two_ply_pgn = "1. e4 e5 2. a3 Bc5 3. b4 Bb6 *\n";
    let two_ply_moves = &parse_pgn_games(two_ply_pgn)[0].moves;
    let mut evals = vec![dummy_eval(); two_ply_moves.len() + 1];
    // Make ply 2 (2. a3) a blunder (+20cp -> -300cp) so we verify it is NOT hidden by Book
    evals[2] = Some(UciEval {
        depth: 10,
        score_cp: Some(20),
        best_move: Some("g1f3".to_string()),
        ..Default::default()
    });
    evals[3] = Some(UciEval {
        depth: 10,
        score_cp: Some(-300),
        best_move: Some("g8f6".to_string()),
        ..Default::default()
    });

    let two_ply_analysis = analyze_game(two_ply_moves, &evals);
    assert_eq!(two_ply_analysis.opening_name, "King's Pawn Game");
    assert_eq!(two_ply_analysis.moves[0].classification, MoveClassification::Book);
    assert_eq!(two_ply_analysis.moves[1].classification, MoveClassification::Book);
    assert_ne!(
        two_ply_analysis.moves[2].classification,
        MoveClassification::Book,
        "Ply 2 is outside the 2-ply matched opening line and must not be Book"
    );
    assert_eq!(
        two_ply_analysis.moves[2].classification,
        MoveClassification::Blunder
    );

    // 3. 10-ply opening (Sicilian Najdorf) -> all 10 plies are Book, ply 10 is not
    let najdorf_pgn = "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. Be3 *\n";
    let najdorf_moves = &parse_pgn_games(najdorf_pgn)[0].moves;
    let sans: Vec<String> = najdorf_moves.iter().map(|m| m.san.clone()).collect();
    assert_eq!(matched_opening_ply_count(&sans), 10);

    let najdorf_evals = vec![dummy_eval(); najdorf_moves.len() + 1];
    let najdorf_analysis = analyze_game(najdorf_moves, &najdorf_evals);
    for i in 0..10 {
        assert_eq!(
            najdorf_analysis.moves[i].classification,
            MoveClassification::Book,
            "ply {} should be Book in 10-ply Najdorf line",
            i
        );
    }
    assert_ne!(
        najdorf_analysis.moves[10].classification,
        MoveClassification::Book
    );
}

#[tokio::test]
async fn test_engine_pool_serves_cached_evals_without_spawning_engine() {
    let db = Database::memory().expect("in-memory db");
    let pgn = "1. e4 e5 2. Nf3 Nc6 *\n";
    let moves = &parse_pgn_games(pgn)[0].moves;

    let mut fens = Vec::with_capacity(moves.len() + 1);
    fens.push(moves[0].fen_before.clone());
    for m in moves {
        fens.push(m.fen_after.clone());
    }

    for (idx, fen) in fens.iter().enumerate() {
        db.set_cached_eval(&CachedEval {
            fen: fen.clone(),
            depth: 12,
            score_cp: Some((idx as i32) * 15),
            mate_in: None,
            best_move: Some("e2e4".to_string()),
            pv: vec!["e2e4".to_string()],
        })
        .expect("cache insert");
    }

    // Use a nonexistent binary path to guarantee no Stockfish process is spawned on cache hits.
    let pool = EnginePool::with_engine_path(db, PathBuf::from("/nonexistent/stockfish"), 2);
    let jobs: Vec<AnalysisJob> = fens
        .iter()
        .enumerate()
        .map(|(idx, fen)| AnalysisJob {
            game_id: 42,
            move_idx: idx,
            fen: fen.clone(),
            depth: 12,
            movetime_ms: None,
        })
        .collect();

    let mut rx = pool.analyze_batch(jobs);
    let mut evals = vec![None; fens.len()];
    let mut finished = false;

    while let Some(msg) = rx.recv().await {
        match msg {
            PoolProgress::Progress { result, .. } => {
                assert!(result.was_cached, "Expected cached hit for idx {}", result.move_idx);
                evals[result.move_idx] = Some(result.eval);
            }
            PoolProgress::Error(e) => panic!("Unexpected pool error on cached batch: {}", e),
            PoolProgress::Finished { total } => {
                assert_eq!(total, fens.len());
                finished = true;
                break;
            }
            PoolProgress::Started { total } => {
                assert_eq!(total, fens.len());
            }
        }
    }

    assert!(finished);
    assert!(evals.iter().all(|e| e.is_some()));

    let analysis = analyze_game(moves, &evals);
    assert_eq!(analysis.moves.len(), moves.len());
    for (i, m) in analysis.moves.iter().enumerate() {
        assert_eq!(m.eval_before.as_ref().unwrap().score_cp, Some((i as i32) * 15));
        assert_eq!(
            m.eval_after.as_ref().unwrap().score_cp,
            Some(((i + 1) as i32) * 15)
        );
    }
}
