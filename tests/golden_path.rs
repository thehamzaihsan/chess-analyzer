use chess_analyzer::analysis::analyze_game;
use chess_analyzer::coach::CoachClient;
use chess_analyzer::db::{Database, Puzzle};
use chess_analyzer::engine::uci::{discover_engine_path, UciEngine, UciEval};
use chess_analyzer::insights::compute_insights;
use chess_analyzer::pgn::parse_pgn_games;
use shakmaty::Position;
use std::fs;

#[tokio::test]
async fn test_golden_path_e2e() {
    println!("\n=== Starting Chess Analyzer Golden Path Verification ===");

    // Step 1: In-memory database & PGN parsing
    println!("Step 1: Initializing in-memory database and parsing sample PGN...");
    let db = Database::memory().expect("Failed to create in-memory database");
    
    let sample_pgn_path = "chess_com_games_2026-09-30.pgn";
    let pgn_content = fs::read_to_string(sample_pgn_path)
        .expect("Failed to read sample PGN archive");
    
    let parsed_games = parse_pgn_games(&pgn_content);
    assert!(!parsed_games.is_empty(), "Parsed games should not be empty");
    println!("Parsed {} games from archive.", parsed_games.len());

    let g = &parsed_games[0];
    let game_id = db.insert_game(
        &g.white,
        &g.black,
        &g.result,
        g.white_elo,
        g.black_elo,
        &g.date,
        &g.time_control,
        &g.eco,
        &g.opening,
        &g.moves.iter().map(|m| m.san.clone()).collect::<Vec<_>>().join(" "),
    ).expect("Failed to insert game");
    println!("Inserted game #{} ({} vs {}, ECO: {})", game_id, g.white, g.black, g.eco);

    // Step 2: Spawn Stockfish via UCI and evaluate moves
    println!("\nStep 2: Spawning Stockfish via UCI...");
    let sf_path = discover_engine_path("stockfish").expect("Stockfish binary must be discoverable");
    let mut sf_engine = UciEngine::spawn(&sf_path, &[])
        .await
        .expect("Stockfish failed to spawn");

    // Evaluate the first 6 plies
    let moves_to_eval = &g.moves[..6.min(g.moves.len())];
    let mut evals: Vec<Option<UciEval>> = Vec::new();
    let mut pos = shakmaty::Chess::default();

    for (i, m) in moves_to_eval.iter().enumerate() {
        let fen = shakmaty::fen::Fen::from_position(&pos, shakmaty::EnPassantMode::Legal).to_string();
        let eval = sf_engine.evaluate_fen(&fen, 8, Some(500))
            .await
            .expect("Failed to evaluate position");
        
        println!("  Ply {}: {} -> Eval: {:?} cp, Best: {:?}", i + 1, m.san, eval.score_cp, eval.best_move);
        evals.push(Some(eval));

        // Advance position
        if let Ok(legal_move) = m.san.parse::<shakmaty::san::San>() {
            if let Ok(m_obj) = legal_move.to_move(&pos) {
                pos = pos.play(m_obj).expect("Legal move play");
            }
        }
    }
    let _ = sf_engine.quit().await;

    // Step 3: CAPS Accuracy and Move Classification
    println!("\nStep 3: Calculating CAPS Accuracy and Classifications...");
    let analysis = analyze_game(&g.moves[..6.min(g.moves.len())], &evals);
    println!("White accuracy: {:.1}%, Black accuracy: {:.1}%", analysis.white_accuracy, analysis.black_accuracy);
    assert!(analysis.white_accuracy >= 0.0 && analysis.white_accuracy <= 100.0);
    assert!(analysis.black_accuracy >= 0.0 && analysis.black_accuracy <= 100.0);

    // Step 4: Insights Engine
    println!("\nStep 4: Computing Player Insights...");
    let stored_games = db.list_games().expect("Failed to list games");
    let analysis_vec = vec![Some(analysis.clone())];
    let insights = compute_insights("ibn-ehsan", &stored_games, &analysis_vec);
    
    println!("Tactics Score: {}/100", insights.skill_scores.tactics);
    println!("Opening Score: {}/100", insights.skill_scores.opening);
    println!("Endgame Score: {}/100", insights.skill_scores.endgame);
    println!("Capitalization Score: {}/100", insights.skill_scores.capitalization);

    // Step 5: Puzzle Mining & Verification
    println!("\nStep 5: Testing Puzzle Mining & Interactive Verification...");
    let puzzle = Puzzle {
        id: "mined-blunder-001".to_string(),
        fen: "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4".to_string(),
        moves: "h5f7".to_string(), // Scholar's mate
        rating: 1500,
        rating_deviation: 80,
        popularity: 100,
        nb_plays: 50,
        themes: "mate,mateIn1".to_string(),
        opening_tags: "Italian Game".to_string(),
        source: "blunder".to_string(),
    };
    db.insert_puzzle(&puzzle).expect("Failed to insert puzzle");
    let loaded_puzzle = db.get_random_puzzle(Some("blunder")).expect("Failed to load puzzle");
    assert!(loaded_puzzle.is_some());
    assert_eq!(loaded_puzzle.unwrap().moves, "h5f7");
    println!("Mined and verified blunder puzzle: moves={}", puzzle.moves);

    // Step 6: Bot Play via Stockfish UCI LimitStrength
    println!("\nStep 6: Testing Bot Play Engine with UCI_LimitStrength...");
    let bot_options = [
        ("UCI_LimitStrength", "true"),
        ("UCI_Elo", "1600"),
    ];
    let mut bot_engine = UciEngine::spawn(sf_path, &bot_options)
        .await
        .expect("Failed to spawn bot engine");
    let bot_eval = bot_engine
        .evaluate_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1", 6, Some(400))
        .await
        .expect("Bot failed to compute move");
    let _ = bot_engine.quit().await;

    assert!(bot_eval.best_move.is_some(), "Bot should generate a valid move");
    println!("Club Tactician (1600 Elo) replied to 1. e4 with best move: {:?}", bot_eval.best_move.unwrap());

    // Step 7: Packaged Local Gemma Coach Client
    println!("\nStep 7: Testing Packaged Local Gemma Coach...");
    let coach = CoachClient::new(None, None);
    assert!(coach.binary_path.is_some(), "Coach runner binary (llama-cli) must be discovered");
    assert!(coach.model_path.is_some(), "Packaged Gemma model (.gguf) must be discovered");
    println!("Packaged runner: {:?}", coach.binary_path.as_ref().unwrap());
    println!("Packaged Gemma model: {:?}", coach.model_path.as_ref().unwrap());

    let explanation_res = coach.explain_move(
        "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
        "Bc4",
        "Best",
        0.5,
        &["Bc4".to_string(), "Bc5".to_string()],
    ).await;
    match explanation_res {
        Ok(exp) => {
            println!("Local Gemma coach explanation:\n---\n{}\n---", exp);
            assert!(!exp.is_empty(), "Coach response should not be empty");
        }
        Err(e) => panic!("Local Gemma coach failed to run: {}", e),
    }

    println!("\n=== Golden Path Verification Complete: ALL TESTS PASSED! ===");
}
