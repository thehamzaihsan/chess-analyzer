use crate::analysis::GameAnalysis;
use crate::db::StoredGame;
use crate::ui::board::{BoardMessage, BoardWidget};
use shakmaty::Position;
use crate::ui::game_layout::{render_unified_board_column, PlayerInfo};
use crate::ui::charts::{EvalGraphMessage, EvalGraphWidget};
use crate::ui::theme::{
    badge_container_style, card_container_style, classification_color, fonts,
    icon_chevron_left, icon_chevron_right, icon_rotate_ccw, icon_skip_back, icon_skip_forward,
    icon_sparkles, icon_widget, palette, primary_button_style, secondary_button_style,
    spacing, type_scale,
};
use iced::border;
use iced::widget::{
    button, column, container, row, scrollable, text, Space,
};
use iced::{Alignment, Color, Element, Length};
use shakmaty::Chess;

#[derive(Debug, Clone)]
pub enum GameReviewMessage {
    Board(BoardMessage),
    FirstMove,
    PrevMove,
    NextMove,
    LastMove,
    GoToMove(usize),
    FlipBoard,
    AskCoach,
    AnalyzeCurrentGame,
    EvalGraph(EvalGraphMessage),
}

#[derive(Debug, Clone)]
pub struct GameReviewScreen {
    pub current_move_idx: usize, // 0 = start position, 1 = after move 1, etc.
    pub board_widget: BoardWidget,
    pub coach_explanation: Option<String>,
    pub is_coach_loading: bool,
    pub cached_moves: Vec<crate::pgn::ParsedMove>,
    pub is_analyzing: bool,
    pub analysis_error: Option<String>,
}

impl GameReviewScreen {
    pub fn new() -> Self {
        Self {
            current_move_idx: 0,
            board_widget: BoardWidget::new(Chess::default()),
            coach_explanation: None,
            is_coach_loading: false,
            cached_moves: Vec::new(),
            is_analyzing: false,
            analysis_error: None,
        }
    }

    pub fn load_game(&mut self, game: &StoredGame) {
        self.current_move_idx = 0;
        self.board_widget = BoardWidget::new(Chess::default());
        self.coach_explanation = None;
        self.is_coach_loading = false;
        self.is_analyzing = false;
        self.analysis_error = None;
        self.cached_moves = crate::pgn::parse_pgn_games(&game.pgn)
            .into_iter()
            .next()
            .map(|g| g.moves)
            .unwrap_or_default();
    }

    pub fn update_board_to_move(&mut self, game: &StoredGame, idx: usize) {
        self.current_move_idx = idx;
        self.coach_explanation = None;

        if self.cached_moves.is_empty() && !game.pgn.trim().is_empty() {
            self.cached_moves = crate::pgn::parse_pgn_games(&game.pgn)
                .into_iter()
                .next()
                .map(|g| g.moves)
                .unwrap_or_default();
        }

        let mut pos = Chess::default();
        let mut last_m = None;

        let target_moves = &self.cached_moves[..idx.min(self.cached_moves.len())];
        for m in target_moves {
            if let Ok(fen) = m.fen_after.parse::<shakmaty::fen::Fen>() {
                if let Ok(next_pos) = fen.into_position(shakmaty::CastlingMode::Standard) {
                    pos = next_pos;
                }
            }
            if let Ok(uci_m) = m.uci.parse::<shakmaty::uci::UciMove>() {
                if let shakmaty::uci::UciMove::Normal { from, to, .. } = uci_m {
                    last_m = Some((from, to));
                }
            }
        }

        self.board_widget.position = pos;
        self.board_widget.last_move = last_m;
        self.board_widget.best_move_arrow = None;
    }

    pub fn set_position_from_analysis(&mut self, analysis: Option<&GameAnalysis>, ply: usize) {
        self.current_move_idx = ply;
        self.coach_explanation = None;
        self.board_widget.last_move = None;
        self.board_widget.best_move_arrow = None;

        if let Some(an) = analysis {
            if ply == 0 {
                self.board_widget.position = Chess::default();
            } else if let Some(m) = an.moves.get(ply - 1) {
                if let Ok(fen) = m.fen_after.parse::<shakmaty::fen::Fen>() {
                    if let Ok(pos) = fen.into_position(shakmaty::CastlingMode::Standard) {
                        self.board_widget.position = pos;
                    }
                }
                if let Ok(uci_m) = m.uci.parse::<shakmaty::uci::UciMove>() {
                    if let shakmaty::uci::UciMove::Normal { from, to, .. } = uci_m {
                        self.board_widget.last_move = Some((from, to));
                    }
                }

                if let Some(eval) = &m.eval_before {
                    if let Some(bm_str) = &eval.best_move {
                        if let Ok(bm) = bm_str.parse::<shakmaty::uci::UciMove>() {
                            if let shakmaty::uci::UciMove::Normal { from, to, .. } = bm {
                                self.board_widget.best_move_arrow = Some((from, to));
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        game: Option<&'a StoredGame>,
        analysis: Option<&'a GameAnalysis>,
    ) -> Element<'a, GameReviewMessage> {
        let (white_name, white_elo) = game.map(|g| (g.white.as_str(), g.white_elo)).unwrap_or(("White", None));
        let (black_name, black_elo) = game.map(|g| (g.black.as_str(), g.black_elo)).unwrap_or(("Black", None));

        let is_white_turn = self.board_widget.position.turn() == shakmaty::Color::White;

        let white_player = PlayerInfo::new(white_name, true)
            .with_rating(white_elo)
            .with_active_turn(is_white_turn);

        let black_player = PlayerInfo::new(black_name, false)
            .with_rating(black_elo)
            .with_active_turn(!is_white_turn);

        let (top_player, bottom_player) = if self.board_widget.flipped {
            (&white_player, &black_player)
        } else {
            (&black_player, &white_player)
        };

        let eval_bar = self.render_eval_bar(analysis);

        let board_column = render_unified_board_column(
            top_player,
            bottom_player,
            &self.board_widget.position,
            &self.board_widget,
            GameReviewMessage::Board,
            Some(eval_bar),
        );

        let sidebar = self.render_sidebar(game, analysis);

        let content = row![
            board_column,
            Space::with_width(spacing::LG),
            sidebar,
        ]
        .padding(spacing::MD)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content).into()
    }

    fn render_eval_bar<'a>(&'a self, analysis: Option<&'a GameAnalysis>) -> Element<'a, GameReviewMessage> {
        let (score_text, white_pct) = match analysis {
            Some(an) => {
                let current_eval = if self.current_move_idx == 0 {
                    an.moves.first().and_then(|m| m.eval_before.clone())
                } else {
                    an.moves.get(self.current_move_idx - 1).and_then(|m| m.eval_after.clone())
                };

                match current_eval {
                    Some(e) => {
                        if let Some(m) = e.mate_in {
                            let text = format!("M{}", m.abs());
                            let pct = if m > 0 { 1.0 } else { 0.0 };
                            (text, pct)
                        } else if let Some(cp) = e.score_cp {
                            let val = cp as f32 / 100.0;
                            let text = format!("{:+.1}", val);
                            let sigmoid = 1.0 / (1.0 + (-0.003682 * cp as f64).exp()) as f32;
                            (text, sigmoid)
                        } else {
                            ("0.0".to_string(), 0.5)
                        }
                    }
                    None => ("—".to_string(), 0.5),
                }
            }
            None => ("—".to_string(), 0.5),
        };

        let bar_width = 34.0;
        // Fill the board column's height and split it proportionally, so the
        // bar tracks the responsive board instead of a fixed pixel height.
        let white_portion = ((white_pct.clamp(0.0, 1.0) * 1000.0).round() as u16).max(1);
        let black_portion = (1000u16.saturating_sub(white_portion)).max(1);
        let flipped = self.board_widget.flipped;

        let black_part = container(Space::new(Length::Fixed(bar_width), Length::Fill))
            .height(Length::FillPortion(black_portion))
            .style(|_theme| container::Style {
                background: Some(palette::BG_SURFACE.into()),
                ..Default::default()
            });

        let white_color = Color::from_rgb8(218, 224, 233);
        let white_part = container(
            text(score_text)
                .font(fonts::MONO_BOLD)
                .size(type_scale::CAPTION)
                .color(palette::BG_BASE),
        )
        .padding(spacing::XXS)
        .width(Length::Fixed(bar_width))
        .height(Length::FillPortion(white_portion))
        .align_x(Alignment::Center)
        .align_y(if flipped { Alignment::Start } else { Alignment::End })
        .style(move |_theme| container::Style {
            background: Some(white_color.into()),
            ..Default::default()
        });

        // White's share sits on White's side of the board (bottom unless flipped).
        let parts = if flipped {
            column![white_part, black_part]
        } else {
            column![black_part, white_part]
        };

        container(parts)
            .width(Length::Fixed(bar_width))
            .height(Length::Fill)
            .clip(true)
            .style(|_theme| container::Style {
                background: Some(palette::BG_SURFACE.into()),
                border: border::rounded(spacing::RADIUS).color(palette::BORDER).width(1.0),
                ..Default::default()
            })
            .into()
    }

    fn render_sidebar<'a>(
        &'a self,
        game: Option<&'a StoredGame>,
        analysis: Option<&'a GameAnalysis>,
    ) -> Element<'a, GameReviewMessage> {
        let game_header = match game {
            Some(g) => {
                let white_title = format!("{} ({})", g.white, g.white_elo.unwrap_or(0));
                let black_title = format!("{} ({})", g.black, g.black_elo.unwrap_or(0));
                let meta = format!("{} • {} • Result: {}", g.date, g.opening, g.result);

                column![
                    text(format!("{} vs {}", white_title, black_title))
                        .font(fonts::UI_BOLD)
                        .size(type_scale::TITLE)
                        .color(palette::TEXT_PRIMARY),
                    text(meta)
                        .font(fonts::UI)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_SECONDARY),
                ]
                .spacing(spacing::XXS)
            }
            None => column![
                text("No Game Selected")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::TITLE)
                    .color(palette::TEXT_MUTED)
            ],
        };

        // Accuracy card
        let accuracy_row: Element<'a, GameReviewMessage> = match analysis {
            Some(an) => {
                let white_acc = container(
                    text(format!("White: {:.1}%", an.white_accuracy))
                        .font(fonts::MONO_BOLD)
                        .size(type_scale::HEADING)
                        .color(palette::TEXT_PRIMARY)
                )
                .padding([spacing::XXS, spacing::XS])
                .style(badge_container_style(Color::from_rgba(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0, 0.25)));

                let black_acc = container(
                    text(format!("Black: {:.1}%", an.black_accuracy))
                        .font(fonts::MONO_BOLD)
                        .size(type_scale::HEADING)
                        .color(palette::TEXT_PRIMARY)
                )
                .padding([spacing::XXS, spacing::XS])
                .style(badge_container_style(Color::from_rgba(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0, 0.25)));

                let blunders = container(
                    text(format!("Blunders: {} / {}", an.white_blunders, an.black_blunders))
                        .font(fonts::MONO)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_PRIMARY)
                )
                .padding([spacing::XXS, spacing::XS])
                .style(badge_container_style(Color::from_rgba(218.0 / 255.0, 54.0 / 255.0, 51.0 / 255.0, 0.25)));

                row![white_acc, Space::with_width(spacing::SM), black_acc, Space::with_width(spacing::SM), blunders]
                    .align_y(Alignment::Center)
                    .into()
            }
            None => {
                let status_label = if self.is_analyzing {
                    text("Analyzing…")
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::BODY)
                        .color(palette::ACCENT)
                } else if let Some(err) = &self.analysis_error {
                    text(err)
                        .font(fonts::UI)
                        .size(type_scale::BODY)
                        .color(palette::DANGER)
                } else {
                    text("Game not analyzed yet.")
                        .font(fonts::UI)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_MUTED)
                };

                let btn_label = if self.is_analyzing {
                    "Analyzing…"
                } else {
                    "Analyze with Engine"
                };

                let can_analyze = !self.is_analyzing && game.is_some();

                row![
                    status_label,
                    Space::with_width(spacing::SM),
                    button(text(btn_label).font(fonts::UI_SEMIBOLD).size(type_scale::BODY))
                        .padding([spacing::XS, spacing::SM])
                        .style(primary_button_style)
                        .on_press_maybe(if can_analyze { Some(GameReviewMessage::AnalyzeCurrentGame) } else { None }),
                ]
                .align_y(Alignment::Center)
                .into()
            }
        };

        // Navigation controls with Lucide SVG icons
        let flip_content = row![
            icon_widget(icon_rotate_ccw(), 14.0, Some(palette::TEXT_PRIMARY)),
            text("Flip").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let nav_controls = row![
            button(icon_widget(icon_skip_back(), 15.0, Some(palette::TEXT_PRIMARY)))
                .padding([spacing::XS, spacing::SM])
                .style(secondary_button_style)
                .on_press(GameReviewMessage::FirstMove),
            button(icon_widget(icon_chevron_left(), 15.0, Some(palette::TEXT_PRIMARY)))
                .padding([spacing::XS, spacing::SM])
                .style(secondary_button_style)
                .on_press(GameReviewMessage::PrevMove),
            button(icon_widget(icon_chevron_right(), 15.0, Some(palette::TEXT_PRIMARY)))
                .padding([spacing::XS, spacing::SM])
                .style(secondary_button_style)
                .on_press(GameReviewMessage::NextMove),
            button(icon_widget(icon_skip_forward(), 15.0, Some(palette::TEXT_PRIMARY)))
                .padding([spacing::XS, spacing::SM])
                .style(secondary_button_style)
                .on_press(GameReviewMessage::LastMove),
            button(flip_content)
                .padding([spacing::XS, spacing::SM])
                .style(secondary_button_style)
                .on_press(GameReviewMessage::FlipBoard),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let keyboard_hint = text("Keys: ← Prev  •  → Next  •  Home / End (↑ / ↓)  •  F Flip")
            .font(fonts::UI)
            .size(type_scale::CAPTION)
            .color(palette::TEXT_MUTED);

        let nav_section = column![
            nav_controls,
            keyboard_hint,
        ]
        .spacing(spacing::XXS);

        // Move Evaluation Graph
        let eval_graph: Element<'a, GameReviewMessage> = match analysis {
            Some(an) if !an.moves.is_empty() => {
                let win_probs: Vec<Option<f32>> = an.moves.iter().map(|m| Some(m.win_prob_after as f32)).collect();
                let graph_widget = EvalGraphWidget::new(win_probs, self.current_move_idx)
                    .view(Length::Fill, Length::Fixed(48.0))
                    .map(|msg| match msg {
                        EvalGraphMessage::SelectPly(ply) => GameReviewMessage::GoToMove(ply),
                    });

                container(
                    column![
                        row![
                            text("Advantage Graph").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED),
                            Space::with_width(Length::Fill),
                            text("Click graph to jump").font(fonts::UI).size(type_scale::CAPTION).color(palette::TEXT_MUTED),
                        ]
                        .align_y(Alignment::Center),
                        Space::with_height(spacing::XXS),
                        graph_widget,
                    ]
                )
                .padding(spacing::XS)
                .style(card_container_style)
                .into()
            }
            _ => Space::new(Length::Shrink, Length::Shrink).into(),
        };

        // Current move details
        let current_move_detail: Element<'a, GameReviewMessage> = match analysis {
            Some(an) if self.current_move_idx > 0 && self.current_move_idx <= an.moves.len() => {
                let m = &an.moves[self.current_move_idx - 1];
                let ply_title = format!("Move {}. {}", m.move_number, m.san);
                let class_color = classification_color(m.classification);

                let class_badge = container(
                    text(format!("{} {}", m.classification.symbol(), m.classification.name()))
                        .font(fonts::UI_BOLD)
                        .size(type_scale::HEADING)
                        .color(palette::TEXT_PRIMARY)
                )
                .padding([spacing::XXS, spacing::XS])
                .style(badge_container_style(class_color));

                let best_line_str = m
                    .eval_before
                    .as_ref()
                    .and_then(|e| e.best_move.clone())
                    .unwrap_or_else(|| "None".to_string());

                let best_move_text = text(format!("Best Move: {}", best_line_str))
                    .font(fonts::MONO)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_SECONDARY);

                let loss_text = text(format!("Win% Loss: {:.1}%", m.win_prob_loss))
                    .font(fonts::MONO)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_SECONDARY);

                let coach_label = if self.is_coach_loading {
                    "Thinking..."
                } else {
                    "Ask Coach"
                };

                let coach_content = row![
                    icon_widget(icon_sparkles(), 14.0, Some(palette::TEXT_PRIMARY)),
                    text(coach_label).font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
                ]
                .spacing(spacing::XS)
                .align_y(Alignment::Center);

                let coach_btn = button(coach_content)
                    .padding([spacing::XS, spacing::SM])
                    .style(primary_button_style)
                    .on_press_maybe(if self.is_coach_loading { None } else { Some(GameReviewMessage::AskCoach) });

                let mut col = column![
                    row![
                        text(ply_title).font(fonts::MONO_BOLD).size(type_scale::TITLE).color(palette::TEXT_PRIMARY),
                        Space::with_width(spacing::SM),
                        class_badge,
                    ]
                    .align_y(Alignment::Center),
                    row![
                        best_move_text,
                        Space::with_width(spacing::MD),
                        loss_text,
                        Space::with_width(spacing::MD),
                        coach_btn,
                    ]
                    .align_y(Alignment::Center),
                ]
                .spacing(spacing::XS);

                if let Some(exp) = &self.coach_explanation {
                    let coach_box = container(
                        column![
                            row![
                                icon_widget(icon_sparkles(), 14.0, Some(palette::ACCENT)),
                                text("Coach Advice").font(fonts::UI_BOLD).size(type_scale::HEADING).color(palette::ACCENT),
                            ]
                            .spacing(spacing::XS)
                            .align_y(Alignment::Center),
                            Space::with_height(spacing::XXS),
                            text(exp).font(fonts::UI).size(type_scale::BODY).color(palette::TEXT_PRIMARY),
                        ]
                    )
                    .padding(spacing::SM)
                    .width(Length::Fill)
                    .style(card_container_style);

                    col = col.push(coach_box);
                }

                col.into()
            }
            _ => text("Position: Starting Position")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::HEADING)
                .color(palette::TEXT_MUTED)
                .into(),
        };

        // Scrollable move history list
        let mut move_rows = column![].spacing(spacing::XXS);
        if let Some(an) = analysis {
            let num_moves = (an.moves.len() + 1) / 2;
            for turn in 0..num_moves {
                let white_idx = turn * 2;
                let black_idx = turn * 2 + 1;

                let num_label = text(format!("{}.", turn + 1))
                    .font(fonts::MONO)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_MUTED)
                    .width(Length::Fixed(32.0));

                let white_btn = if let Some(m) = an.moves.get(white_idx) {
                    let is_active = self.current_move_idx == white_idx + 1;
                    let label = format!("{} {}", m.san, m.classification.symbol());
                    button(text(label).font(fonts::MONO).size(type_scale::BODY))
                        .padding([spacing::XXS, spacing::XS])
                        .style(move |_theme, _status| {
                            if is_active {
                                button::Style {
                                    background: Some(palette::ACCENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    border: border::rounded(spacing::RADIUS),
                                    ..Default::default()
                                }
                            } else {
                                button::Style {
                                    background: Some(Color::TRANSPARENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    ..Default::default()
                                }
                            }
                        })
                        .on_press(GameReviewMessage::GoToMove(white_idx + 1))
                } else {
                    button(text("").size(type_scale::BODY)).padding([spacing::XXS, spacing::XS])
                };

                let black_btn = if let Some(m) = an.moves.get(black_idx) {
                    let is_active = self.current_move_idx == black_idx + 1;
                    let label = format!("{} {}", m.san, m.classification.symbol());
                    button(text(label).font(fonts::MONO).size(type_scale::BODY))
                        .padding([spacing::XXS, spacing::XS])
                        .style(move |_theme, _status| {
                            if is_active {
                                button::Style {
                                    background: Some(palette::ACCENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    border: border::rounded(spacing::RADIUS),
                                    ..Default::default()
                                }
                            } else {
                                button::Style {
                                    background: Some(Color::TRANSPARENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    ..Default::default()
                                }
                            }
                        })
                        .on_press(GameReviewMessage::GoToMove(black_idx + 1))
                } else {
                    button(text("").size(type_scale::BODY)).padding([spacing::XXS, spacing::XS])
                };

                let row_item = row![
                    num_label,
                    container(white_btn).width(Length::FillPortion(1)),
                    container(black_btn).width(Length::FillPortion(1)),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXS);

                move_rows = move_rows.push(row_item);
            }
        } else if game.is_some() {
            let num_moves = (self.cached_moves.len() + 1) / 2;
            for turn in 0..num_moves {
                let white_idx = turn * 2;
                let black_idx = turn * 2 + 1;

                let num_label = text(format!("{}.", turn + 1))
                    .font(fonts::MONO)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_MUTED)
                    .width(Length::Fixed(32.0));

                let white_btn = if let Some(m) = self.cached_moves.get(white_idx) {
                    let is_active = self.current_move_idx == white_idx + 1;
                    let label = m.san.clone();
                    button(text(label).font(fonts::MONO).size(type_scale::BODY))
                        .padding([spacing::XXS, spacing::XS])
                        .style(move |_theme, _status| {
                            if is_active {
                                button::Style {
                                    background: Some(palette::ACCENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    border: border::rounded(spacing::RADIUS),
                                    ..Default::default()
                                }
                            } else {
                                button::Style {
                                    background: Some(Color::TRANSPARENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    ..Default::default()
                                }
                            }
                        })
                        .on_press(GameReviewMessage::GoToMove(white_idx + 1))
                } else {
                    button(text("").size(type_scale::BODY)).padding([spacing::XXS, spacing::XS])
                };

                let black_btn = if let Some(m) = self.cached_moves.get(black_idx) {
                    let is_active = self.current_move_idx == black_idx + 1;
                    let label = m.san.clone();
                    button(text(label).font(fonts::MONO).size(type_scale::BODY))
                        .padding([spacing::XXS, spacing::XS])
                        .style(move |_theme, _status| {
                            if is_active {
                                button::Style {
                                    background: Some(palette::ACCENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    border: border::rounded(spacing::RADIUS),
                                    ..Default::default()
                                }
                            } else {
                                button::Style {
                                    background: Some(Color::TRANSPARENT.into()),
                                    text_color: palette::TEXT_PRIMARY,
                                    ..Default::default()
                                }
                            }
                        })
                        .on_press(GameReviewMessage::GoToMove(black_idx + 1))
                } else {
                    button(text("").size(type_scale::BODY)).padding([spacing::XXS, spacing::XS])
                };

                let row_item = row![
                    num_label,
                    container(white_btn).width(Length::FillPortion(1)),
                    container(black_btn).width(Length::FillPortion(1)),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXS);

                move_rows = move_rows.push(row_item);
            }
        }

        let move_list_card = container(
            scrollable(move_rows)
                .height(Length::Fixed(190.0))
                .width(Length::Fill)
        )
        .padding(spacing::SM)
        .style(card_container_style);

        column![
            game_header,
            Space::with_height(spacing::XXS),
            accuracy_row,
            Space::with_height(spacing::XS),
            nav_section,
            Space::with_height(spacing::XS),
            eval_graph,
            Space::with_height(spacing::XS),
            current_move_detail,
            Space::with_height(spacing::XS),
            move_list_card,
        ]
        .spacing(spacing::XS)
        .width(Length::Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{AnalyzedMove, GameAnalysis, GamePhase, MoveClassification};
    use crate::engine::UciEval;
    use shakmaty::Square;

    fn sample_stored_game(pgn: &str) -> StoredGame {
        StoredGame {
            id: 42,
            pgn: pgn.to_string(),
            white: "Alice".to_string(),
            black: "Bob".to_string(),
            white_elo: Some(1500),
            black_elo: Some(1500),
            result: "1-0".to_string(),
            date: "2026.01.01".to_string(),
            time_control: "600".to_string(),
            eco: "C20".to_string(),
            opening: "King's Pawn".to_string(),
            accuracy_white: None,
            accuracy_black: None,
            analysis_json: None,
        }
    }

    #[test]
    fn test_load_game_caches_moves_and_resets_state() {
        let mut screen = GameReviewScreen::new();
        screen.current_move_idx = 3;
        screen.coach_explanation = Some("Old explanation".to_string());
        screen.analysis_error = Some("Old error".to_string());

        let game = sample_stored_game("1. e4 e5 2. Nf3 Nc6");
        screen.load_game(&game);

        assert_eq!(screen.current_move_idx, 0);
        assert!(screen.coach_explanation.is_none());
        assert!(screen.analysis_error.is_none());
        assert_eq!(screen.cached_moves.len(), 4);
        assert_eq!(screen.cached_moves[0].san, "e4");
        assert_eq!(screen.cached_moves[3].san, "Nc6");

        // Navigating via update_board_to_move uses cached moves and updates last_move
        screen.update_board_to_move(&game, 2);
        assert_eq!(screen.current_move_idx, 2);
        assert_eq!(screen.board_widget.last_move, Some((Square::E7, Square::E5)));
        assert!(screen.board_widget.best_move_arrow.is_none());
    }

    #[test]
    fn test_set_position_from_analysis_uses_eval_before_and_clears_stale_arrows() {
        let mut screen = GameReviewScreen::new();
        let parsed = crate::pgn::parse_pgn_games("1. e4 e5");
        let m1 = &parsed[0].moves[0];
        let m2 = &parsed[0].moves[1];

        let analysis = GameAnalysis {
            moves: vec![
                AnalyzedMove {
                    ply: 1,
                    move_number: 1,
                    is_white: true,
                    san: m1.san.clone(),
                    uci: m1.uci.clone(),
                    fen_before: m1.fen_before.clone(),
                    fen_after: m1.fen_after.clone(),
                    eval_before: Some(UciEval {
                        depth: 12,
                        score_cp: Some(20),
                        mate_in: None,
                        best_move: Some("d2d4".to_string()),
                        pv: vec![],
                    }),
                    eval_after: Some(UciEval {
                        depth: 12,
                        score_cp: Some(15),
                        mate_in: None,
                        best_move: Some("c7c5".to_string()),
                        pv: vec![],
                    }),
                    win_prob_before: 52.0,
                    win_prob_after: 51.0,
                    win_prob_loss: 1.0,
                    classification: MoveClassification::Best,
                    phase: GamePhase::Opening,
                    clock_seconds: None,
                },
                AnalyzedMove {
                    ply: 2,
                    move_number: 1,
                    is_white: false,
                    san: m2.san.clone(),
                    uci: m2.uci.clone(),
                    fen_before: m2.fen_before.clone(),
                    fen_after: m2.fen_after.clone(),
                    eval_before: None,
                    eval_after: Some(UciEval {
                        depth: 12,
                        score_cp: Some(25),
                        mate_in: None,
                        best_move: Some("g1f3".to_string()),
                        pv: vec![],
                    }),
                    win_prob_before: 51.0,
                    win_prob_after: 52.0,
                    win_prob_loss: 0.0,
                    classification: MoveClassification::Book,
                    phase: GamePhase::Opening,
                    clock_seconds: None,
                },
            ],
            white_accuracy: 95.0,
            black_accuracy: 95.0,
            white_blunders: 0,
            black_blunders: 0,
            white_mistakes: 0,
            black_mistakes: 0,
            white_inaccuracies: 0,
            black_inaccuracies: 0,
            opening_eco: "C20".to_string(),
            opening_name: "King's Pawn".to_string(),
            early_queen_white: false,
            early_queen_black: false,
            castled_white: None,
            castled_black: None,
        };

        // Ply 1 must show eval_before.best_move (d2d4), NOT eval_after.best_move (c7c5)
        screen.set_position_from_analysis(Some(&analysis), 1);
        assert_eq!(screen.board_widget.last_move, Some((Square::E2, Square::E4)));
        assert_eq!(screen.board_widget.best_move_arrow, Some((Square::D2, Square::D4)));

        // Ply 2 has eval_before: None -> best_move_arrow must be cleared, not stale d2d4
        screen.set_position_from_analysis(Some(&analysis), 2);
        assert_eq!(screen.board_widget.last_move, Some((Square::E7, Square::E5)));
        assert_eq!(screen.board_widget.best_move_arrow, None);

        // Ply 0 clears both last_move and best_move_arrow
        screen.set_position_from_analysis(Some(&analysis), 1);
        assert!(screen.board_widget.best_move_arrow.is_some());
        screen.set_position_from_analysis(Some(&analysis), 0);
        assert_eq!(screen.board_widget.last_move, None);
        assert_eq!(screen.board_widget.best_move_arrow, None);
    }
}

