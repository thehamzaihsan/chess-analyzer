use crate::insights::RatingPoint;
use crate::ui::theme::{fonts, palette, type_scale};
use iced::mouse;
use iced::widget::canvas::event::{Event, Status};
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

// ---------------------------------------------------------------------------
// Rating Progression Chart
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ChartMessage {
    PointHovered(Option<usize>),
    PointClicked(usize),
}

#[derive(Debug, Clone)]
pub struct RatingChartState {
    pub hovered_index: Option<usize>,
}

impl Default for RatingChartState {
    fn default() -> Self {
        Self {
            hovered_index: None,
        }
    }
}

pub struct RatingChart {
    pub points: Vec<RatingPoint>,
}

impl RatingChart {
    pub fn new(points: &[RatingPoint]) -> Self {
        Self {
            points: points.to_vec(),
        }
    }

    pub fn view<'a>(self, width: Length, height: Length) -> Element<'a, ChartMessage> {
        canvas::Canvas::new(self)
            .width(width)
            .height(height)
            .into()
    }
}

impl Program<ChartMessage> for RatingChart {
    type State = RatingChartState;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (Status, Option<ChartMessage>) {
        if self.points.is_empty() {
            return (Status::Ignored, None);
        }

        let padding_left = 60.0;
        let padding_right = 30.0;
        let _padding_top = 25.0;
        let _padding_bottom = 35.0;

        let plot_w = bounds.width - padding_left - padding_right;

        match event {
            Event::Mouse(mouse::Event::CursorMoved { position: _ }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if pos.x >= padding_left && pos.x <= bounds.width - padding_right {
                        let rel_x = (pos.x - padding_left) / plot_w;
                        let n = self.points.len();
                        let idx = if n <= 1 {
                            0
                        } else {
                            let raw = rel_x * (n - 1) as f32;
                            (raw.round() as usize).clamp(0, n - 1)
                        };

                        if state.hovered_index != Some(idx) {
                            state.hovered_index = Some(idx);
                            return (Status::Captured, Some(ChartMessage::PointHovered(Some(idx))));
                        }
                    } else if state.hovered_index.is_some() {
                        state.hovered_index = None;
                        return (Status::Captured, Some(ChartMessage::PointHovered(None)));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                if state.hovered_index.is_some() {
                    state.hovered_index = None;
                    return (Status::Captured, Some(ChartMessage::PointHovered(None)));
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(idx) = state.hovered_index {
                    return (Status::Captured, Some(ChartMessage::PointClicked(idx)));
                }
            }
            _ => {}
        }

        (Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background card
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            bounds.size(),
            palette::BG_SURFACE,
        );

        if self.points.is_empty() {
            let empty_text = Text {
                content: "No rating progression data available yet. Import games to view your rating curve.".to_string(),
                position: Point::new(bounds.width / 2.0, bounds.height / 2.0),
                color: palette::TEXT_MUTED,
                size: iced::Pixels(type_scale::BODY),
                font: fonts::UI,
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..Default::default()
            };
            frame.fill_text(empty_text);
            return vec![frame.into_geometry()];
        }

        let padding_left = 60.0;
        let padding_right = 30.0;
        let padding_top = 30.0;
        let padding_bottom = 40.0;

        let plot_w = bounds.width - padding_left - padding_right;
        let plot_h = bounds.height - padding_top - padding_bottom;

        if plot_w <= 10.0 || plot_h <= 10.0 {
            return vec![frame.into_geometry()];
        }

        // Compute rating range with padding
        let ratings: Vec<i32> = self.points.iter().map(|p| p.elo).collect();
        let min_r = ratings.iter().copied().min().unwrap_or(1200);
        let max_r = ratings.iter().copied().max().unwrap_or(1600);

        // Floor to multiple of 50, ceil to multiple of 50
        let floor_r = ((min_r - 25) / 50) * 50;
        let ceil_r = ((max_r + 74) / 50) * 50;
        let r_span = (ceil_r - floor_r).max(50) as f32;

        // Grid lines (every 50 or 100 points depending on span)
        let step = if r_span > 600.0 { 100 } else { 50 };
        let mut cur_grid = floor_r;
        while cur_grid <= ceil_r {
            let y_rel = 1.0 - (cur_grid - floor_r) as f32 / r_span;
            let y = padding_top + y_rel * plot_h;

            let line = Path::line(
                Point::new(padding_left, y),
                Point::new(bounds.width - padding_right, y),
            );
            frame.stroke(
                &line,
                Stroke::default()
                    .with_color(palette::BORDER)
                    .with_width(1.0),
            );

            // Rating label on left
            let label = Text {
                content: format!("{}", cur_grid),
                position: Point::new(padding_left - 8.0, y),
                color: palette::TEXT_MUTED,
                size: iced::Pixels(type_scale::CAPTION),
                font: fonts::MONO,
                horizontal_alignment: iced::alignment::Horizontal::Right,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..Default::default()
            };
            frame.fill_text(label);

            cur_grid += step;
        }

        let n = self.points.len();
        let to_screen_point = |idx: usize, elo: i32| -> Point {
            let x_rel = if n <= 1 {
                0.5
            } else {
                idx as f32 / (n - 1) as f32
            };
            let y_rel = 1.0 - (elo - floor_r) as f32 / r_span;
            Point::new(padding_left + x_rel * plot_w, padding_top + y_rel * plot_h)
        };

        // Filled area below line
        let mut area_builder = Builder::new();
        let first_pt = to_screen_point(0, self.points[0].elo);
        let bottom_y = padding_top + plot_h;

        area_builder.move_to(Point::new(first_pt.x, bottom_y));
        area_builder.line_to(first_pt);

        for (i, p) in self.points.iter().enumerate().skip(1) {
            let pt = to_screen_point(i, p.elo);
            area_builder.line_to(pt);
        }

        let last_pt = to_screen_point(n - 1, self.points[n - 1].elo);
        area_builder.line_to(Point::new(last_pt.x, bottom_y));
        area_builder.close();

        let area_path = area_builder.build();
        let area_color = Color::from_rgba(56.0 / 255.0, 139.0 / 255.0, 253.0 / 255.0, 0.08);
        frame.fill(&area_path, area_color);

        // Line curve
        let mut line_builder = Builder::new();
        line_builder.move_to(first_pt);
        for (i, p) in self.points.iter().enumerate().skip(1) {
            let pt = to_screen_point(i, p.elo);
            line_builder.line_to(pt);
        }
        let line_path = line_builder.build();
        frame.stroke(
            &line_path,
            Stroke::default()
                .with_color(palette::ACCENT)
                .with_width(2.0),
        );

        // Individual points / dots
        for (i, p) in self.points.iter().enumerate() {
            let pt = to_screen_point(i, p.elo);
            let dot_color = match p.result.as_str() {
                "win" => palette::SUCCESS,
                "loss" => palette::DANGER,
                _ => palette::TEXT_MUTED,
            };

            let is_hovered = state.hovered_index == Some(i);
            let radius = if is_hovered { 6.0 } else { 3.5 };

            let circle = Path::circle(pt, radius);
            frame.fill(&circle, dot_color);

            if is_hovered {
                frame.stroke(
                    &circle,
                    Stroke::default()
                        .with_color(palette::TEXT_PRIMARY)
                        .with_width(2.0),
                );
            }
        }

        // Date labels on X-axis (first, middle, last)
        let date_indices = if n >= 3 {
            vec![0, n / 2, n - 1]
        } else if n == 2 {
            vec![0, 1]
        } else {
            vec![0]
        };

        for idx in date_indices {
            if let Some(p) = self.points.get(idx) {
                let pt = to_screen_point(idx, p.elo);
                let halign = if idx == 0 {
                    iced::alignment::Horizontal::Left
                } else if idx == n - 1 {
                    iced::alignment::Horizontal::Right
                } else {
                    iced::alignment::Horizontal::Center
                };

                let date_label = Text {
                    content: p.date.clone(),
                    position: Point::new(pt.x, padding_top + plot_h + 10.0),
                    color: palette::TEXT_MUTED,
                    size: iced::Pixels(type_scale::CAPTION),
                    font: fonts::UI,
                    horizontal_alignment: halign,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    ..Default::default()
                };
                frame.fill_text(date_label);
            }
        }

        // Hover Card / Tooltip
        if let Some(hovered_idx) = state.hovered_index {
            if let Some(p) = self.points.get(hovered_idx) {
                let pt = to_screen_point(hovered_idx, p.elo);

                // Vertical crosshair indicator
                let crosshair = Path::line(
                    Point::new(pt.x, padding_top),
                    Point::new(pt.x, padding_top + plot_h),
                );
                frame.stroke(
                    &crosshair,
                    Stroke::default()
                        .with_color(Color::from_rgba(255.0, 255.0, 255.0, 0.2))
                        .with_width(1.0),
                );

                // Tooltip box dimensions
                let card_w = 170.0;
                let card_h = 74.0;
                let card_x = if pt.x + card_w + 15.0 <= bounds.width {
                    pt.x + 12.0
                } else {
                    pt.x - card_w - 12.0
                };
                let card_y = (pt.y - card_h / 2.0)
                    .clamp(padding_top, bounds.height - padding_bottom - card_h);

                // Tooltip background
                let tooltip_rect = Path::rectangle(Point::new(card_x, card_y), Size::new(card_w, card_h));
                frame.fill(&tooltip_rect, palette::BG_ELEVATED);
                frame.stroke(
                    &tooltip_rect,
                    Stroke::default()
                        .with_color(palette::BORDER)
                        .with_width(1.0),
                );

                // Tooltip content: Elo, Date, Opponent, Result
                let elo_text = Text {
                    content: format!("{} Elo", p.elo),
                    position: Point::new(card_x + 8.0, card_y + 8.0),
                    color: palette::TEXT_PRIMARY,
                    size: iced::Pixels(type_scale::HEADING),
                    font: fonts::MONO_BOLD,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    ..Default::default()
                };
                frame.fill_text(elo_text);

                let res_color = match p.result.as_str() {
                    "win" => palette::SUCCESS,
                    "loss" => palette::DANGER,
                    _ => palette::TEXT_MUTED,
                };
                let res_label = match p.result.as_str() {
                    "win" => "Victory",
                    "loss" => "Defeat",
                    _ => "Draw",
                };

                let res_text = Text {
                    content: res_label.to_string(),
                    position: Point::new(card_x + card_w - 8.0, card_y + 8.0),
                    color: res_color,
                    size: iced::Pixels(type_scale::BODY),
                    font: fonts::UI_SEMIBOLD,
                    horizontal_alignment: iced::alignment::Horizontal::Right,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    ..Default::default()
                };
                frame.fill_text(res_text);

                let opp_str = if p.opponent.is_empty() {
                    p.date.clone()
                } else if let Some(elo) = p.opponent_elo {
                    format!("vs {} ({})", p.opponent, elo)
                } else {
                    format!("vs {}", p.opponent)
                };

                let opp_text = Text {
                    content: opp_str,
                    position: Point::new(card_x + 8.0, card_y + 32.0),
                    color: palette::TEXT_SECONDARY,
                    size: iced::Pixels(type_scale::CAPTION),
                    font: fonts::UI,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    ..Default::default()
                };
                frame.fill_text(opp_text);

                let date_sub = Text {
                    content: p.date.clone(),
                    position: Point::new(card_x + 8.0, card_y + 50.0),
                    color: palette::TEXT_MUTED,
                    size: iced::Pixels(type_scale::CAPTION),
                    font: fonts::UI,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    ..Default::default()
                };
                frame.fill_text(date_sub);
            }
        }

        vec![frame.into_geometry()]
    }
}

// ---------------------------------------------------------------------------
// Move-by-Move Evaluation / Advantage Graph
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum EvalGraphMessage {
    SelectPly(usize),
}

pub struct EvalGraphWidget {
    pub evaluations: Vec<Option<f32>>, // Centipawns clamped (-10.0..+10.0) or Win% (0.0..1.0)
    pub current_ply: usize,
}

impl EvalGraphWidget {
    pub fn new(evaluations: Vec<Option<f32>>, current_ply: usize) -> Self {
        Self {
            evaluations,
            current_ply,
        }
    }

    pub fn view<'a>(self, width: Length, height: Length) -> Element<'a, EvalGraphMessage> {
        canvas::Canvas::new(self)
            .width(width)
            .height(height)
            .into()
    }
}

impl Program<EvalGraphMessage> for EvalGraphWidget {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (Status, Option<EvalGraphMessage>) {
        if self.evaluations.is_empty() {
            return (Status::Ignored, None);
        }

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(pos) = cursor.position_in(bounds) {
                let n = self.evaluations.len();
                let rel_x = (pos.x / bounds.width).clamp(0.0, 1.0);
                let ply = ((rel_x * n as f32).floor() as usize).clamp(0, n);
                return (Status::Captured, Some(EvalGraphMessage::SelectPly(ply)));
            }
        }

        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            bounds.size(),
            palette::BG_SURFACE,
        );

        let mid_y = bounds.height / 2.0;

        // Baseline (0.0 advantage)
        let baseline = Path::line(Point::new(0.0, mid_y), Point::new(bounds.width, mid_y));
        frame.stroke(
            &baseline,
            Stroke::default()
                .with_color(palette::BORDER)
                .with_width(1.0),
        );

        if self.evaluations.is_empty() {
            return vec![frame.into_geometry()];
        }

        let n = self.evaluations.len();
        let step_x = bounds.width / n.max(1) as f32;

        // Draw eval curve and area fills
        for (i, eval_opt) in self.evaluations.iter().enumerate() {
            let win_pct = eval_opt.unwrap_or(0.5); // 0.0 to 1.0
            let x = i as f32 * step_x;
            let bar_w = step_x.max(1.0);

            // Win pct > 0.5 = White advantage, < 0.5 = Black advantage
            if win_pct >= 0.5 {
                let h = ((win_pct - 0.5) * 2.0) * mid_y;
                let top_y = mid_y - h;
                let white_advantage_color = Color::from_rgba(218.0 / 255.0, 224.0 / 255.0, 233.0 / 255.0, 0.85);
                frame.fill_rectangle(
                    Point::new(x, top_y),
                    Size::new(bar_w, h),
                    white_advantage_color,
                );
            } else {
                let h = ((0.5 - win_pct) * 2.0) * mid_y;
                let black_advantage_color = Color::from_rgba(33.0 / 255.0, 38.0 / 255.0, 45.0 / 255.0, 0.95);
                frame.fill_rectangle(
                    Point::new(x, mid_y),
                    Size::new(bar_w, h),
                    black_advantage_color,
                );
            }
        }

        // Draw cursor line for current move
        if self.current_ply <= n {
            let cur_x = if n > 0 {
                (self.current_ply as f32 / n as f32) * bounds.width
            } else {
                0.0
            };

            let cur_line = Path::line(Point::new(cur_x, 0.0), Point::new(cur_x, bounds.height));
            frame.stroke(
                &cur_line,
                Stroke::default()
                    .with_color(palette::ACCENT)
                    .with_width(2.0),
            );
        }

        vec![frame.into_geometry()]
    }
}
