use chess_analyzer::analysis::{centipawns_to_win_prob, classify_move, determine_game_phase, MoveClassification};
use chess_analyzer::db::{CachedEval, Database, Puzzle};
use chess_analyzer::engine::discover_engine_path;
use chess_analyzer::openings::identify_opening;
use chess_analyzer::pgn::parse_pgn_games;
use shakmaty::Chess;

#[test]
fn test_opening_identification() {
    let sans = vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string(), "Nc6".to_string(), "Bc4".to_string()];
    let op = identify_opening(&sans);
    assert_eq!(op.eco, "C50");
    assert!(op.name.contains("Italian Game"));
}

#[test]
fn test_pgn_parsing_sample_file() {
    let pgn_text = std::fs::read_to_string("chess_com_games_2026-09-30.pgn")
        .expect("Sample PGN file must exist");
    let games = parse_pgn_games(&pgn_text);
    assert!(!games.is_empty(), "Parsed games should not be empty");
    let first = &games[0];
    assert!(!first.white.is_empty());
    assert!(!first.black.is_empty());
    assert!(!first.moves.is_empty());
}

#[test]
fn test_win_probability_and_classification() {
    let p_even = centipawns_to_win_prob(Some(0), None);
    assert!((p_even - 50.0).abs() < 0.1);

    let p_plus2 = centipawns_to_win_prob(Some(200), None);
    assert!(p_plus2 > 65.0);

    let p_mate = centipawns_to_win_prob(None, Some(3));
    assert_eq!(p_mate, 100.0);

    // Blunder test: White went from 80% to 20%
    let (class, loss) = classify_move(true, "Qxd4", "d1d4", Some("e1g1"), 80.0, 20.0, false);
    assert_eq!(class, MoveClassification::Blunder);
    assert!(loss > 50.0);
}

#[test]
fn test_game_phase_detection() {
    let pos = Chess::default();
    let phase = determine_game_phase(10, &pos);
    assert_eq!(phase, chess_analyzer::analysis::GamePhase::Opening);
}

#[test]
fn test_database_crud() {
    let db = Database::memory().expect("In-memory SQLite should open");

    // Insert game
    let _game_id = db.insert_game(
        "Magnus",
        "Hikaru",
        "1-0",
        Some(2850),
        Some(2820),
        "2026.01.01",
        "180+2",
        "B90",
        "Sicilian Defense: Najdorf",
        "1. e4 c5 2. Nf3 d6",
    ).expect("Insert game should succeed");

    let games = db.list_games().expect("List games should succeed");
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].white, "Magnus");

    // Cache eval
    let cached = CachedEval {
        fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string(),
        depth: 18,
        score_cp: Some(25),
        mate_in: None,
        best_move: Some("c7c5".to_string()),
        pv: vec!["c7c5".to_string(), "g1f3".to_string()],
    };
    db.set_cached_eval(&cached).expect("Set cached eval should succeed");

    let retrieved = db.get_cached_eval(&cached.fen).expect("Get cached eval");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().depth, 18);

    // Insert puzzle
    let puzzle = Puzzle {
        id: "puz_001".to_string(),
        fen: "8/8/8/8/8/5K2/4R3/5k2 w - - 0 1".to_string(),
        moves: "e2e3 f1g1".to_string(),
        rating: 1650,
        rating_deviation: 80,
        popularity: 95,
        nb_plays: 250,
        themes: "endgame,rookEndgame".to_string(),
        opening_tags: "".to_string(),
        source: "lichess".to_string(),
    };
    db.insert_puzzle(&puzzle).expect("Insert puzzle");
    let random_p = db.get_random_puzzle(None).expect("Get random puzzle");
    assert!(random_p.is_some());
    assert_eq!(random_p.unwrap().id, "puz_001");
}

#[test]
fn test_stockfish_discovery() {
    let sf_path = discover_engine_path("stockfish");
    assert!(sf_path.is_some(), "Stockfish binary must be discoverable in assets/stockfish/");
}

#[test]
fn test_chesscom_sound_classification() {
    use chess_analyzer::audio::{sound_for_san, SoundType};

    assert_eq!(sound_for_san("e4"), SoundType::Move);
    assert_eq!(sound_for_san("Nf3"), SoundType::Move);
    assert_eq!(sound_for_san("Nxd4"), SoundType::Capture);
    assert_eq!(sound_for_san("exd5"), SoundType::Capture);
    assert_eq!(sound_for_san("O-O"), SoundType::Castle);
    assert_eq!(sound_for_san("O-O-O"), SoundType::Castle);
    assert_eq!(sound_for_san("e8=Q"), SoundType::Promote);
    assert_eq!(sound_for_san("Qh5+"), SoundType::Check);
    assert_eq!(sound_for_san("Qxf7#"), SoundType::Check);
}
