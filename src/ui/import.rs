use crate::db::StoredGame;
use crate::ui::theme::{
    badge_container_style, card_container_style, fonts, icon_cpu, icon_download, icon_eye,
    icon_widget, palette, primary_button_style, secondary_button_style, spacing,
    text_input_style, truncate, type_scale, with_tooltip,
};
use iced::border;
use iced::keyboard::{key::Named, Key};
use iced::widget::{
    button, column, container, row, scrollable, text, text_editor, text_input, tooltip, Space,
};
use iced::{Alignment, Element, Length, Theme};
use std::collections::HashSet;

pub const GAMES_PER_PAGE: usize = 25;

#[derive(Debug, Clone)]
pub enum ImportMessage {
    UsernameChanged(String),
    FetchChessCom,
    PgnAction(text_editor::Action),
    LoadPgnText,
    LoadSamplePgn,
    ClearPlayerFilter,
    SelectGameForReview(i64),
    StartGameAnalysis(i64),
    AnalyzeAllGames,
    NextPage,
    PrevPage,
}

#[derive(Debug, Default)]
pub struct ImportScreen {
    pub username_input: String,
    pub pgn_content: text_editor::Content,
    pub status_message: Option<String>,
    pub is_loading: bool,
    pub current_page: usize,
    pub analyzing_games: HashSet<i64>,
    pub is_batch_analyzing: bool,
    pub active_player_filter: Option<String>,
}

fn pgn_editor_style(_theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let border_color = match status {
        text_editor::Status::Focused => palette::BORDER_FOCUS,
        text_editor::Status::Hovered => palette::BORDER,
        text_editor::Status::Disabled => palette::BORDER_SUBTLE,
        text_editor::Status::Active => palette::BORDER,
    };

    text_editor::Style {
        background: palette::BG_ELEVATED.into(),
        border: border::rounded(spacing::RADIUS).color(border_color).width(1.0),
        icon: palette::TEXT_MUTED,
        placeholder: palette::TEXT_MUTED,
        value: palette::TEXT_PRIMARY,
        selection: palette::ACCENT_MUTED,
    }
}

impl ImportScreen {
    pub fn new() -> Self {
        Self {
            username_input: String::new(),
            pgn_content: text_editor::Content::new(),
            status_message: None,
            is_loading: false,
            current_page: 0,
            analyzing_games: HashSet::new(),
            is_batch_analyzing: false,
            active_player_filter: None,
        }
    }

    pub fn view<'a>(&'a self, games: &'a [StoredGame]) -> Element<'a, ImportMessage> {
        let title = text("Import Games & Archives")
            .font(fonts::UI_BOLD)
            .size(type_scale::TITLE)
            .color(palette::TEXT_PRIMARY)
            .wrapping(text::Wrapping::WordOrGlyph);

        let user_label = text("Fetch from Chess.com Public API:")
            .font(fonts::UI_SEMIBOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_SECONDARY)
            .wrapping(text::Wrapping::WordOrGlyph);

        let user_input = text_input(
            "Enter Chess.com username (e.g. GothamChess)",
            &self.username_input,
        )
        .font(fonts::UI)
        .on_input(ImportMessage::UsernameChanged)
        .on_submit(ImportMessage::FetchChessCom)
        .padding(spacing::SM)
        .width(Length::Fixed(340.0))
        .style(text_input_style);

        let fetch_content = row![
            icon_widget(icon_download(), 14.0, Some(palette::TEXT_PRIMARY)),
            text(if self.is_loading { "Fetching..." } else { "Fetch Archives" })
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let fetch_btn = button(fetch_content)
            .padding([spacing::SM, spacing::MD])
            .style(primary_button_style)
            .on_press_maybe(if self.is_loading { None } else { Some(ImportMessage::FetchChessCom) });

        let sample_btn = button(
            text("Load Sample PGN Archive")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        )
        .padding([spacing::SM, spacing::MD])
        .style(secondary_button_style)
        .on_press(ImportMessage::LoadSamplePgn);

        let fetch_row = row![user_input, fetch_btn, sample_btn]
            .spacing(spacing::SM)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        let pgn_label = text("Or paste PGN text directly (multi-game PGN supported, Ctrl+Enter to import):")
            .font(fonts::UI_SEMIBOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_SECONDARY)
            .wrapping(text::Wrapping::WordOrGlyph);

        let pgn_editor = text_editor(&self.pgn_content)
            .placeholder("Paste single or multi-game PGN here...")
            .font(fonts::MONO)
            .size(type_scale::BODY)
            .height(Length::Fixed(96.0))
            .padding(spacing::SM)
            .on_action(ImportMessage::PgnAction)
            .key_binding(|key_press| {
                if key_press.status == text_editor::Status::Focused
                    && key_press.key == Key::Named(Named::Enter)
                    && key_press.modifiers.command()
                {
                    Some(text_editor::Binding::Custom(ImportMessage::LoadPgnText))
                } else {
                    text_editor::Binding::from_key_press(key_press)
                }
            })
            .style(pgn_editor_style);

        let parse_btn = button(
            text("Parse & Import")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        )
        .padding([spacing::SM, spacing::MD])
        .style(primary_button_style)
        .on_press(ImportMessage::LoadPgnText);

        let pgn_row = row![pgn_editor, parse_btn]
            .spacing(spacing::SM)
            .align_y(Alignment::End)
            .width(Length::Fill);

        let import_panel = container(
            column![
                user_label,
                Space::with_height(spacing::XXS),
                fetch_row,
                Space::with_height(spacing::SM),
                pgn_label,
                Space::with_height(spacing::XXS),
                pgn_row,
            ]
            .spacing(spacing::XS)
            .width(Length::Fill),
        )
        .padding(spacing::MD)
        .width(Length::Fill)
        .clip(true)
        .style(card_container_style);

        let status_view: Element<'a, ImportMessage> = match &self.status_message {
            Some(msg) => text(msg)
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .color(palette::ACCENT)
                .wrapping(text::Wrapping::WordOrGlyph)
                .width(Length::Fill)
                .into(),
            None => Space::new(Length::Shrink, Length::Shrink).into(),
        };

        // Table header & player filter indicator
        let filter_controls: Element<'a, ImportMessage> = match &self.active_player_filter {
            Some(player) => {
                let short_player = truncate(player, 20);
                let player_txt = text(format!("Player: {}", short_player))
                    .font(fonts::MONO)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_PRIMARY)
                    .wrapping(text::Wrapping::None);
                let badge = container(player_txt)
                    .padding([spacing::XXS, spacing::XS])
                    .clip(true)
                    .style(badge_container_style(palette::ACCENT_MUTED));
                let badge_el: Element<'a, ImportMessage> = if short_player != *player {
                    with_tooltip(badge, player, tooltip::Position::Bottom)
                } else {
                    badge.into()
                };

                row![
                    badge_el,
                    Space::with_width(spacing::XS),
                    button(
                        text("Show All Games")
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::CAPTION)
                            .wrapping(text::Wrapping::None)
                    )
                    .padding([spacing::XXS, spacing::XS])
                    .style(secondary_button_style)
                    .on_press_maybe(if self.is_loading {
                        None
                    } else {
                        Some(ImportMessage::ClearPlayerFilter)
                    }),
                ]
                .align_y(Alignment::Center)
                .into()
            }
            None => container(
                text("All Players")
                    .font(fonts::MONO)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_SECONDARY)
                    .wrapping(text::Wrapping::None),
            )
            .padding([spacing::XXS, spacing::XS])
            .style(badge_container_style(palette::BG_ELEVATED))
            .into(),
        };

        let analyze_all_label = if self.is_batch_analyzing {
            "Analyzing…"
        } else {
            "Analyze All with Stockfish"
        };

        let analyze_all_content = row![
            icon_widget(icon_cpu(), 14.0, Some(palette::TEXT_PRIMARY)),
            text(analyze_all_label)
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let can_analyze_all = !self.is_batch_analyzing && !games.is_empty();
        let analyze_all_btn = button(analyze_all_content)
            .padding([spacing::XS, spacing::SM])
            .style(primary_button_style)
            .on_press_maybe(if can_analyze_all {
                Some(ImportMessage::AnalyzeAllGames)
            } else {
                None
            });

        let table_title_row = row![
            text(format!("Stored Games ({})", games.len()))
                .font(fonts::UI_BOLD)
                .size(type_scale::HEADING)
                .color(palette::TEXT_PRIMARY)
                .wrapping(text::Wrapping::None),
            Space::with_width(spacing::SM),
            filter_controls,
            Space::with_width(Length::Fill),
            analyze_all_btn,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let header_row = row![
            text("White").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None).width(Length::FillPortion(3)),
            text("Black").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None).width(Length::FillPortion(3)),
            text("Result").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None).width(Length::FillPortion(1)),
            text("Opening").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None).width(Length::FillPortion(4)),
            text("Date").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None).width(Length::FillPortion(2)),
            text("Accuracy").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None).width(Length::FillPortion(2)),
            text("Actions").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None).width(Length::FillPortion(3)),
        ]
        .spacing(spacing::SM)
        .padding([spacing::XS, spacing::SM])
        .width(Length::Fill);

        let total_pages = games.len().div_ceil(GAMES_PER_PAGE).max(1);
        let current_page = self.current_page.min(total_pages - 1);
        let page_start = current_page * GAMES_PER_PAGE;

        let mut games_list = column![].spacing(spacing::XXS).width(Length::Fill);
        for game in games.iter().skip(page_start).take(GAMES_PER_PAGE) {
            let white_full = format!("{} ({})", game.white, game.white_elo.unwrap_or(0));
            let black_full = format!("{} ({})", game.black, game.black_elo.unwrap_or(0));
            let white_short = truncate(&white_full, 22);
            let black_short = truncate(&black_full, 22);
            let opening_short = truncate(&game.opening, 32);

            let white_el: Element<'a, ImportMessage> = if white_short != white_full {
                with_tooltip(
                    text(white_short).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).wrapping(text::Wrapping::None),
                    white_full,
                    tooltip::Position::Top,
                )
            } else {
                text(white_short).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).wrapping(text::Wrapping::None).into()
            };

            let black_el: Element<'a, ImportMessage> = if black_short != black_full {
                with_tooltip(
                    text(black_short).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).wrapping(text::Wrapping::None),
                    black_full,
                    tooltip::Position::Top,
                )
            } else {
                text(black_short).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).wrapping(text::Wrapping::None).into()
            };

            let opening_el: Element<'a, ImportMessage> = if opening_short != game.opening {
                with_tooltip(
                    text(opening_short).font(fonts::UI).size(type_scale::BODY).color(palette::TEXT_SECONDARY).wrapping(text::Wrapping::None),
                    &game.opening,
                    tooltip::Position::Top,
                )
            } else {
                text(opening_short).font(fonts::UI).size(type_scale::BODY).color(palette::TEXT_SECONDARY).wrapping(text::Wrapping::None).into()
            };

            let acc_str = match (game.accuracy_white, game.accuracy_black) {
                (Some(w), Some(b)) => format!("{:.0}% / {:.0}%", w, b),
                _ => "—".to_string(),
            };

            let game_id = game.id;
            let review_content = row![
                icon_widget(icon_eye(), 13.0, Some(palette::TEXT_PRIMARY)),
                text("Review").font(fonts::UI_SEMIBOLD).size(type_scale::BODY).wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let review_btn = button(review_content)
                .padding([spacing::XXS, spacing::SM])
                .style(secondary_button_style)
                .on_press(ImportMessage::SelectGameForReview(game_id));

            let is_analyzing = self.analyzing_games.contains(&game_id);
            let analyze_content = row![
                icon_widget(icon_cpu(), 13.0, Some(palette::TEXT_PRIMARY)),
                text(if is_analyzing { "Analyzing…" } else { "Analyze" })
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::BODY)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let analyze_btn = button(analyze_content)
                .padding([spacing::XXS, spacing::SM])
                .style(primary_button_style)
                .on_press_maybe(if is_analyzing {
                    None
                } else {
                    Some(ImportMessage::StartGameAnalysis(game_id))
                });

            let row_item = container(
                row![
                    container(white_el).width(Length::FillPortion(3)).clip(true),
                    container(black_el).width(Length::FillPortion(3)).clip(true),
                    container(text(&game.result).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::TEXT_SECONDARY).wrapping(text::Wrapping::None)).width(Length::FillPortion(1)).clip(true),
                    container(opening_el).width(Length::FillPortion(4)).clip(true),
                    container(text(&game.date).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_MUTED).wrapping(text::Wrapping::None)).width(Length::FillPortion(2)).clip(true),
                    container(text(acc_str).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::SUCCESS).wrapping(text::Wrapping::None)).width(Length::FillPortion(2)).clip(true),
                    container(row![review_btn, analyze_btn].spacing(spacing::XS)).width(Length::FillPortion(3)).clip(true),
                ]
                .spacing(spacing::SM)
                .align_y(Alignment::Center)
                .padding([spacing::XS, spacing::SM])
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .clip(true)
            .style(card_container_style);

            games_list = games_list.push(row_item);
        }

        let scrollable_table = scrollable(games_list).height(Length::Fill);

        let pagination_row = row![
            button(text("< Prev").font(fonts::UI_SEMIBOLD).size(type_scale::BODY).wrapping(text::Wrapping::None))
                .padding([spacing::XXS, spacing::SM])
                .style(secondary_button_style)
                .on_press_maybe(if current_page > 0 { Some(ImportMessage::PrevPage) } else { None }),
            Space::with_width(spacing::SM),
            text(format!("Page {} of {}", current_page + 1, total_pages))
                .font(fonts::MONO)
                .size(type_scale::BODY)
                .color(palette::TEXT_SECONDARY)
                .wrapping(text::Wrapping::None),
            Space::with_width(spacing::SM),
            button(text("Next >").font(fonts::UI_SEMIBOLD).size(type_scale::BODY).wrapping(text::Wrapping::None))
                .padding([spacing::XXS, spacing::SM])
                .style(secondary_button_style)
                .on_press_maybe(if current_page + 1 < total_pages { Some(ImportMessage::NextPage) } else { None }),
        ]
        .align_y(Alignment::Center);

        let content = column![
            title,
            Space::with_height(spacing::XS),
            import_panel,
            Space::with_height(spacing::XXS),
            status_view,
            Space::with_height(spacing::SM),
            table_title_row,
            header_row,
            scrollable_table,
            Space::with_height(spacing::XS),
            pagination_row,
        ]
        .spacing(spacing::XS)
        .padding(spacing::MD)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .into()
    }
}
