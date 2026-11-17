use crate::ui::board::{BoardMessage, BoardWidget};
use crate::ui::game_layout::{
    render_result_banner_with_tone, render_unified_board_column, GameOutcome, OutcomeTone,
    PlayerInfo, PlayerResult,
};
use crate::ui::theme::{
    card_container_style, danger_button_style, fonts, icon_flag, icon_play, icon_refresh_cw,
    icon_widget, nav_button_style, palette, primary_button_style, secondary_button_style,
    spacing, type_scale,
};
use iced::border;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};
use shakmaty::{CastlingMode, Chess, Color, EnPassantMode, Move, Position, Role, Square};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerColorChoice {
    White,
    Black,
    Random,
}

#[derive(Debug, Clone)]
pub enum BotMessage {
    Board(BoardMessage),
    SelectBot(usize),
    SelectColor(PlayerColorChoice),
    NewGame,
    Resign,
    SelectPromotion(Role),
    CancelPromotion,
    RetryBotMove,
    PlayFallbackMove,
    EngineMoveMade(String), // UCI string
}

#[derive(Debug, Clone)]
pub struct BotProfile {
    pub name: &'static str,
    pub title: &'static str,
    pub elo: u32,
    pub bio: &'static str,
}

pub const BOTS: &[BotProfile] = &[
    BotProfile {
        name: "Casual Scout",
        title: "Beginner",
        elo: 1350,
        bio: "Plays relaxed moves, occasionally misses tactics and overlooks hanging pieces.",
    },
    BotProfile {
        name: "Club Tactician",
        title: "Intermediate",
        elo: 1600,
        bio: "Consistent opening play with active piece development, vigilant against 1-move blunders.",
    },
    BotProfile {
        name: "Tournament Candidate",
        title: "Advanced",
        elo: 1900,
        bio: "Strong positional understanding and sharp calculation in complex tactical positions.",
    },
    BotProfile {
        name: "Master Analyst",
        title: "Master",
        elo: 2200,
        bio: "Near-flawless endgame conversion and deep opening preparation.",
    },
];

fn position_repetition_key(pos: &Chess) -> String {
    let fen = shakmaty::fen::Fen::from_position(pos, EnPassantMode::Legal).to_string();
    fen.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
}

fn pseudo_random_usize(salt: u64) -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ (d.as_secs() << 17))
        .unwrap_or(0);
    let mixed = nanos ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (mixed ^ (mixed >> 13)) as usize
}

pub struct BotPlayScreen {
    pub selected_bot_idx: usize,
    pub board_widget: BoardWidget,
    pub is_engine_thinking: bool,
    pub game_status: String,
    pub player_is_white: bool,
    pub color_choice: PlayerColorChoice,
    pub game_generation: u64,
    pub pending_promotion: Option<(Square, Square)>,
    pub engine_error: Option<String>,
    pub is_game_over: bool,
    pub game_outcome: Option<GameOutcome>,
    pub position_counts: HashMap<String, u32>,
}

impl BotPlayScreen {
    pub fn new() -> Self {
        let pos = Chess::default();
        let mut position_counts = HashMap::new();
        position_counts.insert(position_repetition_key(&pos), 1);

        Self {
            selected_bot_idx: 1, // Default: Club Tactician (1600)
            board_widget: BoardWidget::new(pos),
            is_engine_thinking: false,
            game_status: "Your turn! Make a move to start playing.".to_string(),
            player_is_white: true,
            color_choice: PlayerColorChoice::White,
            game_generation: 0,
            pending_promotion: None,
            engine_error: None,
            is_game_over: false,
            game_outcome: None,
            position_counts,
        }
    }

    pub fn current_bot(&self) -> &'static BotProfile {
        &BOTS[self.selected_bot_idx]
    }

    pub fn is_player_turn(&self) -> bool {
        let white_to_move = self.board_widget.position.turn() == Color::White;
        white_to_move == self.player_is_white
    }

    fn record_current_position(&mut self) -> u32 {
        let key = position_repetition_key(&self.board_widget.position);
        let entry = self.position_counts.entry(key).or_insert(0);
        *entry += 1;
        *entry
    }

    fn evaluate_game_over(
        &self,
        user_just_moved: bool,
        rep_count: u32,
    ) -> Option<(GameOutcome, String)> {
        let pos = &self.board_widget.position;
        if pos.is_checkmate() {
            if user_just_moved {
                let res = if self.player_is_white { "1-0" } else { "0-1" };
                Some((
                    GameOutcome::new(res, "Checkmate — You won!"),
                    "Checkmate! You won!".to_string(),
                ))
            } else {
                let res = if self.player_is_white { "0-1" } else { "1-0" };
                let bot_name = self.current_bot().name;
                Some((
                    GameOutcome::new(res, format!("Checkmate — {} won", bot_name)),
                    format!("Checkmate! {} won.", bot_name),
                ))
            }
        } else if pos.is_stalemate() {
            Some((
                GameOutcome::new("1/2-1/2", "Draw by stalemate"),
                "Draw by stalemate.".to_string(),
            ))
        } else if pos.is_insufficient_material() {
            Some((
                GameOutcome::new("1/2-1/2", "Draw by insufficient material"),
                "Draw by insufficient material.".to_string(),
            ))
        } else if rep_count >= 3 {
            Some((
                GameOutcome::new("1/2-1/2", "Draw by repetition"),
                "Draw by threefold repetition.".to_string(),
            ))
        } else if pos.halfmoves() >= 100 {
            Some((
                GameOutcome::new("1/2-1/2", "Draw by 50-move rule"),
                "Draw by 50-move rule.".to_string(),
            ))
        } else if pos.is_game_over() {
            Some((
                GameOutcome::new("1/2-1/2", "Game drawn"),
                "Game drawn.".to_string(),
            ))
        } else {
            None
        }
    }

    /// Resets the board for a new game. Returns `Some(fen)` if the bot plays White
    /// and should immediately search for its opening move.
    pub fn reset_game(&mut self) -> Option<String> {
        crate::audio::play_sound(crate::audio::SoundType::GameStart);
        self.game_generation = self.game_generation.wrapping_add(1);

        self.player_is_white = match self.color_choice {
            PlayerColorChoice::White => true,
            PlayerColorChoice::Black => false,
            PlayerColorChoice::Random => pseudo_random_usize(self.game_generation) % 2 == 0,
        };

        let pos = Chess::default();
        self.board_widget = BoardWidget::new(pos);
        self.board_widget.flipped = !self.player_is_white;
        self.board_widget.interactive = true;
        self.pending_promotion = None;
        self.engine_error = None;
        self.is_game_over = false;
        self.game_outcome = None;
        self.position_counts.clear();
        self.record_current_position();

        if !self.player_is_white {
            self.is_engine_thinking = true;
            self.game_status = format!("{} is thinking...", self.current_bot().name);
            Some(self.board_widget.position_fen())
        } else {
            self.is_engine_thinking = false;
            self.game_status = "New game started! Your turn (White to move).".to_string();
            None
        }
    }

    pub fn resign(&mut self) {
        if self.is_game_over {
            return;
        }
        crate::audio::play_sound(crate::audio::SoundType::GameEnd);
        self.game_generation = self.game_generation.wrapping_add(1);
        self.is_engine_thinking = false;
        self.is_game_over = true;
        self.pending_promotion = None;
        self.engine_error = None;
        self.board_widget.selected_square = None;
        self.board_widget.interactive = false;
        let res = if self.player_is_white { "0-1" } else { "1-0" };
        self.game_outcome = Some(GameOutcome::new(res, "You resigned"));
        self.game_status = format!("You resigned — {} won.", self.current_bot().name);
    }

    pub fn handle_square_clicked(&mut self, sq: Square) -> Option<String> {
        if self.is_engine_thinking || self.is_game_over || !self.is_player_turn() || self.engine_error.is_some() {
            return None;
        }

        if self.pending_promotion.is_some() {
            self.pending_promotion = None;
            if self.board_widget.position.board().color_at(sq) == Some(self.board_widget.position.turn()) {
                self.board_widget.selected_square = Some(sq);
            } else {
                self.board_widget.selected_square = None;
            }
            self.game_status = "Promotion cancelled. Your turn.".to_string();
            return None;
        }

        if let Some(selected) = self.board_widget.selected_square {
            if selected == sq {
                self.board_widget.selected_square = None;
                return None;
            }

            let matching: Vec<Move> = self
                .board_widget
                .position
                .legal_moves()
                .into_iter()
                .filter(|m| m.from() == Some(selected) && m.to() == sq)
                .collect();

            if matching.iter().any(|m| m.promotion().is_some()) {
                self.pending_promotion = Some((selected, sq));
                self.board_widget.selected_square = None;
                self.game_status = "Choose promotion piece.".to_string();
                return None;
            }

            if let Some(m) = matching.into_iter().next() {
                return self.apply_user_move(m);
            }

            if self.board_widget.position.board().color_at(sq) == Some(self.board_widget.position.turn()) {
                self.board_widget.selected_square = Some(sq);
            } else {
                self.board_widget.selected_square = None;
            }
            None
        } else {
            if self.board_widget.position.board().color_at(sq) == Some(self.board_widget.position.turn()) {
                self.board_widget.selected_square = Some(sq);
            }
            None
        }
    }

    pub fn handle_promotion(&mut self, role: Role) -> Option<String> {
        if self.is_engine_thinking || self.is_game_over || !self.is_player_turn() {
            self.pending_promotion = None;
            return None;
        }

        let (from, to) = self.pending_promotion.take()?;
        let matching_move = self
            .board_widget
            .position
            .legal_moves()
            .into_iter()
            .find(|m| m.from() == Some(from) && m.to() == to && m.promotion() == Some(role))?;

        self.apply_user_move(matching_move)
    }

    pub fn cancel_promotion(&mut self) {
        self.pending_promotion = None;
        if !self.is_game_over && self.is_player_turn() {
            self.game_status = "Your turn! Make your move.".to_string();
        }
    }

    fn apply_user_move(&mut self, m: Move) -> Option<String> {
        let from = m.from().unwrap_or(m.to());
        let to = m.to();
        if let Ok(next_pos) = self.board_widget.position.clone().play(m.clone()) {
            let sound = crate::audio::sound_for_move(&m, &next_pos);
            crate::audio::play_sound(sound);
            self.board_widget.position = next_pos;
            self.board_widget.last_move = Some((from, to));
            self.board_widget.selected_square = None;
            self.pending_promotion = None;
            self.engine_error = None;

            let rep_count = self.record_current_position();
            if let Some((outcome, status)) = self.evaluate_game_over(true, rep_count) {
                crate::audio::play_sound(crate::audio::SoundType::GameEnd);
                self.is_game_over = true;
                self.is_engine_thinking = false;
                self.board_widget.interactive = false;
                self.game_outcome = Some(outcome);
                self.game_status = status;
                return None;
            }

            self.is_engine_thinking = true;
            self.game_status = format!("{} is thinking...", self.current_bot().name);
            return Some(self.board_widget.position_fen());
        }
        None
    }

    pub fn handle_user_move(&mut self, from: Square, to: Square) -> Option<String> {
        if self.is_engine_thinking || self.is_game_over || !self.is_player_turn() {
            return None;
        }

        let matching: Vec<Move> = self
            .board_widget
            .position
            .legal_moves()
            .into_iter()
            .filter(|m| m.from() == Some(from) && m.to() == to)
            .collect();

        if matching.iter().any(|m| m.promotion().is_some()) {
            self.pending_promotion = Some((from, to));
            self.board_widget.selected_square = None;
            self.game_status = "Choose promotion piece.".to_string();
            return None;
        }

        if let Some(m) = matching.into_iter().next() {
            return self.apply_user_move(m);
        }
        None
    }

    pub fn apply_engine_result(&mut self, generation: u64, result: Result<String, String>) {
        if generation != self.game_generation {
            return;
        }

        self.is_engine_thinking = false;

        if self.is_game_over || self.is_player_turn() {
            return;
        }

        match result {
            Ok(uci_str) => {
                let parsed = uci_str
                    .trim()
                    .parse::<shakmaty::uci::UciMove>()
                    .ok()
                    .and_then(|m| m.to_move(&self.board_widget.position).ok());

                let Some(valid_m) = parsed else {
                    let msg = format!("Engine returned invalid move '{}'.", uci_str);
                    self.engine_error = Some(msg.clone());
                    self.game_status = format!("{} Retry or play a fallback move.", msg);
                    return;
                };

                let from = valid_m.from().unwrap_or(valid_m.to());
                let to = valid_m.to();
                match self.board_widget.position.clone().play(valid_m.clone()) {
                    Ok(next_pos) => {
                        let sound = crate::audio::sound_for_move(&valid_m, &next_pos);
                        crate::audio::play_sound(sound);
                        self.board_widget.position = next_pos;
                        self.board_widget.last_move = Some((from, to));
                        self.engine_error = None;

                        let rep_count = self.record_current_position();
                        if let Some((outcome, status)) = self.evaluate_game_over(false, rep_count) {
                            crate::audio::play_sound(crate::audio::SoundType::GameEnd);
                            self.is_game_over = true;
                            self.board_widget.interactive = false;
                            self.game_outcome = Some(outcome);
                            self.game_status = status;
                        } else {
                            self.game_status = "Your turn! Make your move.".to_string();
                        }
                    }
                    Err(e) => {
                        let msg = format!("Failed to play engine move '{}': {}", uci_str, e);
                        self.engine_error = Some(msg.clone());
                        self.game_status = msg;
                    }
                }
            }
            Err(err) => {
                self.engine_error = Some(err.clone());
                self.game_status = format!("Engine error: {}", err);
            }
        }
    }

    pub fn retry_bot_move(&mut self) -> Option<String> {
        if self.is_game_over || self.is_player_turn() || self.is_engine_thinking {
            return None;
        }
        self.engine_error = None;
        self.is_engine_thinking = true;
        self.game_status = format!("{} is thinking...", self.current_bot().name);
        Some(self.board_widget.position_fen())
    }

    pub fn play_fallback_move(&mut self) {
        if self.is_game_over || self.is_player_turn() || self.is_engine_thinking {
            return;
        }
        let legal = self.board_widget.position.legal_moves();
        if legal.is_empty() {
            return;
        }
        let idx = pseudo_random_usize(self.game_generation) % legal.len();
        let chosen = &legal[idx];
        let uci_str = chosen.to_uci(CastlingMode::Standard).to_string();
        self.apply_engine_result(self.game_generation, Ok(uci_str));
    }

    pub fn apply_engine_move(&mut self, uci_str: &str) {
        self.apply_engine_result(self.game_generation, Ok(uci_str.to_string()));
    }

    pub fn view<'a>(&'a self) -> Element<'a, BotMessage> {
        let bot = self.current_bot();
        let is_white_turn = self.board_widget.position.turn() == Color::White;

        let bot_is_white = !self.player_is_white;
        let bot_active = !self.is_game_over && (is_white_turn == bot_is_white);
        let player_active = !self.is_game_over && (is_white_turn == self.player_is_white);

        let bot_result = self
            .game_outcome
            .as_ref()
            .and_then(|o| o.player_result(bot_is_white));
        let user_result = self
            .game_outcome
            .as_ref()
            .and_then(|o| o.player_result(self.player_is_white));

        let bot_status = if self.is_game_over {
            "Game Over"
        } else if self.is_engine_thinking {
            "Thinking..."
        } else if self.engine_error.is_some() && !self.is_player_turn() {
            "Error"
        } else {
            "Bot"
        };

        let user_status = if self.is_game_over {
            "Game Over"
        } else if self.is_engine_thinking {
            "Waiting..."
        } else if self.engine_error.is_some() && !self.is_player_turn() {
            "Waiting..."
        } else {
            "Your turn"
        };

        let bot_player = PlayerInfo::new(bot.name, bot_is_white)
            .with_rating(Some(bot.elo as i32))
            .with_title(bot.title)
            .with_status(bot_status)
            .with_active_turn(bot_active)
            .with_result(bot_result);

        let user_player = PlayerInfo::new("You", self.player_is_white)
            .with_status(user_status)
            .with_active_turn(player_active)
            .with_result(user_result);

        // board_widget.flipped is always !player_is_white, so user is always at the bottom.
        let (top_player, bottom_player) = (&bot_player, &user_player);

        let board_el = render_unified_board_column(
            top_player,
            bottom_player,
            &self.board_widget.position,
            &self.board_widget,
            BotMessage::Board,
            None,
        );

        let title = text("Play Human-Like Personality Bots")
            .font(fonts::UI_BOLD)
            .size(type_scale::TITLE)
            .color(palette::TEXT_PRIMARY)
            .wrapping(text::Wrapping::WordOrGlyph)
            .width(Length::Fill);

        let subtitle = text("Practice against realistic calibrated opponents from Beginner to Master.")
            .font(fonts::UI)
            .size(type_scale::BODY)
            .color(palette::TEXT_SECONDARY)
            .wrapping(text::Wrapping::WordOrGlyph)
            .width(Length::Fill);

        let make_color_btn = |choice: PlayerColorChoice, label: &'static str| {
            let is_sel = self.color_choice == choice;
            button(
                text(label)
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::BODY)
                    .wrapping(text::Wrapping::None),
            )
            .padding([spacing::XS, spacing::SM])
            .style(nav_button_style(is_sel))
            .on_press(BotMessage::SelectColor(choice))
        };

        let color_row = container(
            row![
                text("Play as:")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_SECONDARY)
                    .wrapping(text::Wrapping::None),
                make_color_btn(PlayerColorChoice::White, "White"),
                make_color_btn(PlayerColorChoice::Black, "Black"),
                make_color_btn(PlayerColorChoice::Random, "Random"),
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .clip(true);

        let mut bot_cards = column![].spacing(spacing::XS).width(Length::Fill);
        for (i, bot) in BOTS.iter().enumerate() {
            let is_selected = self.selected_bot_idx == i;
            let elo_badge = text(format!("{} Elo", bot.elo))
                .font(fonts::MONO_BOLD)
                .size(type_scale::CAPTION)
                .color(palette::ACCENT)
                .wrapping(text::Wrapping::None);

            let header = row![
                container(
                    row![
                        text(bot.name)
                            .font(fonts::UI_BOLD)
                            .size(type_scale::HEADING)
                            .color(if is_selected {
                                palette::TEXT_PRIMARY
                            } else {
                                palette::TEXT_SECONDARY
                            })
                            .wrapping(text::Wrapping::None),
                        Space::with_width(spacing::XS),
                        text(format!("({})", bot.title))
                            .font(fonts::UI)
                            .size(type_scale::CAPTION)
                            .color(palette::TEXT_MUTED)
                            .wrapping(text::Wrapping::None),
                    ]
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .clip(true),
                Space::with_width(spacing::XS),
                elo_badge,
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill);

            let card = button(
                column![
                    header,
                    Space::with_height(spacing::XXS),
                    text(bot.bio)
                        .font(fonts::UI)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_MUTED)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .width(Length::Fill),
                ]
                .spacing(spacing::XXS)
                .width(Length::Fill),
            )
            .padding(spacing::SM)
            .width(Length::Fill)
            .style(move |_theme, _status| {
                if is_selected {
                    button::Style {
                        background: Some(palette::BG_ELEVATED.into()),
                        border: border::rounded(spacing::RADIUS)
                            .color(palette::ACCENT)
                            .width(2.0),
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        background: Some(palette::BG_SURFACE.into()),
                        border: border::rounded(spacing::RADIUS)
                            .color(palette::BORDER_SUBTLE)
                            .width(1.0),
                        ..Default::default()
                    }
                }
            })
            .on_press(BotMessage::SelectBot(i));

            bot_cards = bot_cards.push(card);
        }

        let new_game_label = if self.is_game_over {
            "Rematch / New Game"
        } else {
            "New Game"
        };
        let new_game_content = row![
            icon_widget(icon_play(), 14.0, Some(palette::TEXT_PRIMARY)),
            text(new_game_label)
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let new_game_btn = button(new_game_content)
            .padding([spacing::SM, spacing::MD])
            .style(primary_button_style)
            .on_press(BotMessage::NewGame);

        let resign_btn = button(
            row![
                icon_widget(icon_flag(), 14.0, Some(palette::TEXT_PRIMARY)),
                text("Resign")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::BODY)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center),
        )
        .padding([spacing::SM, spacing::MD])
        .style(danger_button_style)
        .on_press(BotMessage::Resign);

        let status_color = if self.engine_error.is_some() {
            palette::DANGER
        } else if self.is_engine_thinking {
            palette::ACCENT
        } else {
            palette::TEXT_PRIMARY
        };

        let status_box = container(
            row![text(&self.game_status)
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::HEADING)
                .color(status_color)
                .wrapping(text::Wrapping::WordOrGlyph)
                .width(Length::Fill)]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .padding(spacing::SM)
        .width(Length::Fill)
        .clip(true)
        .style(card_container_style);

        let mut side_col = column![
            title,
            subtitle,
            Space::with_height(spacing::XS),
            color_row,
            Space::with_height(spacing::XS),
            bot_cards,
            Space::with_height(spacing::SM),
        ]
        .spacing(spacing::XS)
        .width(Length::Fill);

        if self.is_game_over {
            if let Some(ref outcome) = self.game_outcome {
                let tone = match user_result {
                    Some(PlayerResult::Win) => OutcomeTone::Win,
                    Some(PlayerResult::Loss) => OutcomeTone::Loss,
                    _ => OutcomeTone::Draw,
                };
                side_col = side_col.push(render_result_banner_with_tone(outcome, tone));
            }
            let rematch_row = row![new_game_btn, status_box]
                .spacing(spacing::SM)
                .align_y(Alignment::Center)
                .width(Length::Fill);
            side_col = side_col.push(rematch_row);
        } else {
            let action_row = row![new_game_btn, resign_btn, status_box]
                .spacing(spacing::SM)
                .align_y(Alignment::Center)
                .width(Length::Fill);
            side_col = side_col.push(action_row);
        }

        if self.pending_promotion.is_some() {
            let promo_btn = |role: Role, label: &'static str| {
                button(
                    text(label)
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::BODY)
                        .wrapping(text::Wrapping::None),
                )
                .padding([spacing::XS, spacing::SM])
                .style(primary_button_style)
                .on_press(BotMessage::SelectPromotion(role))
            };

            let promo_box = container(
                column![
                    text("Promote Pawn To:")
                        .font(fonts::UI_BOLD)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_PRIMARY),
                    row![
                        promo_btn(Role::Queen, "Queen"),
                        promo_btn(Role::Rook, "Rook"),
                        promo_btn(Role::Bishop, "Bishop"),
                        promo_btn(Role::Knight, "Knight"),
                        button(
                            text("Cancel")
                                .font(fonts::UI)
                                .size(type_scale::BODY)
                                .wrapping(text::Wrapping::None)
                        )
                        .padding([spacing::XS, spacing::SM])
                        .style(secondary_button_style)
                        .on_press(BotMessage::CancelPromotion),
                    ]
                    .spacing(spacing::XS),
                ]
                .spacing(spacing::XS)
                .width(Length::Fill),
            )
            .padding(spacing::SM)
            .width(Length::Fill)
            .clip(true)
            .style(card_container_style);

            side_col = side_col.push(Space::with_height(spacing::XS)).push(promo_box);
        }

        if self.engine_error.is_some() && !self.is_game_over && !self.is_player_turn() {
            let retry_btn = button(
                row![
                    icon_widget(icon_refresh_cw(), 14.0, Some(palette::TEXT_PRIMARY)),
                    text("Retry Engine")
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::BODY)
                        .wrapping(text::Wrapping::None),
                ]
                .spacing(spacing::XS)
                .align_y(Alignment::Center),
            )
            .padding([spacing::XS, spacing::SM])
            .style(danger_button_style)
            .on_press(BotMessage::RetryBotMove);

            let fallback_btn = button(
                text("Play Random Legal Move")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::BODY)
                    .wrapping(text::Wrapping::None),
            )
            .padding([spacing::XS, spacing::SM])
            .style(secondary_button_style)
            .on_press(BotMessage::PlayFallbackMove);

            let recovery_box = container(
                row![retry_btn, fallback_btn]
                    .spacing(spacing::SM)
                    .align_y(Alignment::Center),
            )
            .padding(spacing::SM)
            .width(Length::Fill)
            .clip(true)
            .style(card_container_style);

            side_col = side_col.push(Space::with_height(spacing::XS)).push(recovery_box);
        }

        let side_panel = container(side_col).width(Length::Fill).clip(true);

        let content = row![board_el, Space::with_width(spacing::LG), side_panel,]
            .padding(spacing::MD)
            .width(Length::Fill)
            .height(Length::Fill);

        container(content).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::fen::Fen;

    #[test]
    fn promotion_choice_applies_selected_piece() {
        let mut screen = BotPlayScreen::new();
        let fen: Fen = "k7/p3P3/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        screen.board_widget.position = fen.into_position(CastlingMode::Standard).unwrap();

        assert!(screen.handle_square_clicked(Square::E7).is_none());
        assert!(screen.handle_square_clicked(Square::E8).is_none());
        assert_eq!(screen.pending_promotion, Some((Square::E7, Square::E8)));

        let next_fen = screen.handle_promotion(Role::Knight);
        assert!(next_fen.is_some());
        assert!(screen.pending_promotion.is_none());
        let promoted = screen.board_widget.position.board().piece_at(Square::E8).unwrap();
        assert_eq!(promoted.role, Role::Knight);
    }

    #[test]
    fn stale_engine_move_is_dropped() {
        let mut screen = BotPlayScreen::new();
        let _ = screen.handle_square_clicked(Square::E2);
        let _ = screen.handle_square_clicked(Square::E4);
        assert!(screen.is_engine_thinking);
        let old_gen = screen.game_generation;

        // User starts a new game while bot is thinking
        screen.reset_game();
        assert_ne!(screen.game_generation, old_gen);
        assert!(!screen.is_engine_thinking);

        // Stale move arrives for old_gen
        screen.apply_engine_result(old_gen, Ok("e7e5".to_string()));
        assert!(screen.board_widget.position.board().piece_at(Square::E7).is_some());
        assert!(screen.board_widget.position.board().piece_at(Square::E5).is_none());
    }

    #[test]
    fn engine_error_does_not_say_your_turn_and_allows_fallback() {
        let mut screen = BotPlayScreen::new();
        let _ = screen.handle_square_clicked(Square::E2);
        let _ = screen.handle_square_clicked(Square::E4);
        let gen = screen.game_generation;

        screen.apply_engine_result(gen, Err("Stockfish missing".to_string()));
        assert!(!screen.is_engine_thinking);
        assert!(screen.engine_error.is_some());
        assert!(!screen.is_player_turn());
        assert!(!screen.game_status.contains("Your turn"));

        // Clicking board while bot failed is ignored
        assert!(screen.handle_square_clicked(Square::D2).is_none());

        // Fallback legal move recovers the game
        screen.play_fallback_move();
        assert!(screen.engine_error.is_none());
        assert!(screen.is_player_turn());
    }

    #[test]
    fn play_as_black_flips_board_and_requests_bot_first_move() {
        let mut screen = BotPlayScreen::new();
        screen.color_choice = PlayerColorChoice::Black;
        let fen = screen.reset_game();
        assert!(fen.is_some());
        assert!(!screen.player_is_white);
        assert!(screen.board_widget.flipped);
        assert!(screen.is_engine_thinking);
    }

    #[test]
    fn stalemate_and_insufficient_material_statuses_are_distinct() {
        let mut screen = BotPlayScreen::new();
        // White Qc7 -> Qb6 stalemates Black Ka8
        let fen: Fen = "k7/2Q5/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        screen.board_widget.position = fen.into_position(CastlingMode::Standard).unwrap();
        let _ = screen.handle_square_clicked(Square::C7);
        let res = screen.handle_square_clicked(Square::B6);
        assert!(res.is_none());
        assert!(screen.is_game_over);
        assert_eq!(screen.game_status, "Draw by stalemate.");

        // White captures last Black knight leaving K vs K -> insufficient material
        let mut screen2 = BotPlayScreen::new();
        let fen2: Fen = "4k3/8/8/8/8/8/4n3/4K3 w - - 0 1".parse().unwrap();
        screen2.board_widget.position = fen2.into_position(CastlingMode::Standard).unwrap();
        let _ = screen2.handle_square_clicked(Square::E1);
        let res2 = screen2.handle_square_clicked(Square::E2);
        assert!(res2.is_none());
        assert!(screen2.is_game_over);
        assert_eq!(screen2.game_status, "Draw by insufficient material.");
    }

    #[test]
    fn threefold_repetition_is_detected() {
        let mut screen = BotPlayScreen::new();
        let moves = [
            ("g1f3", "g8f6"),
            ("f3g1", "f6g8"),
            ("g1f3", "g8f6"),
            ("f3g1", "f6g8"),
        ];
        for (i, (w, b)) in moves.iter().enumerate() {
            let wm: shakmaty::uci::UciMove = w.parse().unwrap();
            let valid_w = wm.to_move(&screen.board_widget.position).unwrap();
            let next_fen = screen.handle_user_move(valid_w.from().unwrap(), valid_w.to());
            assert!(next_fen.is_some());

            let gen = screen.game_generation;
            screen.apply_engine_result(gen, Ok((*b).to_string()));
            if i == 3 {
                assert!(screen.is_game_over);
                assert_eq!(screen.game_status, "Draw by threefold repetition.");
            }
        }
    }

    #[test]
    fn resign_ends_game_disables_board_and_sets_outcome() {
        let mut screen = BotPlayScreen::new();
        assert!(screen.board_widget.interactive);
        assert!(screen.game_outcome.is_none());

        screen.resign();
        assert!(screen.is_game_over);
        assert!(!screen.board_widget.interactive);
        let outcome = screen.game_outcome.as_ref().unwrap();
        assert_eq!(outcome.result, "0-1");
        assert_eq!(outcome.reason, "You resigned");

        // Subsequent board clicks are ignored
        assert!(screen.handle_square_clicked(Square::E2).is_none());

        // Reset restores board interactivity and clears outcome
        screen.reset_game();
        assert!(!screen.is_game_over);
        assert!(screen.board_widget.interactive);
        assert!(screen.game_outcome.is_none());
    }
}
