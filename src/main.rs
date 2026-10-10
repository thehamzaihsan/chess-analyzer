use chess_analyzer::app::App;
use iced::window;
use iced::Size;

const ICON_RGBA: &[u8] = include_bytes!("../assets/icon.rgba");
const FONT_INTER: &[u8] = include_bytes!("../assets/fonts/Inter-Variable.ttf");
const FONT_JETBRAINS_MONO: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
const FONT_JETBRAINS_MONO_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");

fn load_app_icon() -> Option<window::Icon> {
    window::icon::from_rgba(ICON_RGBA.to_vec(), 64, 64).ok()
}

fn main() -> iced::Result {
    iced::application("Chess Analyzer - Native Rust", App::update, App::view)
        .settings(iced::Settings {
            id: Some("chess-analyzer".to_string()),
            ..Default::default()
        })
        .window(window::Settings {
            size: Size::new(1280.0, 840.0),
            min_size: Some(Size::new(960.0, 640.0)),
            resizable: true,
            decorations: true,
            icon: load_app_icon(),
            ..Default::default()
        })
        .font(FONT_INTER)
        .font(FONT_JETBRAINS_MONO)
        .font(FONT_JETBRAINS_MONO_BOLD)
        .default_font(iced::Font {
            family: iced::font::Family::Name("Inter"),
            ..Default::default()
        })
        .theme(App::theme)
        .run_with(App::new)
}
