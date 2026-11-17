pub mod board;
pub mod bots;
pub mod charts;
pub mod explorer;
pub mod game_layout;
pub mod game_review;
pub mod import;
pub mod insights;
pub mod puzzles;
pub mod shell;
pub mod shell_theme;
pub mod theme;

pub use board::{BoardMessage, BoardWidget};
pub use bots::{BotMessage, BotPlayScreen, BOTS};
pub use charts::{ChartMessage, EvalGraphMessage, EvalGraphWidget, RatingChart};
pub use explorer::{ExplorerMessage, ExplorerScreen};
pub use game_layout::{
    compute_board_layout, render_board_workspace, render_player_bar, render_result_banner,
    render_result_banner_with_tone, render_sized_board_column, render_unified_board_column,
    BoardLayoutMetrics, GameOutcome, OutcomeTone, PlayerInfo, PlayerResult,
};
pub use game_review::{GameReviewMessage, GameReviewScreen};
pub use import::{ImportMessage, ImportScreen};
pub use insights::{InsightsMessage, InsightsScreen};
pub use puzzles::{PuzzleMessage, PuzzleScreen};
