use crate::analysis::GameAnalysis;
use crate::db::StoredGame;
use crate::ui::board::{BoardMessage, BoardWidget};
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
}

impl GameReviewScreen {
    pub fn new() -> Self {
        Self {
            current_move_idx: 0,
            board_widget: BoardWidget::new(Chess::default()),
            coach_explanation: None,
            is_coach_loading: false,
        }
    }

    pub fn load_game(&mut self, _game: &StoredGame) {
        self.current_move_idx = 0;
        self.board_widget = BoardWidget::new(Chess::default());
        self.coach_explanation = None;
        self.is_coach_loading = false;
    }

    pub fn update_board_to_move(&mut self, game: &StoredGame, idx: usize) {
        self.current_move_idx = idx;
        self.coach_explanation = None;

        let parsed = crate::pgn::parse_pgn_games(&game.pgn);
        let mut pos = Chess::default();
        let mut last_m = None;

        if let Some(first_g) = parsed.first() {
            let target_moves = &first_g.moves[..idx.min(first_g.moves.len())];
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
        }

        self.board_widget.position = pos;
        self.board_widget.last_move = last_m;
        self.board_widget.best_move_arrow = None;
    }

    pub fn set_position_from_analysis(&mut self, analysis: Option<&GameAnalysis>, ply: usize) {
        self.current_move_idx = ply;
        self.coach_explanation = None;

        if let Some(an) = analysis {
            if ply == 0 {
                self.board_widget.position = Chess::default();
                self.board_widget.last_move = None;
                self.board_widget.best_move_arrow = None;
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

                if let Some(eval) = &m.eval_after {
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
        let board_size = 520.0;
        let board_el = self.board_widget.view(board_size).map(GameReviewMessage::Board);

        // Cohesive dark-theme evaluation bar
        let eval_bar = self.render_eval_bar(analysis, board_size);

        let board_and_bar = row![eval_bar, Space::with_width(spacing::SM), board_el]
            .align_y(Alignment::Center);

        // Right side info & controls
        let sidebar = self.render_sidebar(game, analysis);

        let content = row![
            board_and_bar,
            Space::with_width(spacing::LG),
            sidebar,
        ]
        .padding(spacing::MD)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content).into()
    }

    fn render_eval_bar<'a>(&'a self, analysis: Option<&'a GameAnalysis>, height: f32) -> Element<'a, GameReviewMessage> {
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
        let white_height = (height * white_pct).clamp(0.0, height);
        let black_height = height - white_height;

        let black_part = container(Space::new(Length::Fixed(bar_width), Length::Fixed(black_height)))
            .style(|_theme| container::Style {
                background: Some(palette::BG_SURFACE.into()),
                ..Default::default()
            });

        let white_color = Color::from_rgb8(218, 224, 233);
        let white_part = container(
            container(
                text(score_text)
                    .font(fonts::MONO_BOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::BG_BASE)
            )
            .padding(spacing::XXS)
            .align_x(Alignment::Center)
            .align_y(Alignment::End)
        )
        .width(Length::Fixed(bar_width))
        .height(Length::Fixed(white_height))
        .style(move |_theme| container::Style {
            background: Some(white_color.into()),
            ..Default::default()
        });

        container(column![black_part, white_part])
            .width(Length::Fixed(bar_width))
            .height(Length::Fixed(height))
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
            None => row![
                text("Game not analyzed yet.")
                    .font(fonts::UI)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_MUTED),
                Space::with_width(spacing::SM),
                button(text("Analyze with Engine").font(fonts::UI_SEMIBOLD).size(type_scale::BODY))
                    .padding([spacing::XS, spacing::SM])
                    .style(primary_button_style)
                    .on_press(GameReviewMessage::AnalyzeCurrentGame),
            ]
            .align_y(Alignment::Center)
            .into(),
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
            nav_controls,
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
