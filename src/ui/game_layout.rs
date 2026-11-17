use crate::ui::board::{captured_pieces, captured_tray, BoardMessage, BoardWidget};
use crate::ui::theme::{
    fonts, icon_flag, icon_handshake, icon_trophy, icon_widget, palette, spacing, truncate,
    type_scale, with_tooltip,
};
use iced::border;
use iced::widget::{column, container, row, text, tooltip, Space};
use iced::{Alignment, Color, Element, Length};
use shakmaty::{Chess, Color as PieceColor, Role};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerResult {
    Win,
    Loss,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeTone {
    Win,
    Loss,
    Draw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOutcome {
    pub result: String, // "1-0", "0-1", "1/2-1/2"
    pub reason: String,
}

impl GameOutcome {
    pub fn new(result: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            result: result.into(),
            reason: reason.into(),
        }
    }

    pub fn display_result(&self) -> &str {
        if self.result == "1/2-1/2" {
            "½-½"
        } else {
            &self.result
        }
    }

    pub fn is_draw(&self) -> bool {
        self.result == "1/2-1/2"
    }

    pub fn player_result(&self, is_white: bool) -> Option<PlayerResult> {
        match self.result.as_str() {
            "1-0" => Some(if is_white {
                PlayerResult::Win
            } else {
                PlayerResult::Loss
            }),
            "0-1" => Some(if is_white {
                PlayerResult::Loss
            } else {
                PlayerResult::Win
            }),
            "1/2-1/2" => Some(PlayerResult::Draw),
            _ => None,
        }
    }

    pub fn default_tone(&self) -> OutcomeTone {
        if self.is_draw() {
            OutcomeTone::Draw
        } else {
            OutcomeTone::Win
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub rating: Option<i32>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub is_white: bool,
    pub is_active_turn: bool,
    pub result: Option<PlayerResult>,
}

impl PlayerInfo {
    pub fn new(name: impl Into<String>, is_white: bool) -> Self {
        Self {
            name: name.into(),
            rating: None,
            title: None,
            status: None,
            is_white,
            is_active_turn: false,
            result: None,
        }
    }

    pub fn with_rating(mut self, rating: Option<i32>) -> Self {
        self.rating = rating;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_active_turn(mut self, active: bool) -> Self {
        self.is_active_turn = active;
        self
    }

    pub fn with_result(mut self, result: Option<PlayerResult>) -> Self {
        self.result = result;
        self
    }
}

fn render_player_result_badge<'a, Message: 'a>(res: PlayerResult) -> Element<'a, Message> {
    let (bg, border_color, text_color, icon, score_label) = match res {
        PlayerResult::Win => (
            Color::from_rgba(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0, 0.22),
            palette::SUCCESS,
            Color::from_rgb8(116, 224, 138),
            icon_trophy(),
            "1",
        ),
        PlayerResult::Loss => (
            Color::from_rgba(218.0 / 255.0, 54.0 / 255.0, 51.0 / 255.0, 0.18),
            Color::from_rgba(218.0 / 255.0, 54.0 / 255.0, 51.0 / 255.0, 0.55),
            Color::from_rgb8(242, 139, 137),
            icon_flag(),
            "0",
        ),
        PlayerResult::Draw => (
            Color::from_rgba(210.0 / 255.0, 153.0 / 255.0, 34.0 / 255.0, 0.20),
            Color::from_rgba(210.0 / 255.0, 153.0 / 255.0, 34.0 / 255.0, 0.55),
            Color::from_rgb8(239, 193, 98),
            icon_handshake(),
            "½",
        ),
    };

    container(
        row![
            icon_widget(icon, 11.0, Some(text_color)),
            text(score_label)
                .font(fonts::MONO_BOLD)
                .size(11.0)
                .color(text_color),
        ]
        .spacing(3.0)
        .align_y(Alignment::Center),
    )
    .padding([2.0, 6.0])
    .style(move |_theme| container::Style {
        background: Some(bg.into()),
        border: border::rounded(4.0).width(1.0).color(border_color),
        ..Default::default()
    })
    .into()
}

pub fn render_result_banner<'a, Message: 'a>(outcome: &GameOutcome) -> Element<'a, Message> {
    render_result_banner_with_tone(outcome, outcome.default_tone())
}

pub fn render_result_banner_with_tone<'a, Message: 'a>(
    outcome: &GameOutcome,
    tone: OutcomeTone,
) -> Element<'a, Message> {
    let (bg, border_color, accent_color, icon) = match tone {
        OutcomeTone::Win => (
            Color::from_rgba(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0, 0.16),
            Color::from_rgba(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0, 0.50),
            Color::from_rgb8(116, 224, 138),
            icon_trophy(),
        ),
        OutcomeTone::Loss => (
            Color::from_rgba(218.0 / 255.0, 54.0 / 255.0, 51.0 / 255.0, 0.16),
            Color::from_rgba(218.0 / 255.0, 54.0 / 255.0, 51.0 / 255.0, 0.50),
            Color::from_rgb8(242, 139, 137),
            icon_flag(),
        ),
        OutcomeTone::Draw => (
            Color::from_rgba(210.0 / 255.0, 153.0 / 255.0, 34.0 / 255.0, 0.16),
            Color::from_rgba(210.0 / 255.0, 153.0 / 255.0, 34.0 / 255.0, 0.45),
            Color::from_rgb8(239, 193, 98),
            icon_handshake(),
        ),
    };

    let score_pill = container(
        text(outcome.display_result().to_string())
            .font(fonts::MONO_BOLD)
            .size(type_scale::BODY)
            .color(accent_color),
    )
    .padding([2.0, 8.0])
    .style(move |_theme| container::Style {
        background: Some(palette::BG_BASE.into()),
        border: border::rounded(4.0).width(1.0).color(border_color),
        ..Default::default()
    });

    let reason_text = text(outcome.reason.clone())
        .font(fonts::UI_SEMIBOLD)
        .size(type_scale::BODY)
        .color(palette::TEXT_PRIMARY)
        .wrapping(text::Wrapping::WordOrGlyph)
        .width(Length::Fill);

    container(
        row![
            icon_widget(icon, 16.0, Some(accent_color)),
            score_pill,
            reason_text,
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding([spacing::XS, spacing::SM])
    .width(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(bg.into()),
        border: border::rounded(spacing::RADIUS)
            .width(1.0)
            .color(border_color),
        ..Default::default()
    })
    .into()
}

fn render_player_bar_with_width<'a, Message: 'a>(
    player: &PlayerInfo,
    captured: &[Role],
    captured_color: PieceColor,
    material_lead: Option<i32>,
    bar_width: Length,
) -> Element<'a, Message> {
    // Avatar / color indicator badge
    let avatar_bg = if player.is_white {
        Color::from_rgb8(235, 238, 242)
    } else {
        Color::from_rgb8(35, 39, 47)
    };
    let avatar_border = if player.is_white {
        Color::from_rgb8(180, 185, 195)
    } else {
        Color::from_rgb8(70, 78, 92)
    };
    let avatar_text_color = if player.is_white {
        Color::from_rgb8(30, 35, 45)
    } else {
        Color::from_rgb8(240, 242, 245)
    };

    let avatar_circle = container(
        text(if player.is_white { "W" } else { "B" })
            .font(fonts::UI_BOLD)
            .size(11.0)
            .color(avatar_text_color),
    )
    .width(Length::Fixed(24.0))
    .height(Length::Fixed(24.0))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_theme| container::Style {
        background: Some(avatar_bg.into()),
        border: border::rounded(12.0).width(1.5).color(avatar_border),
        ..Default::default()
    });

    let short_name = truncate(&player.name, 20);
    let name_text = text(short_name.clone())
        .font(fonts::UI_BOLD)
        .size(type_scale::BODY)
        .color(palette::TEXT_PRIMARY)
        .wrapping(text::Wrapping::None);
    let name_el: Element<'a, Message> = if short_name != player.name {
        with_tooltip(name_text, &player.name, tooltip::Position::Top)
    } else {
        name_text.into()
    };

    let mut info_row = row![
        avatar_circle,
        Space::with_width(spacing::XS),
        name_el,
    ]
    .align_y(Alignment::Center);

    if let Some(ref title) = player.title {
        let short_title = truncate(title, 12);
        let title_badge = container(
            text(short_title.clone())
                .font(fonts::MONO_BOLD)
                .size(10.0)
                .color(palette::TEXT_PRIMARY)
                .wrapping(text::Wrapping::None),
        )
        .padding([2.0, 5.0])
        .style(|_theme| container::Style {
            background: Some(palette::ACCENT.into()),
            border: border::rounded(4.0),
            ..Default::default()
        });
        let title_el: Element<'a, Message> = if short_title != *title {
            with_tooltip(title_badge, title, tooltip::Position::Top)
        } else {
            title_badge.into()
        };
        info_row = info_row.push(Space::with_width(spacing::XXS)).push(title_el);
    }

    if let Some(rating) = player.rating {
        let rating_text = text(format!("({})", rating))
            .font(fonts::MONO_BOLD)
            .size(type_scale::CAPTION)
            .color(palette::TEXT_SECONDARY)
            .wrapping(text::Wrapping::None);
        info_row = info_row.push(Space::with_width(spacing::XXS)).push(rating_text);
    }

    if player.is_active_turn && player.result.is_none() {
        let turn_dot = container(Space::with_width(Length::Fixed(8.0)).height(Length::Fixed(8.0)))
            .style(|_theme| container::Style {
                background: Some(palette::SUCCESS.into()),
                border: border::rounded(4.0),
                ..Default::default()
            });
        info_row = info_row.push(Space::with_width(spacing::XS)).push(turn_dot);
    }

    if let Some(ref status) = player.status {
        let short_status = truncate(status, 18);
        let status_lbl = text(short_status.clone())
            .font(fonts::UI)
            .size(type_scale::CAPTION)
            .color(palette::TEXT_MUTED)
            .wrapping(text::Wrapping::None);
        let status_el: Element<'a, Message> = if short_status != *status {
            with_tooltip(status_lbl, status, tooltip::Position::Top)
        } else {
            status_lbl.into()
        };
        info_row = info_row.push(Space::with_width(spacing::XS)).push(status_el);
    }

    let tray = captured_tray(captured, captured_color, material_lead, 16.0);

    let mut right_row = row![container(tray).clip(true)]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

    if let Some(res) = player.result {
        right_row = right_row.push(render_player_result_badge(res));
    }

    let content = row![
        container(info_row).width(Length::Fill).clip(true),
        Space::with_width(spacing::XS),
        container(right_row).clip(true),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .padding([6.0, 10.0])
        .width(bar_width)
        .clip(true)
        .style(|_theme| container::Style {
            background: Some(palette::BG_SURFACE.into()),
            border: border::rounded(6.0).width(1.0).color(palette::BORDER_SUBTLE),
            ..Default::default()
        })
        .into()
}

/// Renders a Chess.com-style Player Bar above or below the board:
/// [Avatar/Color Badge] [Player Name] [Title] [Elo Rating] [Status / Active Indicator]  ...  [Captured Pieces Tray] [+Lead] [Result Badge]
pub fn render_player_bar<'a, Message: 'a>(
    player: &PlayerInfo,
    captured: &[Role],
    captured_color: PieceColor,
    material_lead: Option<i32>,
    bar_width: f32,
) -> Element<'a, Message> {
    render_player_bar_with_width(
        player,
        captured,
        captured_color,
        material_lead,
        Length::Fixed(bar_width),
    )
}

pub const EVAL_BAR_WIDTH: f32 = 34.0;
pub const EVAL_BAR_GAP: f32 = 8.0;
pub const WORKSPACE_PAD_X: f32 = spacing::SM;
pub const WORKSPACE_PAD_Y: f32 = spacing::XS;
pub const WORKSPACE_COL_GAP: f32 = spacing::MD;
pub const PLAYER_BARS_RESERVED_H: f32 = 96.0;
pub const SIDE_PANEL_MIN_W: f32 = 320.0;
pub const SIDE_PANEL_MAX_W: f32 = 480.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoardLayoutMetrics {
    pub board_side: f32,
    pub eval_total_w: f32,
    pub board_col_w: f32,
    pub side_panel_w: f32,
}

/// Computes the largest square board size that fits within `avail` while guaranteeing
/// the right side panel stays within `[SIDE_PANEL_MIN_W, SIDE_PANEL_MAX_W]`.
pub fn compute_board_layout(avail: iced::Size, has_eval_bar: bool) -> BoardLayoutMetrics {
    let inner_w = (avail.width - WORKSPACE_PAD_X * 2.0).max(400.0);
    let inner_h = (avail.height - WORKSPACE_PAD_Y * 2.0).max(320.0);
    let eval_total_w = if has_eval_bar {
        EVAL_BAR_WIDTH + EVAL_BAR_GAP
    } else {
        0.0
    };

    let max_board_by_h = (inner_h - PLAYER_BARS_RESERVED_H).max(240.0);
    let max_board_by_w =
        (inner_w - WORKSPACE_COL_GAP - SIDE_PANEL_MIN_W - eval_total_w).max(240.0);
    let board_side = max_board_by_h.min(max_board_by_w).floor().max(240.0);
    let board_col_w = board_side + eval_total_w;
    let side_panel_w = (inner_w - WORKSPACE_COL_GAP - board_col_w)
        .clamp(SIDE_PANEL_MIN_W, SIDE_PANEL_MAX_W)
        .floor();

    BoardLayoutMetrics {
        board_side,
        eval_total_w,
        board_col_w,
        side_panel_w,
    }
}

/// Renders the Chess.com-style Board Column with an exact square `board_side`, matching
/// player bar widths (`board_side + eval_bar`), and matching eval bar height.
pub fn render_sized_board_column<'a, Message: 'a>(
    top_player: &PlayerInfo,
    bottom_player: &PlayerInfo,
    position: &Chess,
    board_widget: &'a BoardWidget,
    map_board_message: impl Fn(BoardMessage) -> Message + 'a,
    eval_bar: Option<Element<'a, Message>>,
    board_side: f32,
) -> Element<'a, Message> {
    let (captured_by_white, captured_by_black, white_advantage) = captured_pieces(position);

    let (top_captured, top_captured_color, top_lead) = if board_widget.flipped {
        (
            &captured_by_white[..],
            PieceColor::Black,
            if white_advantage > 0 { Some(white_advantage) } else { None },
        )
    } else {
        (
            &captured_by_black[..],
            PieceColor::White,
            if white_advantage < 0 { Some(-white_advantage) } else { None },
        )
    };

    let (bottom_captured, bottom_captured_color, bottom_lead) = if board_widget.flipped {
        (
            &captured_by_black[..],
            PieceColor::White,
            if white_advantage < 0 { Some(-white_advantage) } else { None },
        )
    } else {
        (
            &captured_by_white[..],
            PieceColor::Black,
            if white_advantage > 0 { Some(white_advantage) } else { None },
        )
    };

    let eval_total_w = if eval_bar.is_some() {
        EVAL_BAR_WIDTH + EVAL_BAR_GAP
    } else {
        0.0
    };
    let bar_width = board_side + eval_total_w;

    let top_bar = render_player_bar(
        top_player,
        top_captured,
        top_captured_color,
        top_lead,
        bar_width,
    );
    let bottom_bar = render_player_bar(
        bottom_player,
        bottom_captured,
        bottom_captured_color,
        bottom_lead,
        bar_width,
    );

    let board_el = board_widget.view(board_side).map(map_board_message);

    let board_row = if let Some(bar) = eval_bar {
        let bar_box = container(bar)
            .width(Length::Fixed(EVAL_BAR_WIDTH))
            .height(Length::Fixed(board_side))
            .clip(true);
        row![bar_box, Space::with_width(EVAL_BAR_GAP), board_el]
            .align_y(Alignment::Center)
    } else {
        row![board_el].align_y(Alignment::Center)
    };

    column![
        top_bar,
        Space::with_height(spacing::XS),
        board_row,
        Space::with_height(spacing::XS),
        bottom_bar,
    ]
    .width(Length::Fixed(bar_width))
    .align_x(Alignment::Start)
    .into()
}

/// Places the sized board column alongside the right side panel, centered cleanly when extra
/// horizontal room exceeds `SIDE_PANEL_MAX_W`.
pub fn render_board_workspace<'a, Message: 'a>(
    board_column: Element<'a, Message>,
    side_panel: Element<'a, Message>,
    metrics: BoardLayoutMetrics,
) -> Element<'a, Message> {
    let side_container = container(side_panel)
        .width(Length::Fixed(metrics.side_panel_w))
        .height(Length::Fill)
        .clip(true);

    let workspace = row![
        board_column,
        Space::with_width(WORKSPACE_COL_GAP),
        side_container,
    ]
    .height(Length::Fill)
    .align_y(Alignment::Center);

    container(workspace)
        .padding([WORKSPACE_PAD_Y, WORKSPACE_PAD_X])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .clip(true)
        .into()
}

/// Renders the unified Chess.com Board Column:
/// - Top player bar
/// - Center: [Optional Eval Bar] + [Responsive Chess Board]
/// - Bottom player bar
pub fn render_unified_board_column<'a, Message: 'a>(
    top_player: &PlayerInfo,
    bottom_player: &PlayerInfo,
    position: &Chess,
    board_widget: &'a BoardWidget,
    map_board_message: impl Fn(BoardMessage) -> Message + 'a,
    eval_bar: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let (captured_by_white, captured_by_black, white_advantage) = captured_pieces(position);

    let (top_captured, top_captured_color, top_lead) = if board_widget.flipped {
        // Flipped: Black is at bottom, White is at top
        (
            &captured_by_white[..],
            PieceColor::Black,
            if white_advantage > 0 { Some(white_advantage) } else { None },
        )
    } else {
        // Standard: White is at bottom, Black is at top
        (
            &captured_by_black[..],
            PieceColor::White,
            if white_advantage < 0 { Some(-white_advantage) } else { None },
        )
    };

    let (bottom_captured, bottom_captured_color, bottom_lead) = if board_widget.flipped {
        // Flipped: Black is at bottom
        (
            &captured_by_black[..],
            PieceColor::White,
            if white_advantage < 0 { Some(-white_advantage) } else { None },
        )
    } else {
        // Standard: White is at bottom
        (
            &captured_by_white[..],
            PieceColor::Black,
            if white_advantage > 0 { Some(white_advantage) } else { None },
        )
    };

    let top_bar = render_player_bar_with_width(
        top_player,
        top_captured,
        top_captured_color,
        top_lead,
        Length::Fill,
    );
    let bottom_bar = render_player_bar_with_width(
        bottom_player,
        bottom_captured,
        bottom_captured_color,
        bottom_lead,
        Length::Fill,
    );

    let board_element = container(
        board_widget.view_responsive().map(map_board_message)
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let gap = 8.0;
    let board_row = if let Some(bar) = eval_bar {
        let bar_container = container(bar)
            .height(Length::Shrink)
            .clip(true);
        row![bar_container, Space::with_width(gap), board_element]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Center)
    } else {
        row![board_element]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Center)
    };

    column![
        top_bar,
        Space::with_height(spacing::XS),
        board_row,
        Space::with_height(spacing::XS),
        bottom_bar,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_board_layout_fits_1040x700_and_1280x840() {
        // Minimum window 1040x700 minus expanded sidebar (~225) and header/status (~84) and shell pad (16)
        let min_avail = iced::Size::new(799.0, 600.0);
        let m_min = compute_board_layout(min_avail, true);
        assert!(m_min.board_side >= 380.0);
        assert!(m_min.side_panel_w >= SIDE_PANEL_MIN_W);
        assert!(m_min.side_panel_w <= SIDE_PANEL_MAX_W);
        assert!(
            m_min.board_col_w + WORKSPACE_COL_GAP + m_min.side_panel_w + WORKSPACE_PAD_X * 2.0
                <= min_avail.width + 0.1
        );
        assert!(
            m_min.board_side + PLAYER_BARS_RESERVED_H + WORKSPACE_PAD_Y * 2.0 <= min_avail.height + 0.1
        );

        // Standard 1280x840 window expands board generously
        let std_avail = iced::Size::new(1039.0, 740.0);
        let m_std = compute_board_layout(std_avail, false);
        assert!(m_std.board_side >= 600.0);
        assert_eq!(m_std.board_col_w, m_std.board_side);
        assert!(m_std.side_panel_w >= SIDE_PANEL_MIN_W);
        assert!(m_std.side_panel_w <= SIDE_PANEL_MAX_W);

        // Wide window caps side panel width at SIDE_PANEL_MAX_W
        let wide_avail = iced::Size::new(1600.0, 900.0);
        let m_wide = compute_board_layout(wide_avail, true);
        assert_eq!(m_wide.side_panel_w, SIDE_PANEL_MAX_W);
        assert_eq!(m_wide.board_col_w, m_wide.board_side + EVAL_BAR_WIDTH + EVAL_BAR_GAP);
    }
}
