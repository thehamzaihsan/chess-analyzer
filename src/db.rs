use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGame {
    pub id: i64,
    pub pgn: String,
    pub white: String,
    pub black: String,
    pub white_elo: Option<i32>,
    pub black_elo: Option<i32>,
    pub result: String,
    pub date: String,
    pub time_control: String,
    pub eco: String,
    pub opening: String,
    pub accuracy_white: Option<f64>,
    pub accuracy_black: Option<f64>,
    pub analysis_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEval {
    pub fen: String,
    pub depth: u32,
    pub score_cp: Option<i32>,
    pub mate_in: Option<i32>,
    pub best_move: Option<String>,
    pub pv: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub id: String,
    pub fen: String,
    pub moves: String,
    pub rating: i32,
    pub rating_deviation: i32,
    pub popularity: i32,
    pub nb_plays: i32,
    pub themes: String,
    pub opening_tags: String,
    pub source: String, // "blunder" or "lichess"
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    pub fn open_default() -> Result<Self> {
        Self::open("chess_analyzer.db")
    }

    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pgn TEXT NOT NULL,
                white TEXT NOT NULL,
                black TEXT NOT NULL,
                white_elo INTEGER,
                black_elo INTEGER,
                result TEXT NOT NULL,
                date TEXT,
                time_control TEXT,
                eco TEXT,
                opening TEXT,
                accuracy_white REAL,
                accuracy_black REAL,
                analysis_json TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_games_date ON games(date);
            CREATE INDEX IF NOT EXISTS idx_games_white ON games(white);
            CREATE INDEX IF NOT EXISTS idx_games_black ON games(black);

            CREATE TABLE IF NOT EXISTS eval_cache (
                fen TEXT PRIMARY KEY,
                depth INTEGER NOT NULL,
                score_cp INTEGER,
                mate_in INTEGER,
                best_move TEXT,
                pv_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS puzzles (
                id TEXT PRIMARY KEY,
                fen TEXT NOT NULL,
                moves TEXT NOT NULL,
                rating INTEGER NOT NULL,
                rating_deviation INTEGER NOT NULL DEFAULT 80,
                popularity INTEGER NOT NULL DEFAULT 90,
                nb_plays INTEGER NOT NULL DEFAULT 100,
                themes TEXT NOT NULL,
                opening_tags TEXT,
                source TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_puzzles_rating ON puzzles(rating);
            CREATE INDEX IF NOT EXISTS idx_puzzles_source ON puzzles(source);

            CREATE TABLE IF NOT EXISTS sessions (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn insert_game(
        &self,
        white: &str,
        black: &str,
        result: &str,
        white_elo: Option<i32>,
        black_elo: Option<i32>,
        date: &str,
        time_control: &str,
        eco: &str,
        opening: &str,
        pgn: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO games (pgn, white, black, white_elo, black_elo, result, date, time_control, eco, opening)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![pgn, white, black, white_elo, black_elo, result, date, time_control, eco, opening],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_games(&self) -> Result<Vec<StoredGame>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, pgn, white, black, white_elo, black_elo, result, date, time_control, eco, opening,
                   accuracy_white, accuracy_black, analysis_json
            FROM games
            ORDER BY id DESC
            "#,
        )?;

        let games = stmt
            .query_map([], |row| {
                Ok(StoredGame {
                    id: row.get(0)?,
                    pgn: row.get(1)?,
                    white: row.get(2)?,
                    black: row.get(3)?,
                    white_elo: row.get(4)?,
                    black_elo: row.get(5)?,
                    result: row.get(6)?,
                    date: row.get(7).unwrap_or_default(),
                    time_control: row.get(8).unwrap_or_default(),
                    eco: row.get(9).unwrap_or_default(),
                    opening: row.get(10).unwrap_or_default(),
                    accuracy_white: row.get(11)?,
                    accuracy_black: row.get(12)?,
                    analysis_json: row.get(13)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(games)
    }

    pub fn get_game(&self, id: i64) -> Result<Option<StoredGame>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, pgn, white, black, white_elo, black_elo, result, date, time_control, eco, opening,
                   accuracy_white, accuracy_black, analysis_json
            FROM games
            WHERE id = ?1
            "#,
        )?;

        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(StoredGame {
                id: row.get(0)?,
                pgn: row.get(1)?,
                white: row.get(2)?,
                black: row.get(3)?,
                white_elo: row.get(4)?,
                black_elo: row.get(5)?,
                result: row.get(6)?,
                date: row.get(7).unwrap_or_default(),
                time_control: row.get(8).unwrap_or_default(),
                eco: row.get(9).unwrap_or_default(),
                opening: row.get(10).unwrap_or_default(),
                accuracy_white: row.get(11)?,
                accuracy_black: row.get(12)?,
                analysis_json: row.get(13)?,
            }))
        } else {
            Ok(None)
        }
    }


    pub fn insert_parsed_game(&self, g: &crate::pgn::ParsedGame) -> Result<i64> {
        let pgn_moves = g.moves.iter().map(|m| m.san.clone()).collect::<Vec<_>>().join(" ");
        self.insert_game(
            &g.white,
            &g.black,
            &g.result,
            g.white_elo,
            g.black_elo,
            &g.date,
            &g.time_control,
            &g.eco,
            &g.opening,
            &pgn_moves,
        )
    }

    pub fn save_analysis(&self, id: i64, analysis: &crate::analysis::GameAnalysis) -> Result<()> {
        let json = serde_json::to_string(analysis).unwrap_or_default();
        self.update_game_analysis(id, analysis.white_accuracy, analysis.black_accuracy, &json)
    }

    pub fn get_analysis(&self, id: i64) -> Result<Option<crate::analysis::GameAnalysis>> {
        if let Some(game) = self.get_game(id)? {
            if let Some(json) = game.analysis_json {
                if let Ok(an) = serde_json::from_str(&json) {
                    return Ok(Some(an));
                }
            }
        }
        Ok(None)
    }

    pub fn update_game_analysis(
        &self,
        id: i64,
        acc_white: f64,
        acc_black: f64,
        analysis_json: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            UPDATE games
            SET accuracy_white = ?1, accuracy_black = ?2, analysis_json = ?3
            WHERE id = ?4
            "#,
            params![acc_white, acc_black, analysis_json, id],
        )?;
        Ok(())
    }

    pub fn get_cached_eval(&self, fen: &str) -> Result<Option<CachedEval>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT fen, depth, score_cp, mate_in, best_move, pv_json
            FROM eval_cache
            WHERE fen = ?1
            "#,
        )?;

        let mut rows = stmt.query([fen])?;
        if let Some(row) = rows.next()? {
            let pv_json: String = row.get(5)?;
            let pv: Vec<String> = serde_json::from_str(&pv_json).unwrap_or_default();
            Ok(Some(CachedEval {
                fen: row.get(0)?,
                depth: row.get(1)?,
                score_cp: row.get(2)?,
                mate_in: row.get(3)?,
                best_move: row.get(4)?,
                pv,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn set_cached_eval(&self, eval: &CachedEval) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let pv_json = serde_json::to_string(&eval.pv)?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO eval_cache (fen, depth, score_cp, mate_in, best_move, pv_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                eval.fen,
                eval.depth,
                eval.score_cp,
                eval.mate_in,
                eval.best_move,
                pv_json
            ],
        )?;
        Ok(())
    }

    pub fn insert_puzzle(&self, puzzle: &Puzzle) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO puzzles (id, fen, moves, rating, rating_deviation, popularity, nb_plays, themes, opening_tags, source)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                puzzle.id,
                puzzle.fen,
                puzzle.moves,
                puzzle.rating,
                puzzle.rating_deviation,
                puzzle.popularity,
                puzzle.nb_plays,
                puzzle.themes,
                puzzle.opening_tags,
                puzzle.source
            ],
        )?;
        Ok(())
    }

    pub fn get_random_puzzle(&self, source_filter: Option<&str>) -> Result<Option<Puzzle>> {
        let conn = self.conn.lock().unwrap();
        let mut query = "SELECT id, fen, moves, rating, rating_deviation, popularity, nb_plays, themes, opening_tags, source FROM puzzles".to_string();
        if let Some(src) = source_filter {
            query.push_str(&format!(" WHERE source = '{}'", src));
        }
        query.push_str(" ORDER BY RANDOM() LIMIT 1");

        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Puzzle {
                id: row.get(0)?,
                fen: row.get(1)?,
                moves: row.get(2)?,
                rating: row.get(3)?,
                rating_deviation: row.get(4)?,
                popularity: row.get(5)?,
                nb_plays: row.get(6)?,
                themes: row.get(7)?,
                opening_tags: row.get(8).unwrap_or_default(),
                source: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_puzzle_count(&self) -> Result<(usize, usize)> {
        let conn = self.conn.lock().unwrap();
        let total: usize = conn.query_row(
            "SELECT count(*) FROM puzzles",
            [],
            |r| r.get(0),
        )?;
        let blunder_count: usize = conn.query_row(
            "SELECT count(*) FROM puzzles WHERE source = 'blunder'",
            [],
            |r| r.get(0),
        )?;
        Ok((total, blunder_count))
    }

    pub fn set_session_value(&self, key: &str, val: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO sessions (key, value) VALUES (?1, ?2)",
            params![key, val],
        )?;
        Ok(())
    }

    pub fn get_session_value(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM sessions WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}
