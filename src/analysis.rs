use crate::engine::UciEval;
use crate::openings::identify_opening;
use crate::pgn::ParsedMove;
use serde::{Deserialize, Serialize};
use shakmaty::{Chess, Position, Role, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveClassification {
    Book,
    Brilliant,
    Great,
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl MoveClassification {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Book => "Book",
            Self::Brilliant => "Brilliant",
            Self::Great => "Great",
            Self::Best => "Best",
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Inaccuracy => "Inaccuracy",
            Self::Mistake => "Mistake",
            Self::Blunder => "Blunder",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Book => "📖",
            Self::Brilliant => "✦✦",
            Self::Great => "!",
            Self::Best => "✓",
            Self::Excellent => "·",
            Self::Good => "·",
            Self::Inaccuracy => "?!",
            Self::Mistake => "?",
            Self::Blunder => "??",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            Self::Book => "#a88b57",
            Self::Brilliant => "#1baca6",
            Self::Great => "#5c8bb0",
            Self::Best => "#96bc4b",
            Self::Excellent => "#96bc4b",
            Self::Good => "#888888",
            Self::Inaccuracy => "#f0c15c",
            Self::Mistake => "#e58f2a",
            Self::Blunder => "#ca3431",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Opening,
    Middlegame,
    Endgame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedMove {
    pub ply: usize,
    pub move_number: usize,
    pub is_white: bool,
    pub san: String,
    pub uci: String,
    pub fen_before: String,
    pub fen_after: String,
    pub eval_before: Option<UciEval>,
    pub eval_after: Option<UciEval>,
    pub win_prob_before: f64,
    pub win_prob_after: f64,
    pub win_prob_loss: f64,
    pub classification: MoveClassification,
    pub phase: GamePhase,
    pub clock_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalysis {
    pub moves: Vec<AnalyzedMove>,
    pub white_accuracy: f64,
    pub black_accuracy: f64,
    pub white_blunders: usize,
    pub black_blunders: usize,
    pub white_mistakes: usize,
    pub black_mistakes: usize,
    pub white_inaccuracies: usize,
    pub black_inaccuracies: usize,
    pub opening_eco: String,
    pub opening_name: String,
    pub early_queen_white: bool,
    pub early_queen_black: bool,
    pub castled_white: Option<usize>,
    pub castled_black: Option<usize>,
}

/// Convert centipawns (from White's perspective) to a winning probability for White (0.0 to 100.0).
pub fn centipawns_to_win_prob(score_cp: Option<i32>, mate_in: Option<i32>) -> f64 {
    if let Some(m) = mate_in {
        return if m > 0 { 100.0 } else { 0.0 };
    }

    let cp = score_cp.unwrap_or(0) as f64;
    // Logistic curve model for win probability
    let sigmoid = 2.0 / (1.0 + (-0.003682 * cp).exp()) - 1.0;
    (50.0 + 50.0 * sigmoid).clamp(0.0, 100.0)
}

/// Classify a move given the evaluation before and after the move.
pub fn classify_move(
    is_white: bool,
    _san: &str,
    uci: &str,
    best_move: Option<&str>,
    win_prob_before: f64,
    win_prob_after: f64,
    is_book: bool,
) -> (MoveClassification, f64) {
    if is_book {
        return (MoveClassification::Book, 0.0);
    }

    let p_before = if is_white { win_prob_before } else { 100.0 - win_prob_before };
    let p_after = if is_white { win_prob_after } else { 100.0 - win_prob_after };
    let loss = (p_before - p_after).max(0.0);

    let is_engine_best = match best_move {
        Some(bm) => bm == uci,
        None => false,
    };

    let class = if is_engine_best && loss < 0.5 {
        MoveClassification::Best
    } else if loss <= 1.0 {
        MoveClassification::Best
    } else if loss <= 3.0 {
        MoveClassification::Excellent
    } else if loss <= 7.0 {
        MoveClassification::Good
    } else if loss <= 15.0 {
        MoveClassification::Inaccuracy
    } else if loss <= 25.0 {
        MoveClassification::Mistake
    } else {
        MoveClassification::Blunder
    };

    (class, loss)
}

/// Calculate harmonic/average accuracy from win% losses.
pub fn calculate_accuracy(losses: &[f64]) -> f64 {
    if losses.is_empty() {
        return 100.0;
    }

    let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;
    let acc = 100.0 * (-0.035 * avg_loss).exp();
    acc.clamp(0.0, 100.0)
}

/// Determine game phase from material and move count.
pub fn determine_game_phase(ply: usize, pos: &Chess) -> GamePhase {
    if ply <= 20 {
        return GamePhase::Opening;
    }

    let mut non_pawn_material = 0;
    for sq in Square::ALL {
        if let Some(piece) = pos.board().piece_at(sq) {
            match piece.role {
                Role::Queen => non_pawn_material += 9,
                Role::Rook => non_pawn_material += 5,
                Role::Bishop | Role::Knight => non_pawn_material += 3,
                _ => {}
            }
        }
    }

    if non_pawn_material <= 13 {
        GamePhase::Endgame
    } else {
        GamePhase::Middlegame
    }
}

/// Run full post-game analysis on parsed moves with provided evaluations.
pub fn analyze_game(
    moves: &[ParsedMove],
    evals: &[Option<UciEval>],
) -> GameAnalysis {
    let mut analyzed_moves = Vec::new();
    let mut white_losses = Vec::new();
    let mut black_losses = Vec::new();

    let mut white_blunders = 0;
    let mut black_blunders = 0;
    let mut white_mistakes = 0;
    let mut black_mistakes = 0;
    let mut white_inaccuracies = 0;
    let mut black_inaccuracies = 0;

    let mut early_queen_white = false;
    let mut early_queen_black = false;
    let mut castled_white = None;
    let mut castled_black = None;

    let sans: Vec<String> = moves.iter().map(|m| m.san.clone()).collect();
    let opening_match = identify_opening(&sans);

    let mut current_pos = Chess::default();

    for (i, m) in moves.iter().enumerate() {
        let is_white = i % 2 == 0;
        let move_number = (i / 2) + 1;

        let eval_before = evals.get(i).cloned().flatten();
        let eval_after = evals.get(i + 1).cloned().flatten();

        let win_prob_before = eval_before.as_ref().map_or(50.0, |e| {
            centipawns_to_win_prob(e.score_cp, e.mate_in)
        });
        let win_prob_after = eval_after.as_ref().map_or(50.0, |e| {
            centipawns_to_win_prob(e.score_cp, e.mate_in)
        });

        let is_book = i < 8 && !opening_match.name.is_empty();
        let best_move = eval_before.as_ref().and_then(|e| e.best_move.as_deref());

        let (classification, loss) = classify_move(
            is_white,
            &m.san,
            &m.uci,
            best_move,
            win_prob_before,
            win_prob_after,
            is_book,
        );

        if is_white {
            white_losses.push(loss);
            match classification {
                MoveClassification::Blunder => white_blunders += 1,
                MoveClassification::Mistake => white_mistakes += 1,
                MoveClassification::Inaccuracy => white_inaccuracies += 1,
                _ => {}
            }
        } else {
            black_losses.push(loss);
            match classification {
                MoveClassification::Blunder => black_blunders += 1,
                MoveClassification::Mistake => black_mistakes += 1,
                MoveClassification::Inaccuracy => black_inaccuracies += 1,
                _ => {}
            }
        }

        // Heuristics:
        if m.san.starts_with('Q') && move_number <= 7 {
            if is_white {
                early_queen_white = true;
            } else {
                early_queen_black = true;
            }
        }

        if m.san == "O-O" || m.san == "O-O-O" {
            if is_white && castled_white.is_none() {
                castled_white = Some(move_number);
            } else if !is_white && castled_black.is_none() {
                castled_black = Some(move_number);
            }
        }

        let phase = determine_game_phase(i + 1, &current_pos);

        analyzed_moves.push(AnalyzedMove {
            ply: i + 1,
            move_number,
            is_white,
            san: m.san.clone(),
            uci: m.uci.clone(),
            fen_before: m.fen_before.clone(),
            fen_after: m.fen_after.clone(),
            eval_before,
            eval_after,
            win_prob_before,
            win_prob_after,
            win_prob_loss: loss,
            classification,
            phase,
            clock_seconds: m.clock_seconds,
        });

        if let Ok(fen) = m.fen_after.parse::<shakmaty::fen::Fen>() {
            if let Ok(pos) = fen.into_position(shakmaty::CastlingMode::Standard) {
                current_pos = pos;
            }
        }
    }

    let white_accuracy = calculate_accuracy(&white_losses);
    let black_accuracy = calculate_accuracy(&black_losses);

    GameAnalysis {
        moves: analyzed_moves,
        white_accuracy,
        black_accuracy,
        white_blunders,
        black_blunders,
        white_mistakes,
        black_mistakes,
        white_inaccuracies,
        black_inaccuracies,
        opening_eco: opening_match.eco,
        opening_name: opening_match.name,
        early_queen_white,
        early_queen_black,
        castled_white,
        castled_black,
    }
}
