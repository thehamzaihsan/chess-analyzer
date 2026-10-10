use crate::db::Puzzle;
use crate::ui::board::{board_panel, BoardMessage, BoardWidget};
use crate::ui::theme::{
    card_container_style, fonts, icon_check, icon_chevron_right, icon_sparkles, icon_widget,
    icon_x, nav_button_style, palette, primary_button_style, secondary_button_style, spacing,
    type_scale,
};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};
use shakmaty::{Chess, Position, Square};

#[derive(Debug, Clone)]
pub enum PuzzleMessage {
    Board(BoardMessage),
    NextPuzzle,
    MineBlunderPuzzles,
    SelectFilter(Option<String>), // None = all, Some("blunder"), Some("lichess")
}

#[derive(Debug, Clone)]
pub struct PuzzleScreen {
    pub current_puzzle: Option<Puzzle>,
    pub board_widget: BoardWidget,
    pub puzzle_status: String,
    pub is_solved: bool,
    pub filter: Option<String>,
    pub puzzle_moves: Vec<String>,
    pub current_move_step: usize,
}

impl PuzzleScreen {
    pub fn new() -> Self {
        Self {
            current_puzzle: None,
            board_widget: BoardWidget::new(Chess::default()),
            puzzle_status: "Load a puzzle to start training".to_string(),
            is_solved: false,
            filter: None,
            puzzle_moves: Vec::new(),
            current_move_step: 0,
        }
    }

    pub fn load_puzzle(&mut self, puzzle: Puzzle) {
        if let Ok(fen) = puzzle.fen.parse::<shakmaty::fen::Fen>() {
            if let Ok(pos) = fen.into_position::<Chess>(shakmaty::CastlingMode::Standard) {
                let is_black = pos.turn() == shakmaty::Color::Black;
                self.board_widget = BoardWidget::new(pos);
                self.board_widget.flipped = is_black;
            }
        }

        self.puzzle_moves = puzzle
            .moves
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        self.current_move_step = 0;
        self.is_solved = false;
        self.puzzle_status = format!(
            "Find the best move! (Rating: {}, Source: {})",
            puzzle.rating, puzzle.source
        );
        self.current_puzzle = Some(puzzle);
    }

    pub fn handle_square_clicked(&mut self, sq: Square) {
        if self.is_solved || self.current_puzzle.is_none() {
            return;
        }

        if let Some(selected) = self.board_widget.selected_square {
            let moves = self.board_widget.position.legal_moves();
            let mut found_move = None;
            for m in moves {
                if m.from() == Some(selected) && m.to() == sq {
                    found_move = Some(m);
                    break;
                }
            }

            if let Some(m) = found_move {
                let uci_str = m.to_uci(shakmaty::CastlingMode::Standard).to_string();
                let expected = self.puzzle_moves.get(self.current_move_step);

                if expected == Some(&uci_str) {
                    if let Ok(next_pos) = self.board_widget.position.clone().play(m) {
                        self.board_widget.position = next_pos;
                        self.board_widget.last_move = Some((selected, sq));
                        self.board_widget.selected_square = None;
                        self.current_move_step += 1;

                        if self.current_move_step >= self.puzzle_moves.len() {
                            self.is_solved = true;
                            self.puzzle_status = "Correct! Puzzle solved!".to_string();
                        } else {
                            if let Some(opp_uci) = self.puzzle_moves.get(self.current_move_step) {
                                if let Ok(opp_m) = opp_uci.parse::<shakmaty::uci::UciMove>() {
                                    if let Ok(valid_opp) = opp_m.to_move(&self.board_widget.position) {
                                        let opp_from = valid_opp.from().unwrap_or(sq);
                                        let opp_to = valid_opp.to();
                                        if let Ok(next_opp_pos) = self.board_widget.position.clone().play(valid_opp) {
                                            self.board_widget.position = next_opp_pos;
                                            self.board_widget.last_move = Some((opp_from, opp_to));
                                            self.current_move_step += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    self.puzzle_status = "Incorrect move. Try again!".to_string();
                    self.board_widget.selected_square = None;
                }
            } else {
                if self.board_widget.position.board().color_at(sq) == Some(self.board_widget.position.turn()) {
                    self.board_widget.selected_square = Some(sq);
                } else {
                    self.board_widget.selected_square = None;
                }
            }
        } else {
            if self.board_widget.position.board().color_at(sq) == Some(self.board_widget.position.turn()) {
                self.board_widget.selected_square = Some(sq);
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, PuzzleMessage> {
        let board_size = 520.0;
        let board_el = self.board_widget.view(board_size).map(PuzzleMessage::Board);
        let board_el = board_panel(&self.board_widget.position, board_el);

        let title = text("Tactics & Puzzle Trainer")
            .font(fonts::UI_BOLD)
            .size(type_scale::TITLE)
            .color(palette::TEXT_PRIMARY);

        let make_filter_btn = |filter_opt: Option<String>, label: &'static str| {
            let is_sel = self.filter == filter_opt;
            button(text(label).font(fonts::UI_SEMIBOLD).size(type_scale::BODY))
                .padding([spacing::XS, spacing::SM])
                .style(nav_button_style(is_sel))
                .on_press(PuzzleMessage::SelectFilter(filter_opt))
        };

        let filter_row = row![
            text("Filter:").font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_SECONDARY),
            make_filter_btn(None, "All Puzzles"),
            make_filter_btn(Some("blunder".to_string()), "My Blunders"),
            make_filter_btn(Some("lichess".to_string()), "Lichess Puzzles"),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let next_btn_content = row![
            text("Next Puzzle").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
            icon_widget(icon_chevron_right(), 14.0, Some(palette::TEXT_PRIMARY)),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let next_btn = button(next_btn_content)
            .padding([spacing::SM, spacing::MD])
            .style(primary_button_style)
            .on_press(PuzzleMessage::NextPuzzle);

        let mine_content = row![
            icon_widget(icon_sparkles(), 14.0, Some(palette::TEXT_PRIMARY)),
            text("Mine Puzzles from Blunders").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let mine_btn = button(mine_content)
            .padding([spacing::SM, spacing::MD])
            .style(secondary_button_style)
            .on_press(PuzzleMessage::MineBlunderPuzzles);

        let action_row = row![next_btn, mine_btn].spacing(spacing::SM);

        let (status_icon, status_color) = if self.is_solved {
            (Some(icon_check()), palette::SUCCESS)
        } else if self.puzzle_status.starts_with("Incorrect") {
            (Some(icon_x()), palette::DANGER)
        } else {
            (None, palette::TEXT_SECONDARY)
        };

        let status_row = if let Some(icon) = status_icon {
            row![
                icon_widget(icon, 16.0, Some(status_color)),
                Space::with_width(spacing::XS),
                text(&self.puzzle_status).font(fonts::UI_SEMIBOLD).size(type_scale::HEADING).color(status_color),
            ]
            .align_y(Alignment::Center)
        } else {
            row![text(&self.puzzle_status).font(fonts::UI_SEMIBOLD).size(type_scale::HEADING).color(status_color)]
                .align_y(Alignment::Center)
        };

        let status_box = container(status_row)
            .padding(spacing::SM)
            .style(card_container_style);

        let side_col = column![
            title,
            Space::with_height(spacing::XS),
            filter_row,
            Space::with_height(spacing::SM),
            action_row,
            Space::with_height(spacing::SM),
            status_box,
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
