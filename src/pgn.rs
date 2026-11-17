use crate::openings::identify_opening;
use crate::ui::game_layout::GameOutcome;
use pgn_reader::{RawComment, RawTag, Reader, SanPlus, Visitor};
use shakmaty::fen::Fen;
use shakmaty::{CastlingMode, Chess, Color, EnPassantMode, Position};
use std::collections::HashMap;
use std::io::Cursor;
use std::ops::ControlFlow;

#[derive(Debug, Clone)]
pub struct ParsedMove {
    pub san: String,
    pub uci: String,
    pub fen_before: String,
    pub fen_after: String,
    pub clock_seconds: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ParsedGame {
    pub headers: HashMap<String, String>,
    pub moves: Vec<ParsedMove>,
    pub white: String,
    pub black: String,
    pub white_elo: Option<i32>,
    pub black_elo: Option<i32>,
    pub result: String,
    pub date: String,
    pub time_control: String,
    pub eco: String,
    pub opening: String,
}

impl ParsedGame {
    /// Serializes this parsed game (headers + numbered SAN moves + result) back to a PGN string
    /// so metadata such as `Termination`, `EndTime`, and `ECOUrl` is preserved when stored.
    pub fn to_pgn_string(&self) -> String {
        let mut out = String::new();
        let priority_keys = [
            "Event",
            "Site",
            "Date",
            "Round",
            "White",
            "Black",
            "Result",
            "WhiteElo",
            "BlackElo",
            "TimeControl",
            "EndTime",
            "ECO",
            "ECOUrl",
            "Opening",
            "Termination",
        ];

        let mut emitted = std::collections::HashSet::new();
        for key in priority_keys {
            if let Some(val) = self.headers.get(key) {
                let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
                out.push_str(&format!("[{} \"{}\"]\n", key, escaped));
                emitted.insert(key);
            }
        }

        let mut extra_keys: Vec<&String> = self
            .headers
            .keys()
            .filter(|k| !emitted.contains(k.as_str()))
            .collect();
        extra_keys.sort();
        for key in extra_keys {
            if let Some(val) = self.headers.get(key) {
                let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
                out.push_str(&format!("[{} \"{}\"]\n", key, escaped));
            }
        }

        if !out.is_empty() {
            out.push('\n');
        }

        for (i, m) in self.moves.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            if i % 2 == 0 {
                out.push_str(&format!("{}. {}", (i / 2) + 1, m.san));
            } else {
                out.push_str(&m.san);
            }
        }

        if !self.result.is_empty() {
            if !self.moves.is_empty() {
                out.push(' ');
            }
            out.push_str(&self.result);
        }

        out
    }
}

/// Derives a human-readable opening name from a Chess.com `ECOUrl` header, e.g.
/// `"https://www.chess.com/openings/Sicilian-Defense-Najdorf-Variation"` ->
/// `"Sicilian Defense Najdorf Variation"`.
pub fn opening_from_eco_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_query = trimmed.split(['?', '#']).next().unwrap_or(trimmed);
    let trimmed_slash = without_query.trim_end_matches('/');
    let segment = if let Some(idx) = trimmed_slash.rfind("/openings/") {
        &trimmed_slash[idx + "/openings/".len()..]
    } else {
        trimmed_slash.rsplit('/').next().unwrap_or("")
    };
    if segment.is_empty() {
        return None;
    }
    let cleaned = segment.replace('-', " ").trim().to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

/// Parses PGN tag-pair headers (`[Key "Value"]`) from the start of a PGN string.
pub fn parse_pgn_headers(pgn_content: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for raw_line in pgn_content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('[') {
            if !headers.is_empty() {
                break;
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let rest = rest.trim_start();
            if let Some(first_quote) = rest.find('"') {
                let key = rest[..first_quote].trim();
                let after_quote = &rest[first_quote + 1..];
                let mut val = String::new();
                let mut escaped = false;
                let mut closed = false;
                for ch in after_quote.chars() {
                    if escaped {
                        val.push(ch);
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        closed = true;
                        break;
                    } else {
                        val.push(ch);
                    }
                }
                if closed && !key.is_empty() {
                    headers.insert(key.to_string(), val);
                }
            }
        }
    }
    headers
}

fn non_empty_tag<'a>(headers: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    headers
        .get(key)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "?" && *s != "-")
}

/// Resolves `(eco, opening)` for an existing stored game (or raw PGN) using:
/// 1) Existing `opening` if non-empty
/// 2) `Opening` PGN header
/// 3) `ECOUrl` PGN header
/// 4) `crate::openings::identify_opening` on the parsed moves.
pub fn resolve_opening_display(opening: &str, eco: &str, pgn: &str) -> (String, String) {
    let clean_opening = opening.trim();
    let clean_eco = eco.trim();
    if !clean_opening.is_empty() && clean_opening != "?" && clean_opening != "-" {
        if !clean_eco.is_empty() && clean_eco != "?" {
            return (clean_eco.to_string(), clean_opening.to_string());
        }
        let headers = parse_pgn_headers(pgn);
        let hdr_eco = non_empty_tag(&headers, "ECO").unwrap_or("");
        return (hdr_eco.to_string(), clean_opening.to_string());
    }

    let headers = parse_pgn_headers(pgn);
    let hdr_eco = non_empty_tag(&headers, "ECO").map(str::to_string);

    if let Some(op) = non_empty_tag(&headers, "Opening") {
        return (hdr_eco.unwrap_or_else(|| clean_eco.to_string()), op.to_string());
    }

    if let Some(op) = non_empty_tag(&headers, "ECOUrl").and_then(opening_from_eco_url) {
        return (hdr_eco.unwrap_or_else(|| clean_eco.to_string()), op);
    }

    let games = parse_pgn_games(pgn);
    if let Some(first) = games.first() {
        let resolved_eco = if !first.eco.is_empty() {
            first.eco.clone()
        } else {
            hdr_eco.unwrap_or_else(|| clean_eco.to_string())
        };
        return (resolved_eco, first.opening.clone());
    }

    (hdr_eco.unwrap_or_else(|| clean_eco.to_string()), String::new())
}

fn is_decisive_or_draw_result(res: &str) -> bool {
    matches!(res, "1-0" | "0-1" | "1/2-1/2")
}

/// Detects the game outcome (`Result` + human-readable `Termination` reason) from
/// `StoredGame.pgn` headers, `stored_result`, and/or the final board position.
pub fn detect_game_outcome(
    pgn: &str,
    stored_result: &str,
    white: &str,
    black: &str,
    final_pos: Option<&Chess>,
) -> Option<GameOutcome> {
    let headers = parse_pgn_headers(pgn);

    let header_result = non_empty_tag(&headers, "Result").filter(|r| is_decisive_or_draw_result(r));
    let stored = stored_result.trim();
    let stored_valid = is_decisive_or_draw_result(stored).then_some(stored);

    let pos_result = final_pos.and_then(|pos| {
        if pos.is_checkmate() {
            Some(if pos.turn() == Color::Black {
                "1-0"
            } else {
                "0-1"
            })
        } else if pos.is_stalemate() || pos.is_insufficient_material() || pos.is_game_over() {
            Some("1/2-1/2")
        } else {
            None
        }
    });

    let result = header_result.or(stored_valid).or(pos_result)?;

    let white_label = {
        let w = white.trim();
        if w.is_empty() || w == "Unknown" || w == "?" {
            "White"
        } else {
            w
        }
    };
    let black_label = {
        let b = black.trim();
        if b.is_empty() || b == "Unknown" || b == "?" {
            "Black"
        } else {
            b
        }
    };
    let winner_label = match result {
        "1-0" => white_label,
        "0-1" => black_label,
        _ => "Draw",
    };

    let reason = if let Some(term) = non_empty_tag(&headers, "Termination") {
        let lower = term.to_ascii_lowercase();
        if lower == "normal" || lower == "unterminated" {
            derive_reason_from_position(result, winner_label, final_pos)
        } else if lower == "time forfeit" {
            if result == "1/2-1/2" {
                "Game drawn on time".to_string()
            } else {
                format!("{} won on time", winner_label)
            }
        } else if lower == "abandoned" {
            if result == "1/2-1/2" {
                "Game abandoned".to_string()
            } else {
                format!("{} won by abandonment", winner_label)
            }
        } else if lower == "rules infraction" {
            if result == "1/2-1/2" {
                "Game drawn (rules infraction)".to_string()
            } else {
                format!("{} won by rules infraction", winner_label)
            }
        } else {
            term.to_string()
        }
    } else {
        derive_reason_from_position(result, winner_label, final_pos)
    };

    Some(GameOutcome::new(result, reason))
}

fn derive_reason_from_position(
    result: &str,
    winner_label: &str,
    final_pos: Option<&Chess>,
) -> String {
    if let Some(pos) = final_pos {
        if pos.is_checkmate() {
            let mate_winner = if pos.turn() == Color::Black {
                if result == "1-0" {
                    winner_label
                } else {
                    "White"
                }
            } else if result == "0-1" {
                winner_label
            } else {
                "Black"
            };
            return format!("{} won by checkmate", mate_winner);
        }
        if pos.is_stalemate() {
            return "Draw by stalemate".to_string();
        }
        if pos.is_insufficient_material() {
            return "Draw by insufficient material".to_string();
        }
        if pos.is_game_over() {
            return "Game drawn".to_string();
        }
    }

    match result {
        "1-0" | "0-1" => format!("{} won", winner_label),
        "1/2-1/2" => "Game drawn".to_string(),
        _ => "Game over".to_string(),
    }
}

struct GameCollector {
    headers: HashMap<String, String>,
    pos: Chess,
    moves: Vec<ParsedMove>,
    current_clock: Option<f64>,
}

impl GameCollector {
    fn new() -> Self {
        Self {
            headers: HashMap::new(),
            pos: Chess::default(),
            moves: Vec::new(),
            current_clock: None,
        }
    }
}

impl Visitor for GameCollector {
    type Tags = ();
    type Movetext = ();
    type Output = ParsedGame;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.headers.clear();
        self.pos = Chess::default();
        self.moves.clear();
        self.current_clock = None;
        ControlFlow::Continue(())
    }

    fn tag(
        &mut self,
        _tags: &mut Self::Tags,
        name: &[u8],
        value: RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
        let key = String::from_utf8_lossy(name).to_string();
        let val = String::from_utf8_lossy(&value.decode()).to_string();
        self.headers.insert(key, val);
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        san_plus: SanPlus,
    ) -> ControlFlow<Self::Output> {
        let san_str = san_plus.to_string();
        let fen_before = Fen::from_position(&self.pos, EnPassantMode::Legal).to_string();

        if let Ok(m) = san_plus.san.to_move(&self.pos) {
            let uci_str = m.to_uci(CastlingMode::Standard).to_string();

            if let Ok(next_pos) = self.pos.clone().play(m) {
                self.pos = next_pos;
                let fen_after =
                    Fen::from_position(&self.pos, EnPassantMode::Legal).to_string();
                self.moves.push(ParsedMove {
                    san: san_str,
                    uci: uci_str,
                    fen_before,
                    fen_after,
                    clock_seconds: self.current_clock.take(),
                });
            }
        }
        ControlFlow::Continue(())
    }

    fn comment(
        &mut self,
        _movetext: &mut Self::Movetext,
        comment: RawComment<'_>,
    ) -> ControlFlow<Self::Output> {
        let text = String::from_utf8_lossy(comment.as_bytes());
        if let Some(pos) = text.find("[%clk ") {
            let rest = &text[pos + 6..];
            if let Some(end) = rest.find(']') {
                let clk_str = &rest[..end].trim();
                let parts: Vec<&str> = clk_str.split(':').collect();
                let seconds = if parts.len() == 3 {
                    let h: f64 = parts[0].parse().unwrap_or(0.0);
                    let m: f64 = parts[1].parse().unwrap_or(0.0);
                    let s: f64 = parts[2].parse().unwrap_or(0.0);
                    Some(h * 3600.0 + m * 60.0 + s)
                } else if parts.len() == 2 {
                    let m: f64 = parts[0].parse().unwrap_or(0.0);
                    let s: f64 = parts[1].parse().unwrap_or(0.0);
                    Some(m * 60.0 + s)
                } else {
                    None
                };
                if let Some(s) = seconds {
                    if let Some(last_move) = self.moves.last_mut() {
                        last_move.clock_seconds = Some(s);
                    } else {
                        self.current_clock = Some(s);
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, _movetext: Self::Movetext) -> Self::Output {
        let white = self
            .headers
            .get("White")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let black = self
            .headers
            .get("Black")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let white_elo = self
            .headers
            .get("WhiteElo")
            .and_then(|s| s.parse::<i32>().ok());
        let black_elo = self
            .headers
            .get("BlackElo")
            .and_then(|s| s.parse::<i32>().ok());
        let result = self
            .headers
            .get("Result")
            .cloned()
            .unwrap_or_else(|| "*".to_string());
        let date = self
            .headers
            .get("Date")
            .cloned()
            .unwrap_or_else(|| "".to_string());
        let time_control = self
            .headers
            .get("TimeControl")
            .cloned()
            .unwrap_or_else(|| "".to_string());

        let sans: Vec<String> = self.moves.iter().map(|m| m.san.clone()).collect();
        let identified = (!sans.is_empty()).then(|| identify_opening(&sans));

        let eco = non_empty_tag(&self.headers, "ECO")
            .map(str::to_string)
            .or_else(|| identified.as_ref().map(|m| m.eco.clone()))
            .unwrap_or_default();

        let opening = non_empty_tag(&self.headers, "Opening")
            .map(str::to_string)
            .or_else(|| non_empty_tag(&self.headers, "ECOUrl").and_then(opening_from_eco_url))
            .or_else(|| identified.as_ref().map(|m| m.name.clone()))
            .unwrap_or_default();

        ParsedGame {
            headers: self.headers.clone(),
            moves: self.moves.clone(),
            white,
            black,
            white_elo,
            black_elo,
            result,
            date,
            time_control,
            eco,
            opening,
        }
    }
}

pub fn parse_pgn_games(pgn_content: &str) -> Vec<ParsedGame> {
    let mut cursor = Cursor::new(pgn_content.as_bytes());
    let mut reader = Reader::new(&mut cursor);
    let mut games = Vec::new();

    let mut visitor = GameCollector::new();
    while let Ok(Some(game)) = reader.read_game(&mut visitor) {
        games.push(game);
    }

    games
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pgn_headers_extracts_result_and_termination() {
        let pgn = r#"[Event "Live Chess"]
[White "GothamChess"]
[Black "Hikaru"]
[Result "1-0"]
[Termination "GothamChess won by resignation"]

1. e4 e5 2. Nf3 Nc6 1-0"#;

        let headers = parse_pgn_headers(pgn);
        assert_eq!(headers.get("Result").map(String::as_str), Some("1-0"));
        assert_eq!(
            headers.get("Termination").map(String::as_str),
            Some("GothamChess won by resignation")
        );

        let outcome = detect_game_outcome(pgn, "1-0", "GothamChess", "Hikaru", None).unwrap();
        assert_eq!(outcome.result, "1-0");
        assert_eq!(outcome.reason, "GothamChess won by resignation");
    }

    #[test]
    fn test_detect_outcome_from_final_position_checkmate_and_stalemate() {
        // Fool's mate: 1. f3 e5 2. g4 Qh4# -> Black wins by checkmate
        let mate_pgn = "1. f3 e5 2. g4 Qh4#";
        let games = parse_pgn_games(mate_pgn);
        let final_fen = &games[0].moves.last().unwrap().fen_after;
        let final_pos: Chess = final_fen
            .parse::<Fen>()
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap();

        let outcome = detect_game_outcome(mate_pgn, "*", "Alice", "Bob", Some(&final_pos)).unwrap();
        assert_eq!(outcome.result, "0-1");
        assert_eq!(outcome.reason, "Bob won by checkmate");

        // Stalemate position: Black king on a8, White queen on b6, White king on e1, Black to move
        let stalemate_fen: Fen = "k7/8/1Q6/8/8/8/8/4K3 b - - 0 1".parse().unwrap();
        let stalemate_pos: Chess = stalemate_fen.into_position(CastlingMode::Standard).unwrap();
        let stalemate_outcome =
            detect_game_outcome("", "1/2-1/2", "Alice", "Bob", Some(&stalemate_pos)).unwrap();
        assert_eq!(stalemate_outcome.result, "1/2-1/2");
        assert_eq!(stalemate_outcome.reason, "Draw by stalemate");

        // Ongoing game with "*" and non-terminal position -> None
        let ongoing = Chess::default();
        assert!(detect_game_outcome("1. e4", "*", "Alice", "Bob", Some(&ongoing)).is_none());
    }

    #[test]
    fn test_detect_outcome_translates_standard_pgn_terminations() {
        let pgn_time = "[Result \"0-1\"]\n[Termination \"Time forfeit\"]\n\n1. e4 e5 0-1";
        let out_time = detect_game_outcome(pgn_time, "0-1", "Alice", "Bob", None).unwrap();
        assert_eq!(out_time.result, "0-1");
        assert_eq!(out_time.reason, "Bob won on time");

        let pgn_abandoned = "[Result \"1-0\"]\n[Termination \"Abandoned\"]\n\n1. e4 1-0";
        let out_abandoned =
            detect_game_outcome(pgn_abandoned, "1-0", "Alice", "Bob", None).unwrap();
        assert_eq!(out_abandoned.result, "1-0");
        assert_eq!(out_abandoned.reason, "Alice won by abandonment");
    }

    #[test]
    fn test_opening_derivation_from_header_eco_url_and_moves() {
        // 1. Explicit Opening header
        let pgn_header = "[Opening \"King's Indian Defense\"]\n[ECO \"E60\"]\n\n1. d4 Nf6 *";
        let g1 = parse_pgn_games(pgn_header);
        assert_eq!(g1[0].opening, "King's Indian Defense");
        assert_eq!(g1[0].eco, "E60");

        // 2. ECOUrl fallback
        let pgn_url = "[ECO \"B90\"]\n[ECOUrl \"https://www.chess.com/openings/Sicilian-Defense-Najdorf-Variation\"]\n\n1. e4 c5 *";
        let g2 = parse_pgn_games(pgn_url);
        assert_eq!(g2[0].opening, "Sicilian Defense Najdorf Variation");
        assert_eq!(g2[0].eco, "B90");

        // 3. Moves fallback via identify_opening
        let pgn_moves = "1. e4 e5 2. Nf3 Nc6 3. Bc4 *";
        let g3 = parse_pgn_games(pgn_moves);
        assert_eq!(g3[0].opening, "Italian Game");
        assert_eq!(g3[0].eco, "C50");

        // 4. Display-time fallback for existing DB row with empty opening
        let (eco, op) = resolve_opening_display("", "", "e4 e5 Nf3 Nc6 Bb5");
        assert_eq!(eco, "C60");
        assert_eq!(op, "Ruy Lopez");
    }

    #[test]
    fn test_parsed_game_roundtrip_preserves_termination() {
        let pgn = "[White \"Alice\"]\n[Black \"Bob\"]\n[Result \"1-0\"]\n[Termination \"Alice won by resignation\"]\n\n1. e4 e5 1-0";
        let parsed = &parse_pgn_games(pgn)[0];
        let serialized = parsed.to_pgn_string();
        let headers = parse_pgn_headers(&serialized);
        assert_eq!(
            headers.get("Termination").map(String::as_str),
            Some("Alice won by resignation")
        );
        assert_eq!(headers.get("Result").map(String::as_str), Some("1-0"));
    }
}

