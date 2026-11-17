use crate::ui::theme::{palette, spacing};
use iced::widget::{button, container, rule, scrollable};
use iced::{border, Color, Theme};

// ==========================================
// Shell Layout Dimensions & Tokens
// ==========================================
pub const SIDEBAR_WIDTH_EXPANDED: f32 = 224.0;
pub const SIDEBAR_WIDTH_COLLAPSED: f32 = 64.0;
pub const HEADER_HEIGHT: f32 = 54.0;
pub const STATUS_BAR_HEIGHT: f32 = 28.0;

// Max content widths & bounded scroll dimensions for screen containment
pub const MAX_CONTENT_WIDTH: f32 = 1320.0;
pub const BOARD_CONTENT_MAX_WIDTH: f32 = 1020.0;
pub const BOARD_CONTENT_MAX_HEIGHT: f32 = 700.0;

pub const STATUS_BAR_BG: Color = Color::from_rgb(11.0 / 255.0, 13.0 / 255.0, 18.0 / 255.0); // #0b0d12
pub const ACTIVE_NAV_BG: Color = Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.14);
pub const ACTIVE_NAV_BG_HOVER: Color =
    Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.20);
pub const ACTIVE_NAV_BORDER: Color =
    Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.28);
pub const SUCCESS_PILL_BG: Color =
    Color::from_rgba(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0, 0.16);
pub const SUCCESS_PILL_BORDER: Color =
    Color::from_rgba(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0, 0.35);

// ==========================================
// Shell Container Styles
// ==========================================

pub fn sidebar_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BG_SURFACE.into()),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn header_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BG_SURFACE.into()),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn status_bar_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(STATUS_BAR_BG.into()),
        text_color: Some(palette::TEXT_SECONDARY),
        ..Default::default()
    }
}

pub fn tooltip_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BG_ELEVATED.into()),
        border: border::rounded(spacing::RADIUS)
            .color(palette::BORDER)
            .width(1.0),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn pill_container_style(
    bg: Color,
    border_color: Color,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(bg.into()),
        border: border::rounded(12.0).color(border_color).width(1.0),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn dot_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(color.into()),
        border: border::rounded(4.0),
        ..Default::default()
    }
}

pub fn active_accent_bar_style(active: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(
            if active {
                palette::ACCENT
            } else {
                Color::TRANSPARENT
            }
            .into(),
        ),
        border: border::rounded(2.0),
        ..Default::default()
    }
}

// ==========================================
// Shell Button & Rule & Scrollable Styles
// ==========================================

pub fn sidebar_nav_button_style(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        if selected {
            let bg = match status {
                button::Status::Hovered => ACTIVE_NAV_BG_HOVER,
                _ => ACTIVE_NAV_BG,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: palette::TEXT_PRIMARY,
                border: border::rounded(spacing::RADIUS)
                    .color(ACTIVE_NAV_BORDER)
                    .width(1.0),
                ..Default::default()
            }
        } else {
            let bg = match status {
                button::Status::Hovered => palette::BG_HOVER,
                button::Status::Pressed => palette::BG_ACTIVE,
                _ => Color::TRANSPARENT,
            };
            let text_color = match status {
                button::Status::Hovered | button::Status::Pressed => palette::TEXT_PRIMARY,
                _ => palette::TEXT_SECONDARY,
            };
            button::Style {
                background: Some(bg.into()),
                text_color,
                border: border::rounded(spacing::RADIUS)
                    .color(Color::TRANSPARENT)
                    .width(1.0),
                ..Default::default()
            }
        }
    }
}

pub fn sidebar_toggle_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => palette::BG_HOVER,
        button::Status::Pressed => palette::BG_ACTIVE,
        _ => Color::TRANSPARENT,
    };
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => palette::TEXT_PRIMARY,
        _ => palette::TEXT_MUTED,
    };
    button::Style {
        background: Some(bg.into()),
        text_color,
        border: border::rounded(spacing::RADIUS)
            .color(Color::TRANSPARENT)
            .width(1.0),
        ..Default::default()
    }
}

pub fn subtle_rule_style(_theme: &Theme) -> rule::Style {
    rule::Style {
        color: palette::BORDER_SUBTLE,
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    }
}

pub fn shell_scrollable_style(_theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_color = match status {
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => palette::BORDER,
        scrollable::Status::Active => palette::BG_HOVER,
    };

    let rail = scrollable::Rail {
        background: Some(Color::TRANSPARENT.into()),
        border: border::rounded(4.0),
        scroller: scrollable::Scroller {
            color: scroller_color,
            border: border::rounded(4.0),
        },
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}
