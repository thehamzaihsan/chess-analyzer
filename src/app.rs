use shakmaty::Position;
use crate::analysis::{analyze_game, GameAnalysis};
use crate::chesscom::ChessComClient;
use crate::coach::CoachClient;
use crate::db::{Database, Puzzle, StoredGame};
use crate::engine::uci::{discover_engine_path, UciEngine, UciEval};
use crate::engine::EnginePool;
use crate::insights::{compute_insights, OverallInsights};
use crate::pgn::parse_pgn_games;
use crate::ui::board::BoardMessage;
use crate::ui::bots::{BotMessage, BotPlayScreen, BOTS};
use crate::ui::explorer::{ExplorerMessage, ExplorerScreen};
use crate::ui::game_review::{GameReviewMessage, GameReviewScreen};
use crate::ui::import::{ImportMessage, ImportScreen};
use crate::ui::charts::{ChartMessage, EvalGraphMessage};
use crate::ui::insights::{InsightsMessage, InsightsScreen};
use crate::ui::puzzles::{PuzzleMessage, PuzzleScreen};
use crate::ui::theme::{
    badge_container_style, fonts, base_container_style, custom_dark_theme, icon_book_open, icon_bot,
    icon_folder_open, icon_insights, icon_logo, icon_puzzle, icon_search, icon_widget, nav_button_style,
    palette, spacing, type_scale,
};
use iced::border;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Task, Theme};
use std::collections::HashMap;
use std::sync::Arc;

/// Keeps only games where the tracked player appears as White or Black
/// (case-insensitive). The games table is shared across every player ever
/// looked up, so this is what scopes "Stored Games"/Insights to whichever
/// username is currently selected instead of showing everyone's games mixed
/// together.
fn games_for_player(games: Vec<StoredGame>, username: &str) -> Vec<StoredGame> {
    games
        .into_iter()
        .filter(|g| g.white.eq_ignore_ascii_case(username) || g.black.eq_ignore_ascii_case(username))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Import,
    GameReview,
    Insights,
    Puzzles,
    Bots,
    Explorer,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    ChangeTab(Tab),
    Import(ImportMessage),
    GameReview(GameReviewMessage),
    Insights(InsightsMessage),
    Puzzle(PuzzleMessage),
    Bot(BotMessage),
    Explorer(ExplorerMessage),

    // Background task completions
    GamesImported(Result<Vec<StoredGame>, String>),
    ChessComArchivesListed(Result<Vec<String>, String>),
    ChessComMonthImported(Result<Vec<StoredGame>, String>),
    EngineBatchAnalysisProgress(i64, usize, usize), // (game_id, current_ply, total_plies)
    EngineGameAnalysisFinished(i64, Result<GameAnalysis, String>),
    AllGamesAnalyzed,
    CoachExplanationReceived(Result<String, String>),
    BotEngineMoveFound(String),
}

pub struct App {
    pub active_tab: Tab,
    pub player_username: String,
    pub db: Arc<Database>,
    pub chesscom: ChessComClient,
    pub coach: Arc<CoachClient>,
    pub engine_pool: Option<Arc<EnginePool>>,

    pub games: Vec<StoredGame>,
    pub selected_game_id: Option<i64>,
    pub analyses: HashMap<i64, GameAnalysis>,
    pub insights: OverallInsights,

    // Chess.com fetch is paginated one month at a time (politeness toward the
    // API's rate limits, and so games appear incrementally instead of one big
    // blocking wait for players with years of history).
    pub pending_archives: Vec<String>,
    pub chesscom_fetch_total: usize,
    pub chesscom_fetch_done: usize,

    // Sub-screens
    pub import_screen: ImportScreen,
    pub game_review_screen: GameReviewScreen,
    pub insights_screen: InsightsScreen,
    pub puzzle_screen: PuzzleScreen,
    pub bot_screen: BotPlayScreen,
    pub explorer_screen: ExplorerScreen,
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        let db = Arc::new(Database::open_default().expect("Failed to initialize SQLite database"));
        let default_username = "GothamChess".to_string();

        let engine_pool = EnginePool::new((*db).clone()).ok().map(Arc::new);

        let coach = Arc::new(CoachClient::new(None, None));

        let initial_games = games_for_player(db.list_games().unwrap_or_default(), &default_username);
        let mut analyses = HashMap::new();
        for g in &initial_games {
            if let Ok(Some(an)) = db.get_analysis(g.id) {
                analyses.insert(g.id, an);
            }
        }

        let analyses_vec: Vec<Option<GameAnalysis>> = initial_games
            .iter()
            .map(|g| analyses.get(&g.id).cloned())
            .collect();
        let insights = compute_insights(&default_username, &initial_games, &analyses_vec);

        let mut app = Self {
            active_tab: Tab::Import,
            player_username: default_username.clone(),
            db: Arc::clone(&db),
            chesscom: ChessComClient::new(),
            coach,
            engine_pool,

            games: initial_games,
            selected_game_id: None,
            analyses,
            insights,

            pending_archives: Vec::new(),
            chesscom_fetch_total: 0,
            chesscom_fetch_done: 0,

            import_screen: ImportScreen::new(),
            game_review_screen: GameReviewScreen::new(),
            insights_screen: InsightsScreen::new(),
            puzzle_screen: PuzzleScreen::new(),
            bot_screen: BotPlayScreen::new(),
            explorer_screen: ExplorerScreen::new(),
        };

        app.import_screen.username_input = default_username;

        (app, Task::none())
    }

    pub fn theme(&self) -> Theme {
        custom_dark_theme()
    }

    fn recompute_insights(&mut self) {
        let analyses_vec: Vec<Option<GameAnalysis>> = self
            .games
            .iter()
            .map(|g| self.analyses.get(&g.id).cloned())
            .collect();
        self.insights = compute_insights(&self.player_username, &self.games, &analyses_vec);
    }

    /// Dequeues and fetches the next pending Chess.com monthly archive. One
    /// request at a time (not parallel) so we stay polite to the API's rate
    /// limits; `ChessComClient::get_with_retry` still backs off on a 429 on
    /// top of this.
    fn fetch_next_archive(&mut self) -> Task<AppMessage> {
        if self.pending_archives.is_empty() {
            return Task::none();
        }
        // pending_archives is newest-first; always take from the front so we
        // process months in that order.
        let next_url = self.pending_archives.remove(0);

        self.import_screen.status_message = Some(format!(
            "Fetching month {}/{} ({})...",
            self.chesscom_fetch_done + 1,
            self.chesscom_fetch_total,
            next_url.rsplit('/').take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("/"),
        ));

        let client = self.chesscom.clone();
        let db = Arc::clone(&self.db);
        let username = self.player_username.clone();

        Task::perform(
            async move {
                // Small pacing delay between months, on top of the client's
                // own 429 retry/backoff, so a long history doesn't fire off
                // requests as fast as possible.
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                let pgns = client.fetch_archive_pgns(&next_url).await.map_err(|e| e.to_string())?;
                for pgn in &pgns {
                    for g in parse_pgn_games(pgn) {
                        let _ = db.insert_parsed_game(&g);
                    }
                }
                let games = db.list_games().map_err(|e| e.to_string())?;
                Ok(games_for_player(games, &username))
            },
            AppMessage::ChessComMonthImported,
        )
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::ChangeTab(tab) => {
                self.active_tab = tab;
                if tab == Tab::Insights {
                    self.recompute_insights();
                } else if tab == Tab::Puzzles && self.puzzle_screen.current_puzzle.is_none() {
                    if let Ok(Some(p)) = self.db.get_random_puzzle(self.puzzle_screen.filter.as_deref()) {
                        self.puzzle_screen.load_puzzle(p);
                    }
                }
                Task::none()
            }

            // --- Import Screen Messages ---
            AppMessage::Import(ImportMessage::UsernameChanged(u)) => {
                self.import_screen.username_input = u;
                Task::none()
            }
            AppMessage::Import(ImportMessage::PgnInputChanged(p)) => {
                self.import_screen.pgn_input = p;
                Task::none()
            }
            AppMessage::Import(ImportMessage::FetchChessCom) => {
                self.import_screen.is_loading = true;
                self.import_screen.current_page = 0;
                let username = self.import_screen.username_input.trim().to_string();
                self.player_username = username.clone();
                self.import_screen.status_message = Some(format!("Looking up {}'s archive list...", username));
                let client = self.chesscom.clone();

                Task::perform(
                    async move { client.fetch_user_archives(&username).await.map_err(|e| e.to_string()) },
                    AppMessage::ChessComArchivesListed,
                )
            }
            AppMessage::ChessComArchivesListed(Ok(mut archives)) => {
                // Chess.com returns oldest-first; fetch newest-first so recent
                // games show up first instead of waiting on years of history.
                archives.reverse();
                self.chesscom_fetch_total = archives.len();
                self.chesscom_fetch_done = 0;
                self.pending_archives = archives;

                if self.pending_archives.is_empty() {
                    self.import_screen.is_loading = false;
                    self.import_screen.status_message =
                        Some(format!("No games found for {}.", self.player_username));
                    Task::none()
                } else {
                    self.fetch_next_archive()
                }
            }
            AppMessage::ChessComArchivesListed(Err(e)) => {
                self.import_screen.is_loading = false;
                self.import_screen.status_message = Some(format!("Failed to fetch archives: {}", e));
                Task::none()
            }
            AppMessage::ChessComMonthImported(result) => {
                self.chesscom_fetch_done += 1;
                match result {
                    Ok(games) => {
                        self.games = games;
                        self.recompute_insights();
                    }
                    Err(e) => {
                        self.import_screen.status_message = Some(format!(
                            "Month {}/{} failed ({}), continuing...",
                            self.chesscom_fetch_done, self.chesscom_fetch_total, e
                        ));
                    }
                }

                if self.pending_archives.is_empty() {
                    self.import_screen.is_loading = false;
                    self.import_screen.status_message = Some(format!(
                        "Done — imported {} games across {} months for {}.",
                        self.games.len(), self.chesscom_fetch_total, self.player_username
                    ));
                    Task::none()
                } else {
                    self.fetch_next_archive()
                }
            }
            AppMessage::Import(ImportMessage::LoadPgnText) => {
                self.import_screen.current_page = 0;
                let pgn_text = self.import_screen.pgn_input.clone();
                let db = Arc::clone(&self.db);

                Task::perform(
                    async move {
                        let parsed = parse_pgn_games(&pgn_text);
                        for g in &parsed {
                            let _ = db.insert_parsed_game(g);
                        }
                        db.list_games().map_err(|e| e.to_string())
                    },
                    AppMessage::GamesImported,
                )
            }
            AppMessage::Import(ImportMessage::LoadSamplePgn) => {
                self.import_screen.current_page = 0;
                let db = Arc::clone(&self.db);
                Task::perform(
                    async move {
                        let paths = [
                            "chess_com_games_2026-09-30.pgn",
                            "../chess_com_games_2026-09-30.pgn",
                            "../../chess_com_games_2026-09-30.pgn",
                        ];
                        let mut pgn_content = None;
                        for p in &paths {
                            if let Ok(content) = std::fs::read_to_string(p) {
                                pgn_content = Some(content);
                                break;
                            }
                        }

                        let pgn_text = pgn_content.ok_or_else(|| "Sample PGN file not found in workspace root".to_string())?;
                        let parsed = parse_pgn_games(&pgn_text);
                        for g in &parsed {
                            let _ = db.insert_parsed_game(g);
                        }
                        db.list_games().map_err(|e| e.to_string())
                    },
                    AppMessage::GamesImported,
                )
            }
            AppMessage::Import(ImportMessage::SelectGameForReview(game_id)) => {
                self.selected_game_id = Some(game_id);
                self.game_review_screen.current_move_idx = 0;
                self.game_review_screen.coach_explanation = None;

                if let Some(g) = self.games.iter().find(|g| g.id == game_id) {
                    self.game_review_screen.load_game(g);
                }

                self.active_tab = Tab::GameReview;
                Task::none()
            }
            AppMessage::Import(ImportMessage::StartGameAnalysis(game_id)) => {
                self.analyze_single_game(game_id)
            }
            AppMessage::Import(ImportMessage::AnalyzeAllGames) => {
                let unanalyzed: Vec<i64> = self
                    .games
                    .iter()
                    .filter(|g| !self.analyses.contains_key(&g.id))
                    .map(|g| g.id)
                    .collect();

                if let Some(first) = unanalyzed.first() {
                    self.import_screen.status_message = Some(format!("Analyzing {} games in background...", unanalyzed.len()));
                    self.analyze_single_game(*first)
                } else {
                    self.import_screen.status_message = Some("All games already analyzed!".to_string());
                    Task::none()
                }
            }
            AppMessage::Import(ImportMessage::NextPage) => {
                self.import_screen.current_page += 1;
                Task::none()
            }
            AppMessage::Import(ImportMessage::PrevPage) => {
                self.import_screen.current_page = self.import_screen.current_page.saturating_sub(1);
                Task::none()
            }

            // --- Async Task Completion Handlers ---
            AppMessage::GamesImported(Ok(games)) => {
                self.import_screen.is_loading = false;
                self.games = games_for_player(games, &self.player_username);
                self.import_screen.status_message = Some(format!("Successfully imported {} total games.", self.games.len()));
                self.recompute_insights();
                Task::none()
            }
            AppMessage::GamesImported(Err(err)) => {
                self.import_screen.is_loading = false;
                self.import_screen.status_message = Some(format!("Import error: {}", err));
                Task::none()
            }
            AppMessage::EngineBatchAnalysisProgress(_game_id, _current, _total) => {
                Task::none()
            }
            AppMessage::EngineGameAnalysisFinished(game_id, Ok(analysis)) => {
                let _ = self.db.save_analysis(game_id, &analysis);
                self.analyses.insert(game_id, analysis);
                self.recompute_insights();

                // Check if more games need analysis
                let remaining: Vec<i64> = self
                    .games
                    .iter()
                    .filter(|g| !self.analyses.contains_key(&g.id))
                    .map(|g| g.id)
                    .collect();

                if let Some(next_id) = remaining.first() {
                    self.analyze_single_game(*next_id)
                } else {
                    self.import_screen.status_message = Some("All game analyses complete!".to_string());
                    Task::none()
                }
            }
            AppMessage::EngineGameAnalysisFinished(_game_id, Err(err)) => {
                self.import_screen.status_message = Some(format!("Analysis failed: {}", err));
                Task::none()
            }
            AppMessage::AllGamesAnalyzed => {
                self.import_screen.status_message = Some("All games analyzed!".to_string());
                Task::none()
            }

            // --- Game Review Messages ---
            AppMessage::GameReview(GameReviewMessage::Board(bm)) => {
                match bm {
                    BoardMessage::SquareClicked(sq) => {
                        self.game_review_screen.board_widget.selected_square = Some(sq);
                    }
                }
                Task::none()
            }
            AppMessage::GameReview(GameReviewMessage::GoToMove(idx)) => {
                self.game_review_screen.current_move_idx = idx;
                self.game_review_screen.coach_explanation = None;
                if let Some(game_id) = self.selected_game_id {
                    if let Some(g) = self.games.iter().find(|g| g.id == game_id) {
                        self.game_review_screen.update_board_to_move(g, idx);
                    }
                }
                Task::none()
            }
            AppMessage::GameReview(GameReviewMessage::PrevMove) => {
                if self.game_review_screen.current_move_idx > 0 {
                    let next_idx = self.game_review_screen.current_move_idx - 1;
                    self.update(AppMessage::GameReview(GameReviewMessage::GoToMove(next_idx)))
                } else {
                    Task::none()
                }
            }
            AppMessage::GameReview(GameReviewMessage::NextMove) => {
                let max_moves = self
                    .selected_game_id
                    .and_then(|id| self.analyses.get(&id))
                    .map(|an| an.moves.len())
                    .unwrap_or(0);

                if self.game_review_screen.current_move_idx < max_moves {
                    let next_idx = self.game_review_screen.current_move_idx + 1;
                    self.update(AppMessage::GameReview(GameReviewMessage::GoToMove(next_idx)))
                } else {
                    Task::none()
                }
            }
            AppMessage::GameReview(GameReviewMessage::FirstMove) => {
                self.update(AppMessage::GameReview(GameReviewMessage::GoToMove(0)))
            }
            AppMessage::GameReview(GameReviewMessage::LastMove) => {
                let max_moves = self
                    .selected_game_id
                    .and_then(|id| self.analyses.get(&id))
                    .map(|an| an.moves.len())
                    .unwrap_or(0);
                self.update(AppMessage::GameReview(GameReviewMessage::GoToMove(max_moves)))
            }
            AppMessage::GameReview(GameReviewMessage::FlipBoard) => {
                self.game_review_screen.board_widget.flipped = !self.game_review_screen.board_widget.flipped;
                Task::none()
            }
            AppMessage::GameReview(GameReviewMessage::AnalyzeCurrentGame) => {
                if let Some(game_id) = self.selected_game_id {
                    self.analyze_single_game(game_id)
                } else {
                    Task::none()
                }
            }
            AppMessage::GameReview(GameReviewMessage::AskCoach) => {
                if let Some(game_id) = self.selected_game_id {
                    if let Some(an) = self.analyses.get(&game_id) {
                        let idx = self.game_review_screen.current_move_idx;
                        if idx > 0 && idx <= an.moves.len() {
                            let m = &an.moves[idx - 1];
                            self.game_review_screen.is_coach_loading = true;
                            let coach = Arc::clone(&self.coach);

                            let fen = m.fen_before.clone();
                            let played_san = m.san.clone();
                            let classification = m.classification.name().to_string();
                            let loss = m.win_prob_loss;
                            let best_line: Vec<String> = m
                                .eval_before
                                .as_ref()
                                .and_then(|e| e.best_move.clone())
                                .map(|s| vec![s])
                                .unwrap_or_default();

                            return Task::perform(
                                async move {
                                    coach
                                        .explain_move(&fen, &played_san, &classification, loss, &best_line)
                                        .await
                                        .map_err(|e| e.to_string())
                                                                },
                                AppMessage::CoachExplanationReceived,
                            );
                        }
                    }
                }
                Task::none()
            }
            AppMessage::GameReview(GameReviewMessage::EvalGraph(egm)) => {
                match egm {
                    EvalGraphMessage::SelectPly(ply) => {
                        self.update(AppMessage::GameReview(GameReviewMessage::GoToMove(ply)))
                    }
                }
            }
            AppMessage::CoachExplanationReceived(Ok(explanation)) => {
                self.game_review_screen.is_coach_loading = false;
                self.game_review_screen.coach_explanation = Some(explanation);
                Task::none()
            }
            AppMessage::CoachExplanationReceived(Err(err)) => {
                self.game_review_screen.is_coach_loading = false;
                self.game_review_screen.coach_explanation = Some(format!(
                    "Packaged Gemma coach could not be reached: {err}. Please ensure `assets/coach/linux/llama-cli` and `assets/models/gemma/*.gguf` are present."
                ));
                Task::none()
            }

            // --- Insights Screen Messages ---
            AppMessage::Insights(InsightsMessage::SelectSection(sec)) => {
                self.insights_screen.current_section = sec;
                Task::none()
            }

            AppMessage::Insights(InsightsMessage::Chart(cm)) => {
                match cm {
                    ChartMessage::PointClicked(idx) => {
                        if let Some(pt) = self.insights.rating_history.get(idx) {
                            if let Some(g) = self.games.iter().find(|g| {
                                g.date == pt.date && (g.white == pt.opponent || g.black == pt.opponent)
                            }) {
                                let gid = g.id;
                                self.selected_game_id = Some(gid);
                                self.game_review_screen.load_game(g);
                                if let Some(an) = self.analyses.get(&gid) {
                                    self.game_review_screen.set_position_from_analysis(Some(an), 0);
                                }
                                self.active_tab = Tab::GameReview;
                            }
                        }
                        Task::none()
                    }
                    ChartMessage::PointHovered(_) => Task::none(),
                }
            }

            // --- Puzzles Screen Messages ---
            AppMessage::Puzzle(PuzzleMessage::Board(bm)) => {
                match bm {
                    BoardMessage::SquareClicked(sq) => {
                        self.puzzle_screen.handle_square_clicked(sq);
                    }
                }
                Task::none()
            }
            AppMessage::Puzzle(PuzzleMessage::NextPuzzle) => {
                if let Ok(Some(p)) = self.db.get_random_puzzle(self.puzzle_screen.filter.as_deref()) {
                    self.puzzle_screen.load_puzzle(p);
                } else {
                    self.puzzle_screen.puzzle_status = "No puzzles found in database for current filter.".to_string();
                }
                Task::none()
            }
            AppMessage::Puzzle(PuzzleMessage::MineBlunderPuzzles) => {
                let mut mined = 0;
                for (game_id, analysis) in &self.analyses {
                    for m in &analysis.moves {
                        if m.classification == crate::analysis::MoveClassification::Blunder {
                            if let Some(best) = m.eval_before.as_ref().and_then(|e| e.best_move.clone()) {
                                let p = Puzzle {
                                    id: format!("blunder_{}_{}", game_id, m.ply),
                                    fen: m.fen_before.clone(),
                                    moves: best,
                                    rating: 1500,
                                    rating_deviation: 100,
                                    popularity: 100,
                                    nb_plays: 1,
                                    themes: "blunder-recovery".to_string(),
                                    opening_tags: "".to_string(),
                                    source: "blunder".to_string(),
                                };
                                let _ = self.db.insert_puzzle(&p);
                                mined += 1;
                            }
                        }
                    }
                }
                self.puzzle_screen.puzzle_status = format!("Mined {} blunder puzzles into database!", mined);
                Task::none()
            }
            AppMessage::Puzzle(PuzzleMessage::SelectFilter(f)) => {
                self.puzzle_screen.filter = f;
                if let Ok(Some(p)) = self.db.get_random_puzzle(self.puzzle_screen.filter.as_deref()) {
                    self.puzzle_screen.load_puzzle(p);
                }
                Task::none()
            }

            // --- Bots Screen Messages ---
            AppMessage::Bot(BotMessage::Board(bm)) => {
                match bm {
                    BoardMessage::SquareClicked(sq) => {
                        if let Some(fen) = self.bot_screen.handle_square_clicked(sq) {
                            // Player just made a move, trigger engine response for selected bot
                            let bot = &BOTS[self.bot_screen.selected_bot_idx];
                            let bot_elo = bot.elo;

                            return Task::perform(
                                async move {
                                    if let Some(engine_path) = discover_engine_path("stockfish") {
                                        if let Ok(mut engine) = UciEngine::spawn(&engine_path, &[]).await {
                                            let _ = engine.configure_bot(bot_elo).await;
                                            let best = engine.search_move(&fen, 600).await;
                                            let _ = engine.quit().await;
                                            return best.unwrap_or_else(|_| "e7e5".to_string());
                                        }
                                    }
                                    "e7e5".to_string()
                                },
                                AppMessage::BotEngineMoveFound,
                            );
                        }
                    }
                }
                Task::none()
            }
            AppMessage::Bot(BotMessage::SelectBot(idx)) => {
                self.bot_screen.selected_bot_idx = idx;
                self.bot_screen.reset_game();
                Task::none()
            }
            AppMessage::Bot(BotMessage::NewGame) => {
                self.bot_screen.reset_game();
                Task::none()
            }
            AppMessage::Bot(BotMessage::EngineMoveMade(uci_str)) => {
                self.bot_screen.apply_engine_move(&uci_str);
                Task::none()
            }
            AppMessage::BotEngineMoveFound(uci_str) => {
                self.bot_screen.apply_engine_move(&uci_str);
                Task::none()
            }

            // --- Explorer Screen Messages ---
            AppMessage::Explorer(ExplorerMessage::Board(bm)) => {
                match bm {
                    BoardMessage::SquareClicked(sq) => {
                        if let Some(selected) = self.explorer_screen.board_widget.selected_square {
                            let moves = self.explorer_screen.board_widget.position.legal_moves();
                            for m in moves {
                                if m.from() == Some(selected) && m.to() == sq {
                                    let san = shakmaty::san::SanPlus::from_move(
                                        self.explorer_screen.board_widget.position.clone(),
                                        m,
                                    )
                                    .to_string();
                                    self.explorer_screen.play_san(&san);
                                    break;
                                }
                            }
                            self.explorer_screen.board_widget.selected_square = None;
                        } else {
                            if self.explorer_screen.board_widget.position.board().color_at(sq)
                                == Some(self.explorer_screen.board_widget.position.turn())
                            {
                                self.explorer_screen.board_widget.selected_square = Some(sq);
                            }
                        }
                    }
                }
                Task::none()
            }
            AppMessage::Explorer(ExplorerMessage::PlayMove(san)) => {
                self.explorer_screen.play_san(&san);
                Task::none()
            }
            AppMessage::Explorer(ExplorerMessage::ResetBoard) => {
                self.explorer_screen.reset();
                Task::none()
            }
            AppMessage::Explorer(ExplorerMessage::UndoMove) => {
                self.explorer_screen.undo();
                Task::none()
            }
        }
    }

    fn analyze_single_game(&self, game_id: i64) -> Task<AppMessage> {
        let game = self.games.iter().find(|g| g.id == game_id).cloned();
        let Some(first_game) = game else {
            return Task::none();
        };

        Task::perform(
            async move {
                let engine_path = discover_engine_path("stockfish")
                    .ok_or_else(|| "Stockfish engine not found".to_string())?;
                let mut engine = UciEngine::spawn(&engine_path, &[])
                    .await
                    .map_err(|e| e.to_string())?;

                let parsed = parse_pgn_games(&first_game.pgn);
                let moves = parsed.first().map(|g| g.moves.clone()).unwrap_or_default();

                let mut evals: Vec<Option<UciEval>> = Vec::new();
                for ply in &moves {
                    match engine.evaluate_fen(&ply.fen_after, 12, Some(500)).await {
                        Ok(eval) => evals.push(Some(eval)),
                        Err(_) => evals.push(None),
                    }
                }

                let _ = engine.quit().await;
                let analysis = analyze_game(&moves, &evals);
                Ok(analysis)
            },
            move |res| AppMessage::EngineGameAnalysisFinished(game_id, res),
        )
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        let nav_btn = |tab: Tab, label: &'static str, icon_handle: iced::widget::svg::Handle| {
            let is_selected = self.active_tab == tab;
            let icon_color = if is_selected {
                palette::TEXT_PRIMARY
            } else {
                palette::TEXT_SECONDARY
            };

            let content = row![
                icon_widget(icon_handle, 16.0, Some(icon_color)),
                text(label).font(fonts::UI_SEMIBOLD).size(type_scale::HEADING),
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center);

            button(content)
                .padding([spacing::XS, spacing::MD])
                .style(nav_button_style(is_selected))
                .on_press(AppMessage::ChangeTab(tab))
        };

        let app_title = row![
            icon_widget(icon_logo(), 28.0, None),
            Space::with_width(spacing::XS),
            text("CHESS ANALYZER").font(fonts::UI_BOLD).size(type_scale::TITLE).color(palette::TEXT_PRIMARY),
            Space::with_width(spacing::XS),
            container(text("RUST").font(fonts::MONO_BOLD).size(type_scale::CAPTION).color(palette::ACCENT))
                .padding([2.0, 6.0])
                .style(badge_container_style(palette::ACCENT_MUTED)),
        ]
        .align_y(Alignment::Center);

        let top_nav = row![
            app_title,
            Space::with_width(spacing::LG),
            nav_btn(Tab::Import, "Import", icon_folder_open()),
            nav_btn(Tab::GameReview, "Game Review", icon_search()),
            nav_btn(Tab::Insights, "Insights", icon_insights()),
            nav_btn(Tab::Puzzles, "Puzzles", icon_puzzle()),
            nav_btn(Tab::Bots, "Play Bots", icon_bot()),
            nav_btn(Tab::Explorer, "Explorer", icon_book_open()),
        ]
        .spacing(spacing::XS)
        .padding([spacing::SM, spacing::LG])
        .align_y(Alignment::Center);

        let top_bar = container(top_nav)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(palette::BG_SURFACE.into()),
                border: border::rounded(0).color(palette::BORDER).width(1.0),
                ..Default::default()
            });

        let main_view: Element<'_, AppMessage> = match self.active_tab {
            Tab::Import => self.import_screen.view(&self.games).map(AppMessage::Import),
            Tab::GameReview => {
                let current_game = self.selected_game_id.and_then(|id| self.games.iter().find(|g| g.id == id));
                let current_analysis = self.selected_game_id.and_then(|id| self.analyses.get(&id));
                self.game_review_screen
                    .view(current_game, current_analysis)
                    .map(AppMessage::GameReview)
            }
            Tab::Insights => self.insights_screen.view(&self.insights).map(AppMessage::Insights),
            Tab::Puzzles => self.puzzle_screen.view().map(AppMessage::Puzzle),
            Tab::Bots => self.bot_screen.view().map(AppMessage::Bot),
            Tab::Explorer => self.explorer_screen.view().map(AppMessage::Explorer),
        };

        column![
            top_bar,
            container(main_view).style(base_container_style),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
