use crate::analysis::MoveClassification;
use iced::widget::{button, container, svg, text_input};
use iced::{border, Color, Theme};
use std::sync::OnceLock;

// ==========================================
// 1. Centralized Palette Tokens
// ==========================================
pub mod palette {
    use iced::Color;

    // Dark background layers
    pub const BG_BASE: Color = Color::from_rgb(15.0 / 255.0, 17.0 / 255.0, 23.0 / 255.0);       // #0f1117
    pub const BG_SURFACE: Color = Color::from_rgb(22.0 / 255.0, 26.0 / 255.0, 34.0 / 255.0);    // #161a22
    pub const BG_ELEVATED: Color = Color::from_rgb(30.0 / 255.0, 36.0 / 255.0, 48.0 / 255.0);   // #1e2430
    pub const BG_HOVER: Color = Color::from_rgb(40.0 / 255.0, 48.0 / 255.0, 64.0 / 255.0);      // #283040
    pub const BG_ACTIVE: Color = Color::from_rgb(50.0 / 255.0, 60.0 / 255.0, 80.0 / 255.0);     // #323c50

    // Borders
    pub const BORDER: Color = Color::from_rgb(48.0 / 255.0, 56.0 / 255.0, 74.0 / 255.0);        // #30384a
    pub const BORDER_SUBTLE: Color = Color::from_rgb(36.0 / 255.0, 42.0 / 255.0, 56.0 / 255.0); // #242a38
    pub const BORDER_FOCUS: Color = Color::from_rgb(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0); // #3b82f6

    // Typography
    pub const TEXT_PRIMARY: Color = Color::from_rgb(230.0 / 255.0, 237.0 / 255.0, 243.0 / 255.0); // #e6edf3
    pub const TEXT_SECONDARY: Color = Color::from_rgb(139.0 / 255.0, 148.0 / 255.0, 158.0 / 255.0); // #8b949e
    pub const TEXT_MUTED: Color = Color::from_rgb(86.0 / 255.0, 95.0 / 255.0, 108.0 / 255.0);    // #565f6c

    // Unified Accent Blue
    pub const ACCENT: Color = Color::from_rgb(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0);       // #3b82f6
    pub const ACCENT_HOVER: Color = Color::from_rgb(96.0 / 255.0, 165.0 / 255.0, 250.0 / 255.0); // #60a5fa
    pub const ACCENT_MUTED: Color = Color::from_rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.2);

    // Status / Outcomes
    pub const SUCCESS: Color = Color::from_rgb(46.0 / 255.0, 160.0 / 255.0, 67.0 / 255.0);       // #2ea043
    pub const DANGER: Color = Color::from_rgb(218.0 / 255.0, 54.0 / 255.0, 51.0 / 255.0);        // #da3633
    pub const WARNING: Color = Color::from_rgb(210.0 / 255.0, 153.0 / 255.0, 34.0 / 255.0);      // #d29922

    // Move Classification Colors
    pub const BLUNDER: Color = Color::from_rgb(202.0 / 255.0, 52.0 / 255.0, 49.0 / 255.0);
    pub const MISTAKE: Color = Color::from_rgb(229.0 / 255.0, 143.0 / 255.0, 42.0 / 255.0);
    pub const INACCURACY: Color = Color::from_rgb(240.0 / 255.0, 193.0 / 255.0, 92.0 / 255.0);
    pub const GOOD: Color = Color::from_rgb(86.0 / 255.0, 172.0 / 255.0, 225.0 / 255.0);
    pub const EXCELLENT: Color = Color::from_rgb(122.0 / 255.0, 196.0 / 255.0, 85.0 / 255.0);
    pub const BEST: Color = Color::from_rgb(150.0 / 255.0, 188.0 / 255.0, 75.0 / 255.0);
    pub const BRILLIANT: Color = Color::from_rgb(27.0 / 255.0, 172.0 / 255.0, 166.0 / 255.0);
    pub const BOOK: Color = Color::from_rgb(213.0 / 255.0, 164.0 / 255.0, 108.0 / 255.0);
}


// ==========================================
// Typography Fonts
// ==========================================
pub mod fonts {
    use iced::font::{Family, Weight};
    use iced::Font;

    pub const UI: Font = Font {
        family: Family::Name("Inter"),
        weight: Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    };

    pub const UI_BOLD: Font = Font {
        family: Family::Name("Inter"),
        weight: Weight::Bold,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    };

    pub const UI_SEMIBOLD: Font = Font {
        family: Family::Name("Inter"),
        weight: Weight::Semibold,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    };

    pub const MONO: Font = Font {
        family: Family::Name("JetBrains Mono"),
        weight: Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    };

    pub const MONO_BOLD: Font = Font {
        family: Family::Name("JetBrains Mono"),
        weight: Weight::Bold,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    };
}

// ==========================================
// 2. Typography Scale
// ==========================================
pub mod type_scale {
    pub const DISPLAY: f32 = 22.0;
    pub const TITLE: f32 = 18.0;
    pub const HEADING: f32 = 14.0;
    pub const BODY: f32 = 13.0;
    pub const CAPTION: f32 = 11.0;
}

// ==========================================
// 3. Spacing Scale & Corner Radius
// ==========================================
pub mod spacing {
    pub const XXS: f32 = 4.0;
    pub const XS: f32 = 8.0;
    pub const SM: f32 = 12.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const RADIUS: f32 = 6.0;
}

// ==========================================
// 4. Custom Dark Theme Definition
// ==========================================
pub fn custom_dark_theme() -> Theme {
    let custom_palette = iced::theme::Palette {
        background: palette::BG_BASE,
        text: palette::TEXT_PRIMARY,
        primary: palette::ACCENT,
        success: palette::SUCCESS,
        danger: palette::DANGER,
    };
    Theme::custom("ChessAnalyzerDark".to_string(), custom_palette)
}

// ==========================================
// 5. Shared Widget Styles
// ==========================================

pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, border_color) = match status {
        button::Status::Hovered => (palette::ACCENT_HOVER, palette::ACCENT_HOVER),
        button::Status::Pressed => (palette::ACCENT, palette::ACCENT_HOVER),
        button::Status::Disabled => (palette::BG_ELEVATED, palette::BORDER_SUBTLE),
        button::Status::Active => (palette::ACCENT, palette::ACCENT),
    };

    button::Style {
        background: Some(bg.into()),
        text_color: palette::TEXT_PRIMARY,
        border: border::rounded(spacing::RADIUS).color(border_color).width(1.0),
        ..Default::default()
    }
}

pub fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, border_color) = match status {
        button::Status::Hovered => (palette::BG_HOVER, palette::BORDER),
        button::Status::Pressed => (palette::BG_ACTIVE, palette::BORDER_FOCUS),
        button::Status::Disabled => (palette::BG_BASE, palette::BORDER_SUBTLE),
        button::Status::Active => (palette::BG_ELEVATED, palette::BORDER),
    };

    button::Style {
        background: Some(bg.into()),
        text_color: palette::TEXT_PRIMARY,
        border: border::rounded(spacing::RADIUS).color(border_color).width(1.0),
        ..Default::default()
    }
}

pub fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb8(238, 74, 71),
        button::Status::Pressed => Color::from_rgb8(190, 44, 41),
        button::Status::Disabled => palette::BG_ELEVATED,
        button::Status::Active => palette::DANGER,
    };

    button::Style {
        background: Some(bg.into()),
        text_color: palette::TEXT_PRIMARY,
        border: border::rounded(spacing::RADIUS).color(palette::DANGER).width(1.0),
        ..Default::default()
    }
}

pub fn nav_button_style(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        if selected {
            button::Style {
                background: Some(palette::ACCENT_MUTED.into()),
                text_color: palette::TEXT_PRIMARY,
                border: border::rounded(spacing::RADIUS).color(palette::ACCENT).width(1.0),
                ..Default::default()
            }
        } else {
            let bg = match status {
                button::Status::Hovered => palette::BG_HOVER,
                button::Status::Pressed => palette::BG_ACTIVE,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: if status == button::Status::Hovered {
                    palette::TEXT_PRIMARY
                } else {
                    palette::TEXT_SECONDARY
                },
                border: border::rounded(spacing::RADIUS).color(Color::TRANSPARENT).width(1.0),
                ..Default::default()
            }
        }
    }
}

pub fn card_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BG_ELEVATED.into()),
        border: border::rounded(spacing::RADIUS).color(palette::BORDER).width(1.0),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn surface_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BG_SURFACE.into()),
        border: border::rounded(spacing::RADIUS).color(palette::BORDER).width(1.0),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn base_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BG_BASE.into()),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn badge_container_style(bg_color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(bg_color.into()),
        border: border::rounded(spacing::RADIUS),
        text_color: Some(palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn text_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => palette::BORDER_FOCUS,
        text_input::Status::Hovered => palette::BORDER,
        text_input::Status::Disabled => palette::BORDER_SUBTLE,
        text_input::Status::Active => palette::BORDER,
    };

    text_input::Style {
        background: palette::BG_ELEVATED.into(),
        border: border::rounded(spacing::RADIUS).color(border_color).width(1.0),
        icon: palette::TEXT_MUTED,
        placeholder: palette::TEXT_MUTED,
        value: palette::TEXT_PRIMARY,
        selection: palette::ACCENT_MUTED,
    }
}

pub fn classification_color(c: MoveClassification) -> Color {
    match c {
        MoveClassification::Brilliant => palette::BRILLIANT,
        MoveClassification::Great => palette::ACCENT,
        MoveClassification::Best => palette::BEST,
        MoveClassification::Excellent => palette::EXCELLENT,
        MoveClassification::Good => palette::GOOD,
        MoveClassification::Book => palette::BOOK,
        MoveClassification::Inaccuracy => palette::INACCURACY,
        MoveClassification::Mistake => palette::MISTAKE,
        MoveClassification::Blunder => palette::BLUNDER,
    }
}

// ==========================================
// 6. Lucide SVG Icon System
// ==========================================
const SVG_FOLDER_OPEN: &[u8] = include_bytes!("../../assets/icons/lucide/folder-open.svg");
const SVG_SEARCH: &[u8] = include_bytes!("../../assets/icons/lucide/search.svg");
const SVG_BAR_CHART: &[u8] = include_bytes!("../../assets/icons/lucide/bar-chart-2.svg");
const SVG_PUZZLE: &[u8] = include_bytes!("../../assets/icons/lucide/puzzle.svg");
const SVG_BOT: &[u8] = include_bytes!("../../assets/icons/lucide/bot.svg");
const SVG_BOOK_OPEN: &[u8] = include_bytes!("../../assets/icons/lucide/book-open.svg");
const SVG_ROTATE_CCW: &[u8] = include_bytes!("../../assets/icons/lucide/rotate-ccw.svg");
const SVG_SPARKLES: &[u8] = include_bytes!("../../assets/icons/lucide/sparkles.svg");
const SVG_CHECK: &[u8] = include_bytes!("../../assets/icons/lucide/check.svg");
const SVG_X: &[u8] = include_bytes!("../../assets/icons/lucide/x.svg");
const SVG_PLAY: &[u8] = include_bytes!("../../assets/icons/lucide/play.svg");
const SVG_TROPHY: &[u8] = include_bytes!("../../assets/icons/lucide/trophy.svg");

static ICON_FOLDER_OPEN: OnceLock<svg::Handle> = OnceLock::new();
static ICON_SEARCH: OnceLock<svg::Handle> = OnceLock::new();
static ICON_BAR_CHART: OnceLock<svg::Handle> = OnceLock::new();
static ICON_PUZZLE: OnceLock<svg::Handle> = OnceLock::new();
static ICON_BOT: OnceLock<svg::Handle> = OnceLock::new();
static ICON_BOOK_OPEN: OnceLock<svg::Handle> = OnceLock::new();
static ICON_ROTATE_CCW: OnceLock<svg::Handle> = OnceLock::new();
static ICON_SPARKLES: OnceLock<svg::Handle> = OnceLock::new();
static ICON_CHECK: OnceLock<svg::Handle> = OnceLock::new();
static ICON_X: OnceLock<svg::Handle> = OnceLock::new();
static ICON_PLAY: OnceLock<svg::Handle> = OnceLock::new();
static ICON_TROPHY: OnceLock<svg::Handle> = OnceLock::new();
const SVG_SKIP_BACK: &[u8] = include_bytes!("../../assets/icons/lucide/skip-back.svg");
const SVG_SKIP_FORWARD: &[u8] = include_bytes!("../../assets/icons/lucide/skip-forward.svg");
const SVG_CHEVRON_LEFT: &[u8] = include_bytes!("../../assets/icons/lucide/chevron-left.svg");
const SVG_CHEVRON_RIGHT: &[u8] = include_bytes!("../../assets/icons/lucide/chevron-right.svg");
const SVG_CHEVRONS_LEFT: &[u8] = include_bytes!("../../assets/icons/lucide/chevrons-left.svg");
const SVG_CHEVRONS_RIGHT: &[u8] = include_bytes!("../../assets/icons/lucide/chevrons-right.svg");
const SVG_UNDO: &[u8] = include_bytes!("../../assets/icons/lucide/undo-2.svg");
const SVG_REFRESH_CW: &[u8] = include_bytes!("../../assets/icons/lucide/refresh-cw.svg");
const SVG_CPU: &[u8] = include_bytes!("../../assets/icons/lucide/cpu.svg");
const SVG_EYE: &[u8] = include_bytes!("../../assets/icons/lucide/eye.svg");
const SVG_DOWNLOAD: &[u8] = include_bytes!("../../assets/icons/lucide/download.svg");

const SVG_LOGO: &[u8] = include_bytes!("../../assets/logo.svg");
static ICON_LOGO: OnceLock<svg::Handle> = OnceLock::new();

static ICON_SKIP_BACK: OnceLock<svg::Handle> = OnceLock::new();
static ICON_SKIP_FORWARD: OnceLock<svg::Handle> = OnceLock::new();
static ICON_CHEVRON_LEFT: OnceLock<svg::Handle> = OnceLock::new();
static ICON_CHEVRON_RIGHT: OnceLock<svg::Handle> = OnceLock::new();
static ICON_CHEVRONS_LEFT: OnceLock<svg::Handle> = OnceLock::new();
static ICON_CHEVRONS_RIGHT: OnceLock<svg::Handle> = OnceLock::new();
static ICON_UNDO: OnceLock<svg::Handle> = OnceLock::new();
static ICON_REFRESH_CW: OnceLock<svg::Handle> = OnceLock::new();
static ICON_CPU: OnceLock<svg::Handle> = OnceLock::new();
static ICON_EYE: OnceLock<svg::Handle> = OnceLock::new();
static ICON_DOWNLOAD: OnceLock<svg::Handle> = OnceLock::new();


pub fn icon_logo() -> svg::Handle {
    ICON_LOGO.get_or_init(|| svg::Handle::from_memory(SVG_LOGO)).clone()
}

pub fn icon_folder_open() -> svg::Handle {
    ICON_FOLDER_OPEN.get_or_init(|| svg::Handle::from_memory(SVG_FOLDER_OPEN)).clone()
}

pub fn icon_search() -> svg::Handle {
    ICON_SEARCH.get_or_init(|| svg::Handle::from_memory(SVG_SEARCH)).clone()
}

pub fn icon_insights() -> svg::Handle {
    ICON_BAR_CHART.get_or_init(|| svg::Handle::from_memory(SVG_BAR_CHART)).clone()
}

pub fn icon_puzzle() -> svg::Handle {
    ICON_PUZZLE.get_or_init(|| svg::Handle::from_memory(SVG_PUZZLE)).clone()
}

pub fn icon_bot() -> svg::Handle {
    ICON_BOT.get_or_init(|| svg::Handle::from_memory(SVG_BOT)).clone()
}

pub fn icon_book_open() -> svg::Handle {
    ICON_BOOK_OPEN.get_or_init(|| svg::Handle::from_memory(SVG_BOOK_OPEN)).clone()
}

pub fn icon_rotate_ccw() -> svg::Handle {
    ICON_ROTATE_CCW.get_or_init(|| svg::Handle::from_memory(SVG_ROTATE_CCW)).clone()
}

pub fn icon_sparkles() -> svg::Handle {
    ICON_SPARKLES.get_or_init(|| svg::Handle::from_memory(SVG_SPARKLES)).clone()
}

pub fn icon_check() -> svg::Handle {
    ICON_CHECK.get_or_init(|| svg::Handle::from_memory(SVG_CHECK)).clone()
}

pub fn icon_x() -> svg::Handle {
    ICON_X.get_or_init(|| svg::Handle::from_memory(SVG_X)).clone()
}

pub fn icon_play() -> svg::Handle {
    ICON_PLAY.get_or_init(|| svg::Handle::from_memory(SVG_PLAY)).clone()
}

pub fn icon_trophy() -> svg::Handle {
    ICON_TROPHY.get_or_init(|| svg::Handle::from_memory(SVG_TROPHY)).clone()
}


pub fn icon_skip_back() -> svg::Handle {
    ICON_SKIP_BACK.get_or_init(|| svg::Handle::from_memory(SVG_SKIP_BACK)).clone()
}

pub fn icon_skip_forward() -> svg::Handle {
    ICON_SKIP_FORWARD.get_or_init(|| svg::Handle::from_memory(SVG_SKIP_FORWARD)).clone()
}

pub fn icon_chevron_left() -> svg::Handle {
    ICON_CHEVRON_LEFT.get_or_init(|| svg::Handle::from_memory(SVG_CHEVRON_LEFT)).clone()
}

pub fn icon_chevron_right() -> svg::Handle {
    ICON_CHEVRON_RIGHT.get_or_init(|| svg::Handle::from_memory(SVG_CHEVRON_RIGHT)).clone()
}

pub fn icon_chevrons_left() -> svg::Handle {
    ICON_CHEVRONS_LEFT.get_or_init(|| svg::Handle::from_memory(SVG_CHEVRONS_LEFT)).clone()
}

pub fn icon_chevrons_right() -> svg::Handle {
    ICON_CHEVRONS_RIGHT.get_or_init(|| svg::Handle::from_memory(SVG_CHEVRONS_RIGHT)).clone()
}

pub fn icon_undo() -> svg::Handle {
    ICON_UNDO.get_or_init(|| svg::Handle::from_memory(SVG_UNDO)).clone()
}

pub fn icon_refresh_cw() -> svg::Handle {
    ICON_REFRESH_CW.get_or_init(|| svg::Handle::from_memory(SVG_REFRESH_CW)).clone()
}

pub fn icon_cpu() -> svg::Handle {
    ICON_CPU.get_or_init(|| svg::Handle::from_memory(SVG_CPU)).clone()
}

pub fn icon_eye() -> svg::Handle {
    ICON_EYE.get_or_init(|| svg::Handle::from_memory(SVG_EYE)).clone()
}

pub fn icon_download() -> svg::Handle {
    ICON_DOWNLOAD.get_or_init(|| svg::Handle::from_memory(SVG_DOWNLOAD)).clone()
}

pub fn icon_widget<'a>(handle: svg::Handle, size: f32, color: Option<Color>) -> svg::Svg<'a, Theme> {
    let s = svg::Svg::new(handle).width(size).height(size);
    if let Some(c) = color {
        s.style(move |_theme, _status| svg::Style { color: Some(c) })
    } else {
        s
    }
}
