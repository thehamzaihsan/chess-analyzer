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
    pub result: String, // "win", "draw", "loss"
    pub opponent: String,
    pub opponent_elo: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingBand {
    pub label: String, // e.g. "1200-1299"
    pub lower_bound: i32,
    pub games_count: usize,
    pub wins: usize,
    pub draws: usize,
    pub losses: usize,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenYouPlayStat {
    pub label: String, // e.g. "Monday", "Afternoon"
    pub games_count: usize,
    pub wins: usize,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverallInsights {
    pub player_id: String,
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
    pub peak_rating: i32,
    pub lowest_rating: i32,
    pub current_rating: i32,
    pub rating_change: i32,

    pub rating_bands: Vec<RatingBand>,
    pub weekday_stats: Vec<WhenYouPlayStat>,
    pub time_of_day_stats: Vec<WhenYouPlayStat>,

    pub top_openings: Vec<OpeningStat>,

    pub castling_rate: f64,
    pub early_queen_rate: f64,
    pub early_queen_win_rate: f64,

    pub avg_blunders_per_game: f64,
    pub takeaways: Vec<String>,
}

/// Detects the most frequently appearing username across games (case-preserved)
pub fn detect_default_player(games: &[StoredGame]) -> Option<String> {
    let mut counts: HashMap<String, (String, usize)> = HashMap::new();
    for g in games {
        for name in [&g.white, &g.black] {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                continue;
            }
            let key = trimmed.to_lowercase();
            counts
                .entry(key)
                .and_modify(|(_, c)| *c += 1)
                .or_insert_with(|| (trimmed.to_string(), 1));
        }
    }
    counts.into_values().max_by_key(|(_, c)| *c).map(|(name, _)| name)
}

fn parse_hour_from_pgn(pgn: &str) -> Option<u32> {
    for line in pgn.lines() {
        if line.starts_with("[EndTime \"") || line.starts_with("[UTCTime \"") || line.starts_with("[Time \"") {
            let after_quote = line.split('"').nth(1)?;
            let hour_str = after_quote.split(':').next()?;
            if let Ok(h) = hour_str.parse::<u32>() {
                if h < 24 {
                    return Some(h);
                }
            }
        }
    }
    None
}

fn parse_date_weekday(date_str: &str) -> Option<usize> {
    let s = date_str.replace('.', "-");
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() >= 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            if let Some(nd) = chrono::NaiveDate::from_ymd_opt(y, m, d) {
                use chrono::Datelike;
                return Some(nd.weekday().num_days_from_monday() as usize);
            }
        }
    }
    None
}

pub fn compute_insights(
    player_username: &str,
    games: &[StoredGame],
    analyses: &[Option<GameAnalysis>],
) -> OverallInsights {
    if games.is_empty() {
        return OverallInsights::default();
    }

    // Determine target player username
    let target_player = if !player_username.trim().is_empty() {
        let p_low = player_username.trim().to_lowercase();
        let found = games.iter().any(|g| {
            g.white.to_lowercase() == p_low || g.black.to_lowercase() == p_low
        });
        if found {
            player_username.trim().to_string()
        } else {
            detect_default_player(games).unwrap_or_else(|| player_username.trim().to_string())
        }
    } else {
        detect_default_player(games).unwrap_or_else(|| "Player".to_string())
    };

    let user_lower = target_player.to_lowercase();
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
    let mut rating_history: Vec<RatingPoint> = Vec::new();
    let mut band_map: HashMap<i32, (usize, usize, usize, usize)> = HashMap::new(); // lower_bound -> (total, w, d, l)

    let mut weekday_counts = vec![(0usize, 0usize); 7]; // (total, wins) for Mon..Sun
    let mut tod_counts = vec![(0usize, 0usize); 4]; // (total, wins) for Night, Morning, Afternoon, Evening

    for (idx, game) in games.iter().enumerate() {
        let is_user_white = game.white.to_lowercase() == user_lower;
        let is_user_black = game.black.to_lowercase() == user_lower;

        if !is_user_white && !is_user_black && !user_lower.is_empty() {
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

        let (my_elo, opp_elo, opp_name) = if is_white {
            white_games += 1;
            (game.white_elo, game.black_elo, game.black.clone())
        } else {
            black_games += 1;
            (game.black_elo, game.white_elo, game.white.clone())
        };

        let res_str = if is_draw {
            "draw"
        } else if user_won {
            "win"
        } else {
            "loss"
        };

        if let Some(elo) = my_elo {
            rating_history.push(RatingPoint {
                date: game.date.clone(),
                elo,
                result: res_str.to_string(),
                opponent: opp_name,
                opponent_elo: opp_elo,
            });
        }

        // Opponent Rating Bands
        if let Some(oe) = opp_elo {
            let lower_bound = (oe / 100) * 100;
            let entry = band_map.entry(lower_bound).or_insert((0, 0, 0, 0));
            entry.0 += 1;
            if is_draw {
                entry.2 += 1;
            } else if user_won {
                entry.1 += 1;
            } else {
                entry.3 += 1;
            }
        }

        // When You Play: Weekday
        if let Some(wd) = parse_date_weekday(&game.date) {
            if wd < 7 {
                weekday_counts[wd].0 += 1;
                if user_won {
                    weekday_counts[wd].1 += 1;
                }
            }
        }

        // When You Play: Time of Day
        if let Some(hour) = parse_hour_from_pgn(&game.pgn) {
            let tod_idx = match hour {
                0..=5 => 0,   // Night
                6..=11 => 1,  // Morning
                12..=17 => 2, // Afternoon
                _ => 3,       // Evening
            };
            tod_counts[tod_idx].0 += 1;
            if user_won {
                tod_counts[tod_idx].1 += 1;
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

    // Rating Progression Statistics
    let peak_rating = rating_history.iter().map(|p| p.elo).max().unwrap_or(0);
    let lowest_rating = rating_history.iter().map(|p| p.elo).min().unwrap_or(0);
    let current_rating = rating_history.last().map(|p| p.elo).unwrap_or(0);
    let first_rating = rating_history.first().map(|p| p.elo).unwrap_or(current_rating);
    let rating_change = current_rating - first_rating;

    // Rating Bands
    let mut rating_bands: Vec<RatingBand> = band_map
        .into_iter()
        .map(|(lower, (cnt, w, d, l))| {
            let wr = if cnt > 0 { (w as f64 / cnt as f64) * 100.0 } else { 0.0 };
            RatingBand {
                label: format!("{}-{}", lower, lower + 99),
                lower_bound: lower,
                games_count: cnt,
                wins: w,
                draws: d,
                losses: l,
                win_rate: wr,
            }
        })
        .collect();
    rating_bands.sort_by_key(|b| b.lower_bound);

    // Weekday stats
    let weekday_names = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    let weekday_stats: Vec<WhenYouPlayStat> = weekday_names
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let (cnt, w) = weekday_counts[i];
            let wr = if cnt > 0 { (w as f64 / cnt as f64) * 100.0 } else { 0.0 };
            WhenYouPlayStat {
                label: name.to_string(),
                games_count: cnt,
                wins: w,
                win_rate: wr,
            }
        })
        .collect();

    // Time of day stats
    let tod_names = ["Night (0-6)", "Morning (6-12)", "Afternoon (12-18)", "Evening (18-24)"];
    let time_of_day_stats: Vec<WhenYouPlayStat> = tod_names
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let (cnt, w) = tod_counts[i];
            let wr = if cnt > 0 { (w as f64 / cnt as f64) * 100.0 } else { 0.0 };
            WhenYouPlayStat {
                label: name.to_string(),
                games_count: cnt,
                wins: w,
                win_rate: wr,
            }
        })
        .collect();

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
    if peak_rating > 0 {
        takeaways.push(format!(
            "Your peak rating is {} Elo across {} tracked rated games (current: {} Elo).",
            peak_rating, rating_history.len(), current_rating
        ));
    }
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
        player_id: target_player,
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
        peak_rating,
        lowest_rating,
        current_rating,
        rating_change,
        rating_bands,
        weekday_stats,
        time_of_day_stats,
        top_openings,
        castling_rate,
        early_queen_rate,
        early_queen_win_rate,
        avg_blunders_per_game,
        takeaways,
    }
}
