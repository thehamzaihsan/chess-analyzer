use crate::ui::board::{board_panel, BoardMessage, BoardWidget};
use crate::ui::theme::{
    card_container_style, fonts, icon_play, icon_widget, palette, primary_button_style,
    spacing, type_scale,
};
use iced::border;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};
use shakmaty::{Chess, Position, Square};

#[derive(Debug, Clone)]
pub enum BotMessage {
    Board(BoardMessage),
    SelectBot(usize),
    NewGame,
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

pub struct BotPlayScreen {
    pub selected_bot_idx: usize,
    pub board_widget: BoardWidget,
    pub is_engine_thinking: bool,
    pub game_status: String,
    pub player_is_white: bool,
}

impl BotPlayScreen {
    pub fn new() -> Self {
        Self {
            selected_bot_idx: 1, // Default: Club Tactician (1600)
            board_widget: BoardWidget::new(Chess::default()),
            is_engine_thinking: false,
            game_status: "Your turn! Make a move to start playing.".to_string(),
            player_is_white: true,
        }
    }

    pub fn current_bot(&self) -> &'static BotProfile {
        &BOTS[self.selected_bot_idx]
    }

    pub fn reset_game(&mut self) {
        self.board_widget.position = Chess::default();
        self.board_widget.last_move = None;
        self.board_widget.selected_square = None;
        self.is_engine_thinking = false;
        self.game_status = "New game started! White to move.".to_string();
    }

    pub fn handle_square_clicked(&mut self, sq: Square) -> Option<String> {
        if self.is_engine_thinking {
            return None;
        }

        if let Some(selected) = self.board_widget.selected_square {
            if selected == sq {
                self.board_widget.selected_square = None;
                return None;
            }
            if let Some(fen) = self.handle_user_move(selected, sq) {
                return Some(fen);
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

    pub fn handle_user_move(&mut self, from: Square, to: Square) -> Option<String> {
        if self.is_engine_thinking {
            return None;
        }

        let moves = self.board_widget.position.legal_moves();
        for m in moves {
            if m.from() == Some(from) && m.to() == to {
                if let Ok(next_pos) = self.board_widget.position.clone().play(m.clone()) {
                    self.board_widget.position = next_pos;
                    self.board_widget.last_move = Some((from, to));
                    self.board_widget.selected_square = None;

                    if self.board_widget.position.is_checkmate() {
                        self.game_status = "Checkmate! You won!".to_string();
                        return None;
                    } else if self.board_widget.position.is_game_over() {
                        self.game_status = "Game drawn.".to_string();
                        return None;
                    }

                    self.is_engine_thinking = true;
                    self.game_status = format!("{} is thinking...", self.current_bot().name);
                    return Some(self.board_widget.position_fen());
                }
            }
        }

        None
    }

    pub fn handle_engine_move(&mut self, uci_str: &str) {
        self.is_engine_thinking = false;
        if let Ok(m) = uci_str.parse::<shakmaty::uci::UciMove>() {
            if let Ok(valid_m) = m.to_move(&self.board_widget.position) {
                let from = valid_m.from().unwrap_or(Square::E7);
                let to = valid_m.to();
                if let Ok(next_pos) = self.board_widget.position.clone().play(valid_m) {
                    self.board_widget.position = next_pos;
                    self.board_widget.last_move = Some((from, to));

                    if self.board_widget.position.is_checkmate() {
                        self.game_status = format!("Checkmate! {} won.", self.current_bot().name);
                    } else if self.board_widget.position.is_game_over() {
                        self.game_status = "Game drawn.".to_string();
                    } else {
                        self.game_status = "Your turn! Make your move.".to_string();
                    }
                }
            }
        }
    }

    pub fn apply_engine_move(&mut self, uci_str: &str) {
        self.handle_engine_move(uci_str);
    }

    pub fn view<'a>(&'a self) -> Element<'a, BotMessage> {
        let board_size = 520.0;
        let board_el = self.board_widget.view(board_size).map(BotMessage::Board);
        let board_el = board_panel(&self.board_widget.position, board_el);

        let title = text("Play Human-Like Personality Bots")
            .font(fonts::UI_BOLD)
            .size(type_scale::TITLE)
            .color(palette::TEXT_PRIMARY);

        let subtitle = text("Practice against realistic calibrated opponents from Beginner to Master.")
            .font(fonts::UI)
            .size(type_scale::BODY)
            .color(palette::TEXT_SECONDARY);

        let mut bot_cards = column![].spacing(spacing::XS);
        for (i, bot) in BOTS.iter().enumerate() {
            let is_selected = self.selected_bot_idx == i;
            let elo_badge = text(format!("{} Elo", bot.elo))
                .font(fonts::MONO_BOLD)
                .size(type_scale::CAPTION)
                .color(palette::ACCENT);

            let header = row![
                text(bot.name)
                    .font(fonts::UI_BOLD)
                    .size(type_scale::HEADING)
                    .color(if is_selected { palette::TEXT_PRIMARY } else { palette::TEXT_SECONDARY }),
                Space::with_width(spacing::XS),
                text(format!("({})", bot.title))
                    .font(fonts::UI)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_MUTED),
                Space::with_width(Length::Fill),
                elo_badge,
            ]
            .align_y(Alignment::Center);

            let card = button(
                column![
                    header,
                    Space::with_height(spacing::XXS),
                    text(bot.bio)
                        .font(fonts::UI)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_MUTED),
                ]
                .spacing(spacing::XXS)
            )
            .padding(spacing::SM)
            .width(Length::Fill)
            .style(move |_theme, _status| {
                if is_selected {
                    button::Style {
                        background: Some(palette::BG_ELEVATED.into()),
                        border: border::rounded(spacing::RADIUS).color(palette::ACCENT).width(2.0),
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        background: Some(palette::BG_SURFACE.into()),
                        border: border::rounded(spacing::RADIUS).color(palette::BORDER_SUBTLE).width(1.0),
                        ..Default::default()
                    }
                }
            })
            .on_press(BotMessage::SelectBot(i));

            bot_cards = bot_cards.push(card);
        }

        let new_game_content = row![
            icon_widget(icon_play(), 14.0, Some(palette::TEXT_PRIMARY)),
            text("Start New Game").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let new_game_btn = button(new_game_content)
            .padding([spacing::SM, spacing::MD])
            .style(primary_button_style)
            .on_press(BotMessage::NewGame);

        let status_box = container(
            row![
                text(&self.game_status)
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::HEADING)
                    .color(if self.is_engine_thinking { palette::ACCENT } else { palette::TEXT_PRIMARY }),
            ]
            .align_y(Alignment::Center)
        )
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(card_container_style);

        let side_col = column![
            title,
            subtitle,
            Space::with_height(spacing::XS),
            bot_cards,
            Space::with_height(spacing::SM),
            row![new_game_btn, Space::with_width(spacing::SM), status_box].align_y(Alignment::Center),
        ]
        .spacing(spacing::XS);

        let content = row![
            board_el,
            Space::with_width(spacing::LG),
            side_col,
        ]
        .padding(spacing::MD)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content).into()
    }
}
