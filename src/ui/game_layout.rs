use crate::ui::board::{captured_pieces, captured_tray, BoardMessage, BoardWidget};
use crate::ui::theme::{fonts, palette, spacing, type_scale};
use iced::border;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};
use shakmaty::{Chess, Color as PieceColor, Role};

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub rating: Option<i32>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub is_white: bool,
    pub is_active_turn: bool,
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
}

/// Renders a Chess.com-style Player Bar above or below the board:
/// [Avatar/Color Badge] [Player Name] [Title] [Elo Rating] [Status / Active Indicator]  ...  [Captured Pieces Tray] [+Lead]
pub fn render_player_bar<'a, Message: 'a>(
    player: &PlayerInfo,
    captured: &[Role],
    captured_color: PieceColor,
    material_lead: Option<i32>,
    bar_width: f32,
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

    let mut info_row = row![
        avatar_circle,
        Space::with_width(spacing::XS),
        text(player.name.clone())
            .font(fonts::UI_BOLD)
            .size(type_scale::BODY)
            .color(palette::TEXT_PRIMARY),
    ]
    .align_y(Alignment::Center);

    if let Some(ref title) = player.title {
        let title_badge = container(
            text(title.clone())
                .font(fonts::MONO_BOLD)
                .size(10.0)
                .color(palette::TEXT_PRIMARY),
        )
        .padding([2.0, 5.0])
        .style(|_theme| container::Style {
            background: Some(palette::ACCENT.into()),
            border: border::rounded(4.0),
            ..Default::default()
        });
        info_row = info_row.push(Space::with_width(spacing::XXS)).push(title_badge);
    }

    if let Some(rating) = player.rating {
        let rating_text = text(format!("({})", rating))
            .font(fonts::MONO_BOLD)
            .size(type_scale::CAPTION)
            .color(palette::TEXT_SECONDARY);
        info_row = info_row.push(Space::with_width(spacing::XXS)).push(rating_text);
    }

    if player.is_active_turn {
        let turn_dot = container(Space::with_width(Length::Fixed(8.0)).height(Length::Fixed(8.0)))
            .style(|_theme| container::Style {
                background: Some(palette::SUCCESS.into()),
                border: border::rounded(4.0),
                ..Default::default()
            });
        info_row = info_row.push(Space::with_width(spacing::XS)).push(turn_dot);
    }

    if let Some(ref status) = player.status {
        let status_lbl = text(status.clone())
            .font(fonts::UI)
            .size(type_scale::CAPTION)
            .color(palette::TEXT_MUTED);
        info_row = info_row.push(Space::with_width(spacing::XS)).push(status_lbl);
    }

    let tray = captured_tray(captured, captured_color, material_lead, 18.0);

    let content = row![
        info_row,
        Space::with_width(Length::Fill),
        tray,
    ]
    .align_y(Alignment::Center)
    .width(Length::Fixed(bar_width));

    container(content)
        .padding([6.0, 10.0])
        .width(Length::Fixed(bar_width))
        .style(|_theme| container::Style {
            background: Some(palette::BG_SURFACE.into()),
            border: border::rounded(6.0).width(1.0).color(palette::BORDER_SUBTLE),
            ..Default::default()
        })
        .into()
}

/// Renders the unified Chess.com Board Column:
/// - Top player bar
/// - Center: [Optional Eval Bar] + [Chess Board]
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

    let board_size = 520.0;
    let eval_bar_w = if eval_bar.is_some() { 32.0 } else { 0.0 };
    let gap = if eval_bar.is_some() { 8.0 } else { 0.0 };
    let total_width = board_size + eval_bar_w + gap;

    let top_bar = render_player_bar(top_player, top_captured, top_captured_color, top_lead, total_width);
    let bottom_bar = render_player_bar(bottom_player, bottom_captured, bottom_captured_color, bottom_lead, total_width);

    let board_element = container(
        board_widget.view(board_size).map(map_board_message)
    )
    .style(|_theme| container::Style {
        border: border::rounded(6.0).width(2.0).color(palette::BORDER),
        ..Default::default()
    });

    let board_row = if let Some(bar) = eval_bar {
        row![bar, Space::with_width(gap), board_element].align_y(Alignment::Center)
    } else {
        row![board_element].align_y(Alignment::Center)
    };

    column![
        top_bar,
        Space::with_height(spacing::XS),
        board_row,
        Space::with_height(spacing::XS),
        bottom_bar,
    ]
    .into()
}
