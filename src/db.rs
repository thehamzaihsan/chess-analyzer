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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PuzzleAttempt {
    pub puzzle_id: String,
    pub seen_count: i64,
    pub attempts: i64,
    pub solved_count: i64,
    pub failed_count: i64,
    pub last_solved: Option<bool>,
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

            CREATE TABLE IF NOT EXISTS puzzle_attempts (
                puzzle_id TEXT PRIMARY KEY,
                seen_count INTEGER NOT NULL DEFAULT 0,
                attempts INTEGER NOT NULL DEFAULT 0,
                solved_count INTEGER NOT NULL DEFAULT 0,
                failed_count INTEGER NOT NULL DEFAULT 0,
                last_solved INTEGER,
                last_seen_at INTEGER NOT NULL DEFAULT 0,
                last_attempted_at INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_puzzle_attempts_seen ON puzzle_attempts(seen_count, last_seen_at);

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
            -- Newest games first. Dates are PGN-style YYYY.MM.DD so they sort
            -- lexically; unknown dates ("????.??.??") go last. Ties fall back to
            -- insertion order, newest insert first.
            ORDER BY (date GLOB '[0-9][0-9][0-9][0-9].[0-9][0-9].[0-9][0-9]') DESC,
                     date DESC,
                     id DESC
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

    /// Inserts a puzzle only if its `id` is not already present in the database.
    /// Returns `Ok(true)` if a new row was inserted, `Ok(false)` if it already existed.
    pub fn insert_puzzle_if_new(&self, puzzle: &Puzzle) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            r#"
            INSERT OR IGNORE INTO puzzles (id, fen, moves, rating, rating_deviation, popularity, nb_plays, themes, opening_tags, source)
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
        Ok(rows > 0)
    }

    /// Imports a slice of puzzles in a single SQLite transaction, optionally
    /// updating a seed version key in `sessions` atomically.
    pub fn import_puzzles_batch(
        &self,
        puzzles: &[Puzzle],
        seed_version: Option<(&str, &str)>,
    ) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO puzzles (id, fen, moves, rating, rating_deviation, popularity, nb_plays, themes, opening_tags, source)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
            )?;
            for puzzle in puzzles {
                stmt.execute(params![
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
                ])?;
            }
        }
        if let Some((key, val)) = seed_version {
            tx.execute(
                "INSERT OR REPLACE INTO sessions (key, value) VALUES (?1, ?2)",
                params![key, val],
            )?;
        }
        tx.commit()?;
        Ok(puzzles.len())
    }

    pub fn count_puzzles_by_source(&self, source: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: usize = conn.query_row(
            "SELECT count(*) FROM puzzles WHERE source = ?1",
            params![source],
            |r| r.get(0),
        )?;
        Ok(count)
    }

    fn next_puzzle_seq(conn: &Connection) -> Result<i64> {
        let seq: i64 = conn.query_row(
            "SELECT COALESCE(MAX(last_seen_at), 0) + 1 FROM puzzle_attempts",
            [],
            |r| r.get(0),
        )?;
        Ok(seq)
    }

    fn mark_puzzle_seen_conn(conn: &Connection, puzzle_id: &str) -> Result<()> {
        let seq = Self::next_puzzle_seq(conn)?;
        conn.execute(
            r#"
            INSERT INTO puzzle_attempts (puzzle_id, seen_count, last_seen_at)
            VALUES (?1, 1, ?2)
            ON CONFLICT(puzzle_id) DO UPDATE SET
                seen_count = puzzle_attempts.seen_count + 1,
                last_seen_at = excluded.last_seen_at
            "#,
            params![puzzle_id, seq],
        )?;
        Ok(())
    }

    /// Marks a puzzle as seen so subsequent selections prefer unseen puzzles.
    pub fn mark_puzzle_seen(&self, puzzle_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        Self::mark_puzzle_seen_conn(&conn, puzzle_id)
    }

    /// Records a solved or failed puzzle attempt for future selection and UI tracking.
    pub fn record_puzzle_attempt(&self, puzzle_id: &str, solved: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let seq = Self::next_puzzle_seq(&conn)?;
        let solved_inc: i64 = if solved { 1 } else { 0 };
        let failed_inc: i64 = if solved { 0 } else { 1 };
        let last_solved_val: i64 = if solved { 1 } else { 0 };
        conn.execute(
            r#"
            INSERT INTO puzzle_attempts (
                puzzle_id, seen_count, attempts, solved_count, failed_count,
                last_solved, last_seen_at, last_attempted_at
            )
            VALUES (?1, 1, 1, ?2, ?3, ?4, ?5, ?5)
            ON CONFLICT(puzzle_id) DO UPDATE SET
                seen_count = MAX(puzzle_attempts.seen_count, 1),
                attempts = puzzle_attempts.attempts + 1,
                solved_count = puzzle_attempts.solved_count + excluded.solved_count,
                failed_count = puzzle_attempts.failed_count + excluded.failed_count,
                last_solved = excluded.last_solved,
                last_seen_at = excluded.last_seen_at,
                last_attempted_at = excluded.last_attempted_at
            "#,
            params![puzzle_id, solved_inc, failed_inc, last_solved_val, seq],
        )?;
        Ok(())
    }

    pub fn get_puzzle_attempt(&self, puzzle_id: &str) -> Result<Option<PuzzleAttempt>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT puzzle_id, seen_count, attempts, solved_count, failed_count, last_solved FROM puzzle_attempts WHERE puzzle_id = ?1",
        )?;
        let mut rows = stmt.query(params![puzzle_id])?;
        if let Some(row) = rows.next()? {
            let last_solved_raw: Option<i64> = row.get(5)?;
            Ok(Some(PuzzleAttempt {
                puzzle_id: row.get(0)?,
                seen_count: row.get(1)?,
                attempts: row.get(2)?,
                solved_count: row.get(3)?,
                failed_count: row.get(4)?,
                last_solved: last_solved_raw.map(|v| v != 0),
            }))
        } else {
            Ok(None)
        }
    }

    /// Selects the next puzzle matching `source_filter` while avoiding repeats
    /// (unseen puzzles first, then unsolved least-recently-seen, then solved
    /// least-recently-seen) and optionally preferring puzzles near `target_rating`.
    pub fn get_next_puzzle(
        &self,
        source_filter: Option<&str>,
        target_rating: Option<i32>,
    ) -> Result<Option<Puzzle>> {
        let conn = self.conn.lock().unwrap();
        let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<Puzzle> {
            Ok(Puzzle {
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
            })
        };

        let selected = if let Some(src) = source_filter {
            let mut stmt = conn.prepare(
                r#"
                SELECT p.id, p.fen, p.moves, p.rating, p.rating_deviation, p.popularity, p.nb_plays, p.themes, p.opening_tags, p.source
                FROM puzzles p
                LEFT JOIN puzzle_attempts pa ON pa.puzzle_id = p.id
                WHERE p.source = ?1
                ORDER BY
                    (pa.puzzle_id IS NULL) DESC,
                    (COALESCE(pa.solved_count, 0) = 0) DESC,
                    COALESCE(pa.seen_count, 0) ASC,
                    COALESCE(pa.last_seen_at, 0) ASC,
                    CASE WHEN ?2 IS NOT NULL THEN (ABS(p.rating - ?2) / 100) ELSE 0 END ASC,
                    RANDOM()
                LIMIT 1
                "#,
            )?;
            let mut rows = stmt.query(params![src, target_rating])?;
            if let Some(row) = rows.next()? {
                Some(map_row(row)?)
            } else {
                None
            }
        } else {
            let mut stmt = conn.prepare(
                r#"
                SELECT p.id, p.fen, p.moves, p.rating, p.rating_deviation, p.popularity, p.nb_plays, p.themes, p.opening_tags, p.source
                FROM puzzles p
                LEFT JOIN puzzle_attempts pa ON pa.puzzle_id = p.id
                ORDER BY
                    (pa.puzzle_id IS NULL) DESC,
                    (COALESCE(pa.solved_count, 0) = 0) DESC,
                    COALESCE(pa.seen_count, 0) ASC,
                    COALESCE(pa.last_seen_at, 0) ASC,
                    CASE WHEN ?1 IS NOT NULL THEN (ABS(p.rating - ?1) / 100) ELSE 0 END ASC,
                    RANDOM()
                LIMIT 1
                "#,
            )?;
            let mut rows = stmt.query(params![target_rating])?;
            if let Some(row) = rows.next()? {
                Some(map_row(row)?)
            } else {
                None
            }
        };

        if let Some(ref puzzle) = selected {
            Self::mark_puzzle_seen_conn(&conn, &puzzle.id)?;
        }

        Ok(selected)
    }

    pub fn get_random_puzzle(&self, source_filter: Option<&str>) -> Result<Option<Puzzle>> {
        self.get_next_puzzle(source_filter, None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn make_puzzle(id: &str, rating: i32, source: &str) -> Puzzle {
        Puzzle {
            id: id.to_string(),
            fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string(),
            moves: "e7e5 g1f3".to_string(),
            rating,
            rating_deviation: 80,
            popularity: 95,
            nb_plays: 1000,
            themes: "fork".to_string(),
            opening_tags: "".to_string(),
            source: source.to_string(),
        }
    }

    #[test]
    fn insert_puzzle_if_new_reports_only_new_rows() {
        let db = Database::memory().unwrap();
        let puzzle = Puzzle {
            id: "blunder_1_5".to_string(),
            fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string(),
            moves: "e7e5".to_string(),
            rating: 1500,
            rating_deviation: 100,
            popularity: 100,
            nb_plays: 1,
            themes: "blunder-recovery".to_string(),
            opening_tags: "".to_string(),
            source: "blunder".to_string(),
        };

        assert!(db.insert_puzzle_if_new(&puzzle).unwrap());
        // Duplicate deterministic id should not be counted as newly inserted
        assert!(!db.insert_puzzle_if_new(&puzzle).unwrap());
        assert_eq!(db.get_puzzle_count().unwrap(), (1, 1));

        let fetched = db.get_random_puzzle(Some("blunder")).unwrap().unwrap();
        assert_eq!(fetched.id, puzzle.id);
        assert!(db.get_random_puzzle(Some("lichess")).unwrap().is_none());
    }

    #[test]
    fn get_next_puzzle_avoids_repeats_and_prefers_unsolved_then_lru() {
        let db = Database::memory().unwrap();
        let puzzles = vec![
            make_puzzle("p1", 1500, "lichess"),
            make_puzzle("p2", 1500, "lichess"),
            make_puzzle("p3", 1500, "lichess"),
        ];
        db.import_puzzles_batch(&puzzles, None).unwrap();

        // First 3 selections must return all 3 distinct puzzles without repeats.
        let mut first_round = HashSet::new();
        for _ in 0..3 {
            let p = db.get_next_puzzle(Some("lichess"), None).unwrap().unwrap();
            assert!(first_round.insert(p.id), "repeated puzzle before unseen exhausted");
        }
        assert_eq!(first_round.len(), 3);

        // Mark p1 and p2 as solved, leave p3 failed/unsolved.
        db.record_puzzle_attempt("p1", true).unwrap();
        db.record_puzzle_attempt("p2", true).unwrap();
        db.record_puzzle_attempt("p3", false).unwrap();

        let att_p3 = db.get_puzzle_attempt("p3").unwrap().unwrap();
        assert_eq!(att_p3.attempts, 1);
        assert_eq!(att_p3.solved_count, 0);
        assert_eq!(att_p3.failed_count, 1);
        assert_eq!(att_p3.last_solved, Some(false));

        // Next selection must prefer unsolved p3 over solved p1/p2.
        let next = db.get_next_puzzle(Some("lichess"), None).unwrap().unwrap();
        assert_eq!(next.id, "p3");
    }

    #[test]
    fn get_next_puzzle_prefers_target_rating_band() {
        let db = Database::memory().unwrap();
        let puzzles = vec![
            make_puzzle("low", 800, "lichess"),
            make_puzzle("mid", 1520, "lichess"),
            make_puzzle("high", 2300, "lichess"),
        ];
        db.import_puzzles_batch(&puzzles, None).unwrap();

        let p = db.get_next_puzzle(Some("lichess"), Some(1500)).unwrap().unwrap();
        assert_eq!(p.id, "mid");

        let p2 = db.get_next_puzzle(Some("lichess"), Some(2280)).unwrap().unwrap();
        assert_eq!(p2.id, "high");

        let p3 = db.get_next_puzzle(Some("lichess"), Some(850)).unwrap().unwrap();
        assert_eq!(p3.id, "low");
    }
}
