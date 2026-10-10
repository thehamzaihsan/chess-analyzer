use crate::analysis::GameAnalysis;
use crate::db::StoredGame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillScores {
    pub opening: u32,
    pub tactics: u32,
    pub endgame: u32,
    pub capitalization: u32,
    pub resourcefulness: u32,
    pub time_management: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningStat {
    pub eco: String,
    pub name: String,
    pub count: usize,
    pub wins: usize,
    pub losses: usize,
    pub draws: usize,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingPoint {
    pub date: String,
    pub elo: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverallInsights {
    pub total_games: usize,
    pub wins: usize,
    pub losses: usize,
    pub draws: usize,
    pub win_rate: f64,

    pub white_games: usize,
    pub white_wins: usize,
    pub white_win_rate: f64,

    pub black_games: usize,
    pub black_wins: usize,
    pub black_win_rate: f64,

    pub avg_accuracy: f64,
    pub skill_scores: SkillScores,
    pub rating_history: Vec<RatingPoint>,
    pub top_openings: Vec<OpeningStat>,

    pub castling_rate: f64,
    pub early_queen_rate: f64,
    pub early_queen_win_rate: f64,

    pub avg_blunders_per_game: f64,
    pub takeaways: Vec<String>,
}

pub fn compute_insights(
    player_username: &str,
    games: &[StoredGame],
    analyses: &[Option<GameAnalysis>],
) -> OverallInsights {
    if games.is_empty() {
        return OverallInsights::default();
    }

    let user_lower = player_username.trim().to_lowercase();
    let mut wins = 0;
    let mut losses = 0;
    let mut draws = 0;

    let mut white_games = 0;
    let mut white_wins = 0;
    let mut black_games = 0;
    let mut black_wins = 0;

    let mut total_accuracy = 0.0;
    let mut accuracy_count = 0;

    let mut total_blunders = 0;
    let mut early_queen_games = 0;
    let mut early_queen_wins = 0;
    let mut castled_games = 0;

    let mut opening_map: HashMap<(String, String), (usize, usize, usize, usize)> = HashMap::new();
    let mut rating_history = Vec::new();

    for (idx, game) in games.iter().enumerate() {
        let is_user_white = game.white.to_lowercase() == user_lower;
        let is_user_black = game.black.to_lowercase() == user_lower;

        // Skip if user not in game and username was given, unless username is empty
        if !user_lower.is_empty() && !is_user_white && !is_user_black {
            continue;
        }

        let is_white = if user_lower.is_empty() { true } else { is_user_white };

        // Result calculation:
        let (user_won, is_draw) = match game.result.as_str() {
            "1-0" => (is_white, false),
            "0-1" => (!is_white, false),
            "1/2-1/2" => (false, true),
            _ => (false, false),
        };

        if is_draw {
            draws += 1;
        } else if user_won {
            wins += 1;
            if is_white {
                white_wins += 1;
            } else {
                black_wins += 1;
            }
        } else {
            losses += 1;
        }

        if is_white {
            white_games += 1;
            if let Some(elo) = game.white_elo {
                rating_history.push(RatingPoint {
                    date: game.date.clone(),
                    elo,
                });
            }
        } else {
            black_games += 1;
            if let Some(elo) = game.black_elo {
                rating_history.push(RatingPoint {
                    date: game.date.clone(),
                    elo,
                });
            }
        }

        // Openings:
        let entry = opening_map
            .entry((game.eco.clone(), game.opening.clone()))
            .or_insert((0, 0, 0, 0));
        entry.0 += 1; // count
        if is_draw {
            entry.3 += 1;
        } else if user_won {
            entry.1 += 1;
        } else {
            entry.2 += 1;
        }

        // Check analysis if available
        if let Some(Some(analysis)) = analyses.get(idx) {
            let acc = if is_white {
                analysis.white_accuracy
            } else {
                analysis.black_accuracy
            };
            total_accuracy += acc;
            accuracy_count += 1;

            let blunders = if is_white {
                analysis.white_blunders
            } else {
                analysis.black_blunders
            };
            total_blunders += blunders;

            let early_q = if is_white {
                analysis.early_queen_white
            } else {
                analysis.early_queen_black
            };
            if early_q {
                early_queen_games += 1;
                if user_won {
                    early_queen_wins += 1;
                }
            }

            let castled = if is_white {
                analysis.castled_white.is_some()
            } else {
                analysis.castled_black.is_some()
            };
            if castled {
                castled_games += 1;
            }
        }
    }

    let total = wins + losses + draws;
    if total == 0 {
        return OverallInsights::default();
    }

    let win_rate = (wins as f64 / total as f64) * 100.0;
    let white_win_rate = if white_games > 0 {
        (white_wins as f64 / white_games as f64) * 100.0
    } else {
        0.0
    };
    let black_win_rate = if black_games > 0 {
        (black_wins as f64 / black_games as f64) * 100.0
    } else {
        0.0
    };

    let avg_accuracy = if accuracy_count > 0 {
        total_accuracy / accuracy_count as f64
    } else {
        72.0 // default estimate
    };

    let castling_rate = if total > 0 {
        (castled_games as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let early_queen_rate = if total > 0 {
        (early_queen_games as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let early_queen_win_rate = if early_queen_games > 0 {
        (early_queen_wins as f64 / early_queen_games as f64) * 100.0
    } else {
        0.0
    };

    let avg_blunders_per_game = if total > 0 {
        total_blunders as f64 / total as f64
    } else {
        0.0
    };

    // Calculate 6 skill scores (0-100):
    let opening_score = ((avg_accuracy * 0.9 + win_rate * 0.1) as u32).min(100);
    let tactics_score = ((100.0 - avg_blunders_per_game * 18.0).max(10.0).min(98.0)) as u32;
    let endgame_score = ((avg_accuracy * 0.85 + (100.0 - losses as f64 / total as f64 * 50.0) * 0.15) as u32).min(100);
    let capitalization = ((win_rate * 0.7 + avg_accuracy * 0.3) as u32).min(100);
    let resourcefulness = (((draws as f64 / total as f64) * 200.0 + avg_accuracy * 0.6) as u32).min(100);
    let time_management = 75; // baseline heuristic

    let skill_scores = SkillScores {
        opening: opening_score,
        tactics: tactics_score,
        endgame: endgame_score,
        capitalization,
        resourcefulness,
        time_management,
    };

    // Top openings:
    let mut top_openings: Vec<OpeningStat> = opening_map
        .into_iter()
        .map(|((eco, name), (cnt, w, l, d))| {
            let wr = if cnt > 0 { (w as f64 / cnt as f64) * 100.0 } else { 0.0 };
            OpeningStat {
                eco,
                name: if name.is_empty() { "Custom Line".to_string() } else { name },
                count: cnt,
                wins: w,
                losses: l,
                draws: d,
                win_rate: wr,
            }
        })
        .collect();
    top_openings.sort_by(|a, b| b.count.cmp(&a.count));

    // Natural language takeaways:
    let mut takeaways = Vec::new();
    if white_win_rate > black_win_rate + 10.0 {
        takeaways.push(format!(
            "Noticeable color asymmetry: you win {:.1}% as White vs {:.1}% as Black.",
            white_win_rate, black_win_rate
        ));
    }
    if let Some(best_op) = top_openings.iter().filter(|o| o.count >= 2).max_by(|a, b| a.win_rate.partial_cmp(&b.win_rate).unwrap()) {
        takeaways.push(format!(
            "Your strongest opening is {} ({}) with a {:.0}% win rate over {} games.",
            best_op.name, best_op.eco, best_op.win_rate, best_op.count
        ));
    }
    if early_queen_rate > 20.0 {
        takeaways.push(format!(
            "Frequent early queen movement ({:.0}% of games). When doing so, your win rate is {:.0}%.",
            early_queen_rate, early_queen_win_rate
        ));
    }
    if avg_blunders_per_game > 1.5 {
        takeaways.push(format!(
            "Tactical blunders average {:.1} per game. Focusing on hanging pieces in the middlegame will yield rapid rating gains.",
            avg_blunders_per_game
        ));
    } else {
        takeaways.push(format!(
            "Solid tactical play: only {:.1} blunders per game on average.",
            avg_blunders_per_game
        ));
    }

    OverallInsights {
        total_games: total,
        wins,
        losses,
        draws,
        win_rate,
        white_games,
        white_wins,
        white_win_rate,
        black_games,
        black_wins,
        black_win_rate,
        avg_accuracy,
        skill_scores,
        rating_history,
        top_openings,
        castling_rate,
        early_queen_rate,
        early_queen_win_rate,
        avg_blunders_per_game,
        takeaways,
    }
}
