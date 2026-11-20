use crate::openings::identify_opening;
use crate::ui::board::{BoardMessage, BoardWidget};
use crate::ui::game_layout::{
    compute_board_layout, render_board_workspace, render_result_banner,
    render_sized_board_column, GameOutcome, PlayerInfo,
};
use crate::ui::theme::{
    card_container_style, fonts, icon_chevron_left, icon_chevron_right, icon_play,
    icon_refresh_cw, icon_rotate_ccw, icon_widget, nav_button_style, palette,
    primary_button_style, secondary_button_style, spacing, truncate, type_scale, with_tooltip,
};
use iced::widget::{button, column, container, row, scrollable, text, tooltip, Space};
use iced::{Alignment, Element, Length};
use shakmaty::{Chess, Color, Move, Position, Role, Square};

#[derive(Debug, Clone)]
pub enum ExplorerMessage {
    Board(BoardMessage),
    PlayMove(String), // SAN
    ResetBoard,
    UndoMove,
    RedoMove,
    FlipBoard,
    JumpToMove(usize),
    SelectPromotion(Role),
    CancelPromotion,
}

#[derive(Debug, Clone)]
pub struct MoveStats {
    pub san: String,
    pub count: usize,
    pub white_wins: usize,
    pub draws: usize,
    pub black_wins: usize,
}

#[derive(Debug, Clone)]
pub struct ExplorerMove {
    pub pos_before: Chess,
    pub pos_after: Chess,
    pub san: String,
    pub last_move: Option<(Square, Square)>,
}

pub struct ExplorerScreen {
    pub board_widget: BoardWidget,
    pub move_history: Vec<ExplorerMove>,
    pub current_ply: usize,
    pub pending_promotion: Option<(Square, Square)>,
    pub current_opening: String,
    pub current_eco: String,
}

impl ExplorerScreen {
    pub fn new() -> Self {
        Self {
            board_widget: BoardWidget::new(Chess::default()),
            move_history: Vec::new(),
            current_ply: 0,
            pending_promotion: None,
            current_opening: "Starting Position".to_string(),
            current_eco: "".to_string(),
        }
    }

    pub fn update_opening(&mut self) {
        if self.current_ply == 0 {
            self.current_opening = "Starting Position".to_string();
            self.current_eco = "".to_string();
            return;
        }
        let sans: Vec<String> = self.move_history[..self.current_ply]
            .iter()
            .map(|m| m.san.clone())
            .collect();
        let m = identify_opening(&sans);
        self.current_opening = if m.name.is_empty() {
            "Custom Position".to_string()
        } else {
            m.name
        };
        self.current_eco = m.eco;
    }

    pub fn handle_square_clicked(&mut self, sq: Square) {
        if self.pending_promotion.is_some() {
            self.pending_promotion = None;
            if self.board_widget.position.board().color_at(sq)
                == Some(self.board_widget.position.turn())
            {
                self.board_widget.selected_square = Some(sq);
            } else {
                self.board_widget.selected_square = None;
            }
            return;
        }

        if let Some(selected) = self.board_widget.selected_square {
            if selected == sq {
                self.board_widget.selected_square = None;
                return;
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
                return;
            }

            if let Some(m) = matching.into_iter().next() {
                self.play_chess_move(m);
            } else if self.board_widget.position.board().color_at(sq)
                == Some(self.board_widget.position.turn())
            {
                self.board_widget.selected_square = Some(sq);
            } else {
                self.board_widget.selected_square = None;
            }
        } else if self.board_widget.position.board().color_at(sq)
            == Some(self.board_widget.position.turn())
        {
            self.board_widget.selected_square = Some(sq);
        }
    }

    pub fn handle_promotion(&mut self, role: Role) {
        let Some((from, to)) = self.pending_promotion.take() else {
            return;
        };

        if let Some(m) = self
            .board_widget
            .position
            .legal_moves()
            .into_iter()
            .find(|m| m.from() == Some(from) && m.to() == to && m.promotion() == Some(role))
        {
            self.play_chess_move(m);
        }
    }

    pub fn play_chess_move(&mut self, m: Move) {
        let prev_pos = self.board_widget.position.clone();
        let san = shakmaty::san::SanPlus::from_move(prev_pos.clone(), m.clone()).to_string();
        let last_move = m.from().map(|f| (f, m.to()));

        if let Ok(next_pos) = prev_pos.clone().play(m.clone()) {
            let sound = crate::audio::sound_for_move(&m, &next_pos);
            crate::audio::play_sound(sound);

            // Playing a new move after undoing drops the future redo branch.
            if self.current_ply < self.move_history.len() {
                self.move_history.truncate(self.current_ply);
            }

            self.move_history.push(ExplorerMove {
                pos_before: prev_pos,
                pos_after: next_pos.clone(),
                san,
                last_move,
            });
            self.current_ply = self.move_history.len();
            self.board_widget.position = next_pos;
            self.board_widget.last_move = last_move;
            self.board_widget.selected_square = None;
            self.pending_promotion = None;
            self.update_opening();
        }
    }

    pub fn play_san(&mut self, san_str: &str) {
        let moves = self.board_widget.position.legal_moves();
        for m in moves {
            let s = shakmaty::san::SanPlus::from_move(
                self.board_widget.position.clone(),
                m.clone(),
            )
            .to_string();
            if s == san_str
                || s.trim_end_matches('+').trim_end_matches('#')
                    == san_str.trim_end_matches('+').trim_end_matches('#')
            {
                self.play_chess_move(m);
                return;
            }
        }
    }

    pub fn jump_to_move(&mut self, ply: usize) {
        let target = ply.min(self.move_history.len());
        let prev_ply = self.current_ply;
        self.pending_promotion = None;
        self.board_widget.selected_square = None;

        if target == 0 {
            self.board_widget.position = Chess::default();
            self.board_widget.last_move = None;
        } else if let Some(entry) = self.move_history.get(target - 1) {
            self.board_widget.position = entry.pos_after.clone();
            self.board_widget.last_move = entry.last_move;
        }

        self.current_ply = target;
        if target != prev_ply {
            crate::audio::play_sound(crate::audio::SoundType::Move);
        }
        self.update_opening();
    }

    pub fn undo(&mut self) {
        if self.current_ply > 0 {
            self.jump_to_move(self.current_ply - 1);
        }
    }

    pub fn redo(&mut self) {
        if self.current_ply < self.move_history.len() {
            self.jump_to_move(self.current_ply + 1);
        }
    }

    pub fn reset(&mut self) {
        crate::audio::play_sound(crate::audio::SoundType::GameStart);
        self.board_widget.position = Chess::default();
        self.board_widget.last_move = None;
        self.board_widget.selected_square = None;
        self.move_history.clear();
        self.current_ply = 0;
        self.pending_promotion = None;
        self.update_opening();
    }

    pub fn terminal_outcome(&self) -> Option<GameOutcome> {
        let pos = &self.board_widget.position;
        if pos.is_checkmate() {
            if pos.turn() == Color::Black {
                Some(GameOutcome::new("1-0", "Checkmate — White wins"))
            } else {
                Some(GameOutcome::new("0-1", "Checkmate — Black wins"))
            }
        } else if pos.is_stalemate() {
            Some(GameOutcome::new("1/2-1/2", "Draw by stalemate"))
        } else if pos.is_insufficient_material() {
            Some(GameOutcome::new("1/2-1/2", "Draw by insufficient material"))
        } else if pos.is_game_over() {
            Some(GameOutcome::new("1/2-1/2", "Game over"))
        } else {
            None
        }
    }

    pub fn view<'a>(&'a self, avail_size: iced::Size) -> Element<'a, ExplorerMessage> {
        let metrics = compute_board_layout(avail_size, false);
            let is_white_turn = self.board_widget.position.turn() == Color::White;
            let terminal = self.terminal_outcome();

            let black_player = PlayerInfo::new("Black", false)
                .with_active_turn(!is_white_turn && terminal.is_none())
                .with_result(terminal.as_ref().and_then(|o| o.player_result(false)));
            let white_player = PlayerInfo::new("White", true)
                .with_active_turn(is_white_turn && terminal.is_none())
                .with_result(terminal.as_ref().and_then(|o| o.player_result(true)));

            let (top_player, bottom_player) = if self.board_widget.flipped {
                (&white_player, &black_player)
            } else {
                (&black_player, &white_player)
            };

            let board_el = render_sized_board_column(
                top_player,
                bottom_player,
                &self.board_widget.position,
                &self.board_widget,
                ExplorerMessage::Board,
                None,
                metrics.board_side,
            );

            let eco_text = if self.current_eco.is_empty() {
                String::new()
            } else {
                format!("[{}] ", self.current_eco)
            };
            let opening_full = format!("{}{}", eco_text, self.current_opening);
            let opening_short = truncate(&opening_full, 48);
            let opening_txt = text(opening_short.clone())
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::HEADING)
                .color(palette::TEXT_PRIMARY)
                .wrapping(text::Wrapping::WordOrGlyph)
                .width(Length::Fill);
            let opening_el: Element<'a, ExplorerMessage> = if opening_short != opening_full {
                with_tooltip(opening_txt, opening_full, tooltip::Position::Bottom)
            } else {
                opening_txt.into()
            };

            let op_card = container(
                column![
                    text("Identified Opening")
                        .font(fonts::UI)
                        .size(type_scale::CAPTION)
                        .color(palette::TEXT_MUTED)
                        .wrapping(text::Wrapping::None),
                    opening_el,
                ]
                .spacing(spacing::XXS)
                .width(Length::Fill),
            )
            .padding(spacing::SM)
            .width(Length::Fill)
            .clip(true)
            .style(card_container_style);

            let can_undo = self.current_ply > 0;
            let can_redo = self.current_ply < self.move_history.len();

            let make_ctrl_btn = |content: Element<'a, ExplorerMessage>,
                                 on_press: Option<ExplorerMessage>| {
                button(container(content).width(Length::Fill).align_x(Alignment::Center))
                    .width(Length::FillPortion(1))
                    .padding([spacing::XS, spacing::XS])
                    .style(secondary_button_style)
                    .on_press_maybe(on_press)
            };

            let undo_content = row![
                icon_widget(icon_chevron_left(), 13.0, Some(palette::TEXT_PRIMARY)),
                text("Undo")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let redo_content = row![
                text("Redo")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .wrapping(text::Wrapping::None),
                icon_widget(icon_chevron_right(), 13.0, Some(palette::TEXT_PRIMARY)),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let reset_content = row![
                icon_widget(icon_refresh_cw(), 13.0, Some(palette::TEXT_PRIMARY)),
                text("Reset")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let flip_content = row![
                icon_widget(icon_rotate_ccw(), 13.0, Some(palette::TEXT_PRIMARY)),
                text("Flip")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let controls_row = container(
                row![
                    make_ctrl_btn(
                        undo_content.into(),
                        can_undo.then_some(ExplorerMessage::UndoMove),
                    ),
                    make_ctrl_btn(
                        redo_content.into(),
                        can_redo.then_some(ExplorerMessage::RedoMove),
                    ),
                    make_ctrl_btn(reset_content.into(), Some(ExplorerMessage::ResetBoard)),
                    make_ctrl_btn(flip_content.into(), Some(ExplorerMessage::FlipBoard)),
                ]
                .spacing(spacing::XS)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .clip(true);

            // Clickable Move History
            let history_header = row![
                text("Move History")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::HEADING)
                    .color(palette::TEXT_SECONDARY)
                    .wrapping(text::Wrapping::None),
                Space::with_width(Length::Fill),
                button(
                    text("Start")
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::CAPTION)
                        .wrapping(text::Wrapping::None)
                )
                .padding([2.0, 8.0])
                .style(nav_button_style(self.current_ply == 0))
                .on_press(ExplorerMessage::JumpToMove(0)),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill);

            let mut history_rows = column![].spacing(spacing::XXS).width(Length::Fill);
            if self.move_history.is_empty() {
                history_rows = history_rows.push(
                    text("No moves yet. Click pieces or candidate continuations to explore.")
                        .font(fonts::UI)
                        .size(type_scale::CAPTION)
                        .color(palette::TEXT_MUTED)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .width(Length::Fill),
                );
            } else {
                for (pair_idx, pair) in self.move_history.chunks(2).enumerate() {
                    let move_num = pair_idx + 1;
                    let w_ply = pair_idx * 2 + 1;
                    let w_san = &pair[0].san;
                    let w_active = self.current_ply == w_ply;
                    let w_future = w_ply > self.current_ply;

                    let w_btn = button(
                        text(w_san)
                            .font(fonts::MONO_BOLD)
                            .size(type_scale::BODY)
                            .wrapping(text::Wrapping::None)
                            .color(if w_future {
                                palette::TEXT_MUTED
                            } else {
                                palette::TEXT_PRIMARY
                            }),
                    )
                    .padding([2.0, 8.0])
                    .style(nav_button_style(w_active))
                    .on_press(ExplorerMessage::JumpToMove(w_ply));

                    let mut move_row = row![
                        text(format!("{}.", move_num))
                            .font(fonts::MONO)
                            .size(type_scale::BODY)
                            .color(palette::TEXT_MUTED)
                            .wrapping(text::Wrapping::None)
                            .width(Length::Fixed(32.0)),
                        container(w_btn).width(Length::FillPortion(1)).clip(true),
                    ]
                    .spacing(spacing::XS)
                    .align_y(Alignment::Center)
                    .width(Length::Fill);

                    if let Some(black_entry) = pair.get(1) {
                        let b_ply = pair_idx * 2 + 2;
                        let b_active = self.current_ply == b_ply;
                        let b_future = b_ply > self.current_ply;

                        let b_btn = button(
                            text(&black_entry.san)
                                .font(fonts::MONO_BOLD)
                                .size(type_scale::BODY)
                                .wrapping(text::Wrapping::None)
                                .color(if b_future {
                                    palette::TEXT_MUTED
                                } else {
                                    palette::TEXT_PRIMARY
                                }),
                        )
                        .padding([2.0, 8.0])
                        .style(nav_button_style(b_active))
                        .on_press(ExplorerMessage::JumpToMove(b_ply));

                        move_row =
                            move_row.push(container(b_btn).width(Length::FillPortion(1)).clip(true));
                    } else {
                        move_row = move_row.push(Space::with_width(Length::FillPortion(1)));
                    }

                    history_rows = history_rows.push(move_row);
                }
            }

            let history_card = container(scrollable(history_rows).height(Length::Fill))
                .padding(spacing::SM)
                .width(Length::Fill)
                .height(Length::FillPortion(2))
                .clip(true)
                .style(card_container_style);

            // Legal candidate moves from current position
            let legal_moves = self.board_widget.position.legal_moves();
            let mut candidate_moves_col = column![row![
                text("Move")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_MUTED)
                    .wrapping(text::Wrapping::None)
                    .width(Length::Fixed(68.0)),
                text("Status")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_MUTED)
                    .wrapping(text::Wrapping::None)
                    .width(Length::FillPortion(1)),
                text("Action")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_MUTED)
                    .wrapping(text::Wrapping::None)
                    .width(Length::Fixed(72.0)),
            ]
            .padding([spacing::XXS, spacing::XS])
            .width(Length::Fill)]
            .spacing(spacing::XXS)
            .width(Length::Fill);

            for m in legal_moves.iter().take(24) {
                let san_str = shakmaty::san::SanPlus::from_move(
                    self.board_widget.position.clone(),
                    m.clone(),
                )
                .to_string();
                let san_btn_str = san_str.clone();

                let play_content = row![
                    icon_widget(icon_play(), 12.0, Some(palette::TEXT_PRIMARY)),
                    text("Play")
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::CAPTION)
                        .wrapping(text::Wrapping::None),
                ]
                .spacing(spacing::XXS)
                .align_y(Alignment::Center);

                let row_item = container(
                    row![
                        container(
                            text(san_str)
                                .font(fonts::MONO_BOLD)
                                .size(type_scale::BODY)
                                .color(palette::TEXT_PRIMARY)
                                .wrapping(text::Wrapping::None)
                        )
                        .width(Length::Fixed(68.0))
                        .clip(true),
                        container(
                            text("Legal Move")
                                .font(fonts::UI)
                                .size(type_scale::CAPTION)
                                .color(palette::SUCCESS)
                                .wrapping(text::Wrapping::None)
                        )
                        .width(Length::FillPortion(1))
                        .clip(true),
                        button(play_content)
                            .width(Length::Fixed(72.0))
                            .padding([spacing::XXS, spacing::XS])
                            .style(secondary_button_style)
                            .on_press(ExplorerMessage::PlayMove(san_btn_str)),
                    ]
                    .spacing(spacing::XS)
                    .align_y(Alignment::Center)
                    .padding([spacing::XXS, spacing::XS])
                    .width(Length::Fill),
                )
                .width(Length::Fill)
                .clip(true)
                .style(card_container_style);

                candidate_moves_col = candidate_moves_col.push(row_item);
            }

            let moves_card = container(scrollable(candidate_moves_col).height(Length::Fill))
                .padding(spacing::XS)
                .width(Length::Fill)
                .height(Length::FillPortion(3))
                .clip(true)
                .style(card_container_style);

            let mut sidebar = column![op_card, controls_row]
                .spacing(spacing::XS)
                .width(Length::Fill)
                .height(Length::Fill);

            if let Some(ref out) = terminal {
                sidebar = sidebar.push(render_result_banner(out));
            }

            sidebar = sidebar
                .push(history_header)
                .push(history_card)
                .push(
                    text("Candidate Continuations")
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::HEADING)
                        .color(palette::TEXT_SECONDARY)
                        .wrapping(text::Wrapping::None),
                )
                .push(moves_card);

            if self.pending_promotion.is_some() {
                let promo_btn = |role: Role, label: &'static str| {
                    button(
                        text(label)
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::CAPTION)
                            .wrapping(text::Wrapping::None),
                    )
                    .padding([spacing::XXS, spacing::XS])
                    .style(primary_button_style)
                    .on_press(ExplorerMessage::SelectPromotion(role))
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
                                    .size(type_scale::CAPTION)
                                    .wrapping(text::Wrapping::None)
                            )
                            .padding([spacing::XXS, spacing::XS])
                            .style(secondary_button_style)
                            .on_press(ExplorerMessage::CancelPromotion),
                        ]
                        .spacing(spacing::XXS),
                    ]
                    .spacing(spacing::XS)
                    .width(Length::Fill),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .clip(true)
                .style(card_container_style);

                sidebar = sidebar.push(promo_box);
            }

            render_board_workspace(board_el, sidebar.into(), metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::fen::Fen;
    use shakmaty::CastlingMode;

    #[test]
    fn undo_redo_and_jump_to_move_navigate_history_cleanly() {
        let mut screen = ExplorerScreen::new();
        screen.play_san("e4");
        screen.play_san("e5");
        screen.play_san("Nf3");
        assert_eq!(screen.current_ply, 3);
        assert_eq!(screen.move_history.len(), 3);

        // Undo to ply 2
        screen.undo();
        assert_eq!(screen.current_ply, 2);
        assert_eq!(screen.board_widget.position.turn(), Color::White);
        assert_eq!(screen.board_widget.last_move, Some((Square::E7, Square::E5)));

        // Redo back to ply 3
        screen.redo();
        assert_eq!(screen.current_ply, 3);
        assert_eq!(screen.board_widget.position.turn(), Color::Black);
        assert_eq!(screen.board_widget.last_move, Some((Square::G1, Square::F3)));

        // Jump directly back to ply 1
        screen.jump_to_move(1);
        assert_eq!(screen.current_ply, 1);
        assert_eq!(screen.board_widget.last_move, Some((Square::E2, Square::E4)));

        // Playing a new move truncates the future redo branch
        screen.play_san("c5");
        assert_eq!(screen.current_ply, 2);
        assert_eq!(screen.move_history.len(), 2);
        assert_eq!(screen.move_history[1].san, "c5");
    }

    #[test]
    fn explorer_promotion_choice_promotes_selected_piece() {
        let mut screen = ExplorerScreen::new();
        let fen: Fen = "k7/4P3/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        screen.board_widget.position = fen.into_position(CastlingMode::Standard).unwrap();

        screen.handle_square_clicked(Square::E7);
        screen.handle_square_clicked(Square::E8);
        assert_eq!(screen.pending_promotion, Some((Square::E7, Square::E8)));

        screen.handle_promotion(Role::Bishop);
        assert!(screen.pending_promotion.is_none());
        let piece = screen.board_widget.position.board().piece_at(Square::E8).unwrap();
        assert_eq!(piece.role, Role::Bishop);
        assert_eq!(screen.move_history.last().unwrap().san, "e8=B");
    }
}
