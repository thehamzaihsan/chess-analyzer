use crate::db::Puzzle;
use crate::ui::board::{BoardMessage, BoardWidget};
use crate::ui::game_layout::{
    compute_board_layout, render_board_workspace, render_result_banner_with_tone,
    render_sized_board_column, GameOutcome, OutcomeTone, PlayerInfo, PlayerResult,
};
use crate::ui::theme::{
    badge_container_style, card_container_style, fonts, icon_check, icon_chevron_right, icon_eye,
    icon_refresh_cw, icon_sparkles, icon_widget, icon_x, palette, primary_button_style,
    secondary_button_style, segmented_button_style, segmented_container_style, spacing, truncate,
    type_scale, with_tooltip,
};
use iced::widget::{button, column, container, row, text, tooltip, Space};
use iced::{Alignment, Element, Length};
use shakmaty::{CastlingMode, Chess, Color, Move, Position, Role, Square};

#[derive(Debug, Clone)]
pub enum PuzzleMessage {
    Board(BoardMessage),
    NextPuzzle,
    MineBlunderPuzzles,
    SelectFilter(Option<String>), // None = all, Some("blunder"), Some("lichess")
    SelectPromotion(Role),
    CancelPromotion,
    ShowHint,
    ShowSolution,
    ResetPuzzle,
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
    pub user_is_white: bool,
    pub pending_promotion: Option<(Square, Square)>,
    pub has_incorrect_attempt: bool,
    pub revealed_solution: Option<String>,
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
            user_is_white: true,
            pending_promotion: None,
            has_incorrect_attempt: false,
            revealed_solution: None,
        }
    }

    pub fn load_puzzle(&mut self, puzzle: Puzzle) {
        let moves: Vec<String> = puzzle
            .moves
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut pos = puzzle
            .fen
            .parse::<shakmaty::fen::Fen>()
            .ok()
            .and_then(|f| f.into_position::<Chess>(CastlingMode::Standard).ok())
            .unwrap_or_default();

        let mut step = 0usize;
        let mut last_move = None;

        // Lichess puzzles store the position BEFORE the opponent's setup move (`moves[0]`).
        // Auto-play `moves[0]` so the board is in the puzzle position with the solver to move (`moves[1]`).
        if puzzle.source == "lichess" {
            if let Some(setup_uci) = moves.first() {
                if let Ok(uci_m) = setup_uci.parse::<shakmaty::uci::UciMove>() {
                    if let Ok(valid_m) = uci_m.to_move(&pos) {
                        let from = valid_m.from();
                        let to = valid_m.to();
                        if let Ok(next_pos) = pos.clone().play(valid_m) {
                            pos = next_pos;
                            last_move = from.map(|f| (f, to));
                            step = 1;
                        }
                    }
                }
            }
        }

        let user_is_white = pos.turn() == Color::White;
        self.user_is_white = user_is_white;
        self.board_widget = BoardWidget::new(pos);
        self.board_widget.flipped = !user_is_white;
        self.board_widget.last_move = last_move;
        self.board_widget.best_move_arrow = None;

        self.puzzle_moves = moves;
        self.current_move_step = step;
        self.is_solved = false;
        self.pending_promotion = None;
        self.has_incorrect_attempt = false;
        self.revealed_solution = None;
        self.puzzle_status = format!(
            "Find the best move for {}! (Rating: {}, Source: {})",
            if user_is_white { "White" } else { "Black" },
            puzzle.rating,
            puzzle.source
        );
        self.current_puzzle = Some(puzzle);
    }

    pub fn reset_current_puzzle(&mut self) {
        if let Some(puzzle) = self.current_puzzle.clone() {
            crate::audio::play_sound(crate::audio::SoundType::GameStart);
            self.load_puzzle(puzzle);
        }
    }

    fn user_color(&self) -> Color {
        if self.user_is_white {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn handle_square_clicked(&mut self, sq: Square) {
        if self.is_solved || self.current_puzzle.is_none() {
            return;
        }
        if self.board_widget.position.turn() != self.user_color() {
            return;
        }

        if self.pending_promotion.is_some() {
            self.pending_promotion = None;
            if self.board_widget.position.board().color_at(sq) == Some(self.user_color()) {
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
                self.puzzle_status = "Choose promotion piece.".to_string();
                return;
            }

            if let Some(m) = matching.into_iter().next() {
                self.try_user_move(m);
            } else if self.board_widget.position.board().color_at(sq) == Some(self.user_color()) {
                self.board_widget.selected_square = Some(sq);
            } else {
                self.board_widget.selected_square = None;
            }
        } else if self.board_widget.position.board().color_at(sq) == Some(self.user_color()) {
            self.board_widget.selected_square = Some(sq);
        }
    }

    pub fn handle_promotion(&mut self, role: Role) {
        if self.is_solved || self.current_puzzle.is_none() {
            self.pending_promotion = None;
            return;
        }

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
            self.try_user_move(m);
        }
    }

    pub fn cancel_promotion(&mut self) {
        self.pending_promotion = None;
        if !self.is_solved {
            self.puzzle_status = format!(
                "Find the best move for {}!",
                if self.user_is_white { "White" } else { "Black" }
            );
        }
    }

    fn try_user_move(&mut self, m: Move) {
        let from = m.from().unwrap_or(m.to());
        let to = m.to();
        let uci_str = m.to_uci(CastlingMode::Standard).to_string();
        let expected = self.puzzle_moves.get(self.current_move_step).map(|s| s.as_str());

        if expected == Some(uci_str.as_str()) {
            if let Ok(next_pos) = self.board_widget.position.clone().play(m.clone()) {
                let sound = crate::audio::sound_for_move(&m, &next_pos);
                crate::audio::play_sound(sound);
                self.board_widget.position = next_pos;
                self.board_widget.last_move = Some((from, to));
                self.board_widget.selected_square = None;
                self.board_widget.best_move_arrow = None;
                self.pending_promotion = None;
                self.has_incorrect_attempt = false;
                self.revealed_solution = None;
                self.current_move_step += 1;

                if self.current_move_step >= self.puzzle_moves.len() {
                    crate::audio::play_sound(crate::audio::SoundType::GameEnd);
                    self.is_solved = true;
                    self.board_widget.interactive = false;
                    self.puzzle_status = "Correct! Puzzle solved!".to_string();
                } else {
                    if let Some(opp_uci) = self.puzzle_moves.get(self.current_move_step) {
                        if let Ok(opp_m) = opp_uci.parse::<shakmaty::uci::UciMove>() {
                            if let Ok(valid_opp) = opp_m.to_move(&self.board_widget.position) {
                                let opp_from = valid_opp.from().unwrap_or(to);
                                let opp_to = valid_opp.to();
                                if let Ok(next_opp_pos) =
                                    self.board_widget.position.clone().play(valid_opp.clone())
                                {
                                    let opp_sound =
                                        crate::audio::sound_for_move(&valid_opp, &next_opp_pos);
                                    crate::audio::play_sound(opp_sound);
                                    self.board_widget.position = next_opp_pos;
                                    self.board_widget.last_move = Some((opp_from, opp_to));
                                    self.current_move_step += 1;
                                }
                            }
                        }
                    }

                    if self.current_move_step >= self.puzzle_moves.len() {
                        crate::audio::play_sound(crate::audio::SoundType::GameEnd);
                        self.is_solved = true;
                        self.board_widget.interactive = false;
                        self.puzzle_status = "Correct! Puzzle solved!".to_string();
                    } else {
                        self.puzzle_status = "Good move! Find the continuation.".to_string();
                    }
                }
            }
        } else {
            self.has_incorrect_attempt = true;
            self.puzzle_status = "Incorrect move. Try again or use Hint / Show Solution!".to_string();
            self.board_widget.selected_square = None;
        }
    }

    pub fn show_hint(&mut self) {
        if self.is_solved || self.current_puzzle.is_none() {
            return;
        }
        if let Some(expected_uci) = self.puzzle_moves.get(self.current_move_step) {
            if let Ok(uci_m) = expected_uci.parse::<shakmaty::uci::UciMove>() {
                if let Ok(valid_m) = uci_m.to_move(&self.board_widget.position) {
                    if let Some(from) = valid_m.from() {
                        self.board_widget.best_move_arrow = Some((from, valid_m.to()));
                        self.board_widget.selected_square = Some(from);
                        self.puzzle_status = format!("Hint: consider moving the piece on {}.", from);
                        return;
                    }
                }
            }
        }
        self.puzzle_status = "No hint available for this position.".to_string();
    }

    pub fn show_solution(&mut self) {
        if self.current_puzzle.is_none() {
            return;
        }

        let mut sim_pos = self.board_widget.position.clone();
        let mut san_steps = Vec::new();
        let mut first_arrow = None;

        for uci_str in self.puzzle_moves.iter().skip(self.current_move_step) {
            if let Ok(uci_m) = uci_str.parse::<shakmaty::uci::UciMove>() {
                if let Ok(valid_m) = uci_m.to_move(&sim_pos) {
                    if first_arrow.is_none() {
                        first_arrow = valid_m.from().map(|f| (f, valid_m.to()));
                    }
                    let san = shakmaty::san::SanPlus::from_move(sim_pos.clone(), valid_m.clone())
                        .to_string();
                    san_steps.push(san);
                    if let Ok(next) = sim_pos.clone().play(valid_m) {
                        sim_pos = next;
                    } else {
                        break;
                    }
                } else {
                    san_steps.push(uci_str.clone());
                    break;
                }
            } else {
                san_steps.push(uci_str.clone());
                break;
            }
        }

        if san_steps.is_empty() {
            self.puzzle_status = "Puzzle already complete.".to_string();
            return;
        }

        let line = san_steps.join(" ");
        self.board_widget.best_move_arrow = first_arrow;
        self.revealed_solution = Some(line.clone());
        self.puzzle_status = format!("Solution: {}", line);
    }

    pub fn solver_move_progress(&self) -> (usize, usize) {
        let Some(ref p) = self.current_puzzle else {
            return (0, 0);
        };
        let is_lichess = p.source == "lichess";
        let setup_offset = if is_lichess && !self.puzzle_moves.is_empty() {
            1
        } else {
            0
        };
        let solver_total = self.puzzle_moves.len().saturating_sub(setup_offset).div_ceil(2);
        if solver_total == 0 {
            return (0, 0);
        }
        if self.is_solved {
            return (solver_total, solver_total);
        }
        let played_since_setup = self.current_move_step.saturating_sub(setup_offset);
        let current = ((played_since_setup / 2) + 1).min(solver_total);
        (current, solver_total)
    }

    pub fn view<'a>(&'a self, avail_size: iced::Size) -> Element<'a, PuzzleMessage> {
        let is_white_turn = self.board_widget.position.turn() == Color::White;
            let puzzle_rating = self.current_puzzle.as_ref().map(|p| p.rating);
            let puzzle_source = self
                .current_puzzle
                .as_ref()
                .map(|p| p.source.as_str())
                .unwrap_or("Puzzle");

            let user_is_white = self.user_is_white;
            let opp_is_white = !user_is_white;

            let user_active = !self.is_solved && (is_white_turn == user_is_white);
            let opp_active = !self.is_solved && (is_white_turn == opp_is_white);

            let (user_badge, opp_badge) = if self.is_solved {
                (Some(PlayerResult::Win), Some(PlayerResult::Loss))
            } else if self.revealed_solution.is_some() {
                (Some(PlayerResult::Loss), Some(PlayerResult::Win))
            } else {
                (None, None)
            };

            let opp_player = PlayerInfo::new(format!("Opponent ({})", puzzle_source), opp_is_white)
                .with_rating(puzzle_rating)
                .with_title("PUZZLE")
                .with_active_turn(opp_active)
                .with_result(opp_badge);

            let user_status = if self.is_solved {
                "Solved!"
            } else if self.revealed_solution.is_some() {
                "Solution revealed"
            } else if self.puzzle_status.starts_with("Incorrect") {
                "Try again!"
            } else {
                "Your turn"
            };

            let user_player = PlayerInfo::new("You", user_is_white)
                .with_status(user_status)
                .with_active_turn(user_active)
                .with_result(user_badge);

            // board_widget.flipped is always !user_is_white, so user is always at the bottom.
            let (top_player, bottom_player) = (&opp_player, &user_player);

            let metrics = compute_board_layout(avail_size, false);
            let board_el = render_sized_board_column(
                top_player,
                bottom_player,
                &self.board_widget.position,
                &self.board_widget,
                PuzzleMessage::Board,
                None,
                metrics.board_side,
            );

            // 1. Segmented Filter Control
            let make_filter_btn = |filter_opt: Option<String>, label: &'static str| {
                let is_sel = self.filter == filter_opt;
                button(
                    container(
                        text(label)
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::CAPTION)
                            .wrapping(text::Wrapping::None),
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
                )
                .width(Length::FillPortion(1))
                .padding([spacing::XXS, spacing::XS])
                .style(segmented_button_style(is_sel))
                .on_press(PuzzleMessage::SelectFilter(filter_opt))
            };

            let filter_control = container(
                row![
                    make_filter_btn(None, "All"),
                    make_filter_btn(Some("blunder".to_string()), "My Blunders"),
                    make_filter_btn(Some("lichess".to_string()), "Lichess"),
                ]
                .spacing(4.0)
                .width(Length::Fill),
            )
            .padding(4.0)
            .width(Length::Fill)
            .clip(true)
            .style(segmented_container_style);

            // 2. Structured Puzzle Info Card
            let info_card: Element<'a, PuzzleMessage> = if let Some(ref puzzle) = self.current_puzzle
            {
                let (cur_move, total_moves) = self.solver_move_progress();
                let source_label = if puzzle.source == "lichess" {
                    "Lichess"
                } else {
                    "My Blunder"
                };
                let to_move_label = if self.user_is_white {
                    "White to move"
                } else {
                    "Black to move"
                };

                let progress_badge = container(
                    text(if total_moves > 0 {
                        format!("Move {} of {}", cur_move, total_moves)
                    } else {
                        "Ready".to_string()
                    })
                    .font(fonts::MONO_BOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_PRIMARY)
                    .wrapping(text::Wrapping::None),
                )
                .padding([2.0, 8.0])
                .style(badge_container_style(palette::ACCENT_MUTED));

                let rating_badge = container(
                    text(format!("{} Elo", puzzle.rating))
                        .font(fonts::MONO_BOLD)
                        .size(type_scale::CAPTION)
                        .color(palette::TEXT_PRIMARY)
                        .wrapping(text::Wrapping::None),
                )
                .padding([2.0, 8.0])
                .style(badge_container_style(palette::BG_SURFACE));

                let source_badge = container(
                    text(source_label)
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::CAPTION)
                        .color(palette::TEXT_SECONDARY)
                        .wrapping(text::Wrapping::None),
                )
                .padding([2.0, 8.0])
                .style(badge_container_style(palette::BG_SURFACE));

                let top_meta = row![
                    text(to_move_label)
                        .font(fonts::UI_BOLD)
                        .size(type_scale::HEADING)
                        .color(palette::TEXT_PRIMARY)
                        .wrapping(text::Wrapping::None),
                    Space::with_width(Length::Fill),
                    progress_badge,
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill);

                let badges_row = row![rating_badge, source_badge]
                    .spacing(spacing::XS)
                    .align_y(Alignment::Center)
                    .width(Length::Fill);

                let mut info_col = column![top_meta, badges_row]
                    .spacing(spacing::XS)
                    .width(Length::Fill);

                let themes_clean = puzzle.themes.trim();
                if !themes_clean.is_empty() {
                    let short_themes = truncate(themes_clean, 44);
                    let themes_txt = text(format!("Themes: {}", short_themes))
                        .font(fonts::UI)
                        .size(type_scale::CAPTION)
                        .color(palette::TEXT_MUTED)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .width(Length::Fill);
                    let themes_el: Element<'a, PuzzleMessage> = if short_themes != themes_clean {
                        with_tooltip(themes_txt, themes_clean, tooltip::Position::Bottom)
                    } else {
                        themes_txt.into()
                    };
                    info_col = info_col.push(themes_el);
                }

                container(info_col)
                    .padding(spacing::SM)
                    .width(Length::Fill)
                    .clip(true)
                    .style(card_container_style)
                    .into()
            } else {
                container(
                    text("No puzzle loaded — click Next Puzzle or Mine Blunders.")
                        .font(fonts::UI)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_MUTED)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .width(Length::Fill),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .clip(true)
                .style(card_container_style)
                .into()
            };

            // 3. Feedback / Outcome Banner
            let (status_icon, status_color) = if self.is_solved {
                (Some(icon_check()), palette::SUCCESS)
            } else if self.puzzle_status.starts_with("Incorrect") {
                (Some(icon_x()), palette::DANGER)
            } else {
                (None, palette::TEXT_SECONDARY)
            };

            let status_row = if let Some(icon) = status_icon {
                row![
                    icon_widget(icon, 15.0, Some(status_color)),
                    Space::with_width(spacing::XS),
                    text(&self.puzzle_status)
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::BODY)
                        .color(status_color)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .width(Length::Fill),
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill)
            } else {
                row![text(&self.puzzle_status)
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::BODY)
                    .color(status_color)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .width(Length::Fill)]
                .align_y(Alignment::Center)
                .width(Length::Fill)
            };

            let status_box = container(status_row)
                .padding(spacing::SM)
                .width(Length::Fill)
                .clip(true)
                .style(card_container_style);

            let feedback_el: Element<'a, PuzzleMessage> = if self.is_solved {
                let score = if user_is_white { "1-0" } else { "0-1" };
                render_result_banner_with_tone(
                    &GameOutcome::new(score, "Puzzle Solved!"),
                    OutcomeTone::Win,
                )
            } else if self.revealed_solution.is_some() {
                let score = if user_is_white { "0-1" } else { "1-0" };
                render_result_banner_with_tone(
                    &GameOutcome::new(score, "Puzzle Failed — Solution Revealed"),
                    OutcomeTone::Loss,
                )
            } else if self.has_incorrect_attempt {
                render_result_banner_with_tone(
                    &GameOutcome::new("TRY", "Incorrect move — try again or use Hint"),
                    OutcomeTone::Loss,
                )
            } else {
                status_box.into()
            };

            // 4. Structured Action Bar
            let next_btn = button(
                container(
                    row![
                        text("Next Puzzle")
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::BODY)
                            .wrapping(text::Wrapping::None),
                        icon_widget(icon_chevron_right(), 14.0, Some(palette::TEXT_PRIMARY)),
                    ]
                    .spacing(spacing::XS)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .align_x(Alignment::Center),
            )
            .width(Length::FillPortion(1))
            .padding([spacing::XS, spacing::SM])
            .style(primary_button_style)
            .on_press(PuzzleMessage::NextPuzzle);

            let mine_btn = button(
                container(
                    row![
                        icon_widget(icon_sparkles(), 14.0, Some(palette::TEXT_PRIMARY)),
                        text("Mine Blunders")
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::BODY)
                            .wrapping(text::Wrapping::None),
                    ]
                    .spacing(spacing::XS)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .align_x(Alignment::Center),
            )
            .width(Length::FillPortion(1))
            .padding([spacing::XS, spacing::SM])
            .style(secondary_button_style)
            .on_press(PuzzleMessage::MineBlunderPuzzles);

            let primary_action_row = row![next_btn, mine_btn]
                .spacing(spacing::XS)
                .width(Length::Fill);

            let mut side_col = column![
                filter_control,
                info_card,
                feedback_el,
                primary_action_row,
            ]
            .spacing(spacing::SM)
            .width(Length::Fill)
            .height(Length::Fill);

            if self.current_puzzle.is_some() {
                let hint_btn = button(
                    container(
                        row![
                            icon_widget(icon_sparkles(), 13.0, Some(palette::TEXT_PRIMARY)),
                            text("Hint")
                                .font(fonts::UI_SEMIBOLD)
                                .size(type_scale::CAPTION)
                                .wrapping(text::Wrapping::None),
                        ]
                        .spacing(spacing::XXS)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
                )
                .width(Length::FillPortion(1))
                .padding([spacing::XS, spacing::XS])
                .style(secondary_button_style)
                .on_press_maybe((!self.is_solved).then_some(PuzzleMessage::ShowHint));

                let solution_btn = button(
                    container(
                        row![
                            icon_widget(icon_eye(), 13.0, Some(palette::TEXT_PRIMARY)),
                            text("Solution")
                                .font(fonts::UI_SEMIBOLD)
                                .size(type_scale::CAPTION)
                                .wrapping(text::Wrapping::None),
                        ]
                        .spacing(spacing::XXS)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
                )
                .width(Length::FillPortion(1))
                .padding([spacing::XS, spacing::XS])
                .style(secondary_button_style)
                .on_press_maybe((!self.is_solved).then_some(PuzzleMessage::ShowSolution));

                let reset_btn = button(
                    container(
                        row![
                            icon_widget(icon_refresh_cw(), 13.0, Some(palette::TEXT_PRIMARY)),
                            text("Reset")
                                .font(fonts::UI_SEMIBOLD)
                                .size(type_scale::CAPTION)
                                .wrapping(text::Wrapping::None),
                        ]
                        .spacing(spacing::XXS)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
                )
                .width(Length::FillPortion(1))
                .padding([spacing::XS, spacing::XS])
                .style(secondary_button_style)
                .on_press(PuzzleMessage::ResetPuzzle);

                let helper_row = row![hint_btn, solution_btn, reset_btn]
                    .spacing(spacing::XS)
                    .width(Length::Fill);

                side_col = side_col.push(helper_row);
            }

            if let Some(ref solution_line) = self.revealed_solution {
                let sol_box = container(
                    column![
                        text("Solution Line")
                            .font(fonts::UI)
                            .size(type_scale::CAPTION)
                            .color(palette::TEXT_MUTED),
                        text(solution_line)
                            .font(fonts::MONO_BOLD)
                            .size(type_scale::BODY)
                            .color(palette::ACCENT)
                            .wrapping(text::Wrapping::WordOrGlyph)
                            .width(Length::Fill),
                    ]
                    .spacing(spacing::XXS)
                    .width(Length::Fill),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .clip(true)
                .style(card_container_style);

                side_col = side_col.push(sol_box);
            }

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
                    .on_press(PuzzleMessage::SelectPromotion(role))
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
                            .on_press(PuzzleMessage::CancelPromotion),
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

                side_col = side_col.push(promo_box);
            }

            render_board_workspace(board_el, side_col.into(), metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lichess_puzzle() -> Puzzle {
        // Initial FEN has White to move; moves[0] is White's setup move e2e4;
        // solver is Black and plays e7e5, then White plays g1f3, Black plays b8c6.
        Puzzle {
            id: "lichess_1".to_string(),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            moves: "e2e4 e7e5 g1f3 b8c6".to_string(),
            rating: 1400,
            rating_deviation: 80,
            popularity: 95,
            nb_plays: 100,
            themes: "opening".to_string(),
            opening_tags: "".to_string(),
            source: "lichess".to_string(),
        }
    }

    #[test]
    fn lichess_puzzle_auto_plays_setup_move_and_orients_to_solver() {
        let mut screen = PuzzleScreen::new();
        screen.load_puzzle(sample_lichess_puzzle());

        // Setup move e2e4 was auto-played -> solver is Black
        assert_eq!(screen.current_move_step, 1);
        assert_eq!(screen.solver_move_progress(), (1, 2));
        assert!(!screen.user_is_white);
        assert!(screen.board_widget.flipped);
        assert_eq!(screen.board_widget.last_move, Some((Square::E2, Square::E4)));
        assert_eq!(screen.board_widget.position.turn(), Color::Black);

        // User plays moves[1] = e7e5
        screen.handle_square_clicked(Square::E7);
        screen.handle_square_clicked(Square::E5);

        // Opponent reply g1f3 auto-played, now step = 3 (b8c6)
        assert_eq!(screen.current_move_step, 3);
        assert_eq!(screen.solver_move_progress(), (2, 2));
        assert!(!screen.is_solved);
        assert_eq!(screen.board_widget.last_move, Some((Square::G1, Square::F3)));

        // User plays moves[3] = b8c6 -> solved
        screen.handle_square_clicked(Square::B8);
        screen.handle_square_clicked(Square::C6);
        assert!(screen.is_solved);
        assert_eq!(screen.solver_move_progress(), (2, 2));
    }

    #[test]
    fn puzzle_promotion_compares_full_uci_with_promotion_letter() {
        let mut screen = PuzzleScreen::new();
        let puzzle = Puzzle {
            id: "promo_1".to_string(),
            fen: "k7/4P3/8/8/8/8/8/4K3 w - - 0 1".to_string(),
            moves: "e7e8n".to_string(),
            rating: 1600,
            rating_deviation: 80,
            popularity: 90,
            nb_plays: 50,
            themes: "underpromotion".to_string(),
            opening_tags: "".to_string(),
            source: "blunder".to_string(),
        };
        screen.load_puzzle(puzzle);
        assert!(screen.user_is_white);
        assert!(!screen.board_widget.flipped);
        assert_eq!(screen.current_move_step, 0);
        assert_eq!(screen.solver_move_progress(), (1, 1));

        // Choose Queen (wrong promotion letter)
        screen.handle_square_clicked(Square::E7);
        screen.handle_square_clicked(Square::E8);
        assert_eq!(screen.pending_promotion, Some((Square::E7, Square::E8)));
        screen.handle_promotion(Role::Queen);
        assert!(!screen.is_solved);
        assert!(screen.has_incorrect_attempt);

        // Choose Knight (expected e7e8n)
        screen.handle_square_clicked(Square::E7);
        screen.handle_square_clicked(Square::E8);
        screen.handle_promotion(Role::Knight);
        assert!(screen.is_solved);
    }

    #[test]
    fn hint_solution_and_reset_work_after_incorrect_move() {
        let mut screen = PuzzleScreen::new();
        screen.load_puzzle(sample_lichess_puzzle());

        // Incorrect move a7a6
        screen.handle_square_clicked(Square::A7);
        screen.handle_square_clicked(Square::A6);
        assert!(screen.has_incorrect_attempt);
        assert!(screen.puzzle_status.starts_with("Incorrect"));

        // Hint highlights E7 -> E5
        screen.show_hint();
        assert_eq!(screen.board_widget.best_move_arrow, Some((Square::E7, Square::E5)));

        // Show solution reveals SAN line
        screen.show_solution();
        assert!(screen.revealed_solution.is_some());
        assert!(screen.puzzle_status.starts_with("Solution:"));

        // Reset restores puzzle to post-setup state
        screen.reset_current_puzzle();
        assert!(!screen.has_incorrect_attempt);
        assert!(screen.revealed_solution.is_none());
        assert!(screen.board_widget.best_move_arrow.is_none());
        assert_eq!(screen.current_move_step, 1);
    }
}
