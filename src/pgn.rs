use pgn_reader::{RawComment, RawTag, Reader, SanPlus, Visitor};
use shakmaty::fen::Fen;
use shakmaty::{CastlingMode, Chess, EnPassantMode, Position};
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
        let eco = self
            .headers
            .get("ECO")
            .cloned()
            .unwrap_or_else(|| "".to_string());
        let opening = self
            .headers
            .get("Opening")
            .cloned()
            .unwrap_or_else(|| "".to_string());

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
