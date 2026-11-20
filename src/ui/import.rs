use crate::analysis::GameAnalysis;
use crate::db::StoredGame;
use crate::pgn::resolve_opening_display;
use crate::ui::theme::{
    badge_container_style, card_container_style, fonts, icon_check, icon_cpu, icon_download,
    icon_eye, icon_widget, palette, primary_button_style, secondary_button_style, spacing,
    surface_container_style, text_input_style, truncate, type_scale, with_tooltip,
};
use iced::border;
use iced::keyboard::{key::Named, Key};
use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, text_editor, text_input,
    tooltip, Space,
};
use iced::{Alignment, Element, Length, Theme};
use std::collections::{HashMap, HashSet};

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

fn table_header_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BG_BASE.into()),
        text_color: Some(palette::TEXT_MUTED),
        ..Default::default()
    }
}

fn table_row_style(is_even: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let bg = if is_even {
            palette::BG_SURFACE
        } else {
            palette::BG_ELEVATED
        };
        container::Style {
            background: Some(bg.into()),
            text_color: Some(palette::TEXT_PRIMARY),
            ..Default::default()
        }
    }
}

fn table_rule_style(_theme: &Theme) -> iced::widget::rule::Style {
    iced::widget::rule::Style {
        color: palette::BORDER_SUBTLE,
        width: 1,
        radius: 0.0.into(),
        fill_mode: iced::widget::rule::FillMode::Full,
    }
}

fn format_table_result(result: &str) -> String {
    match result.trim() {
        "1/2-1/2" => "½-½".to_string(),
        "" => "—".to_string(),
        other => other.to_string(),
    }
}

fn format_player_cell<'a>(
    name: &'a str,
    elo: Option<i32>,
) -> Element<'a, ImportMessage> {
    // Keep name + rating on one line: the rating is a short fixed-width
    // suffix, so give the name the remaining budget and never let the row
    // squeeze the rating into a vertical character stack.
    let display_name = truncate(name, 14);
    let full_label = match elo {
        Some(rating) if rating > 0 => format!("{} ({})", name, rating),
        _ => name.to_string(),
    };

    let name_widget = text(display_name.clone())
        .font(fonts::UI_SEMIBOLD)
        .size(type_scale::BODY)
        .color(palette::TEXT_PRIMARY)
        .wrapping(text::Wrapping::None);

    let cell_content: Element<'a, ImportMessage> = match elo {
        Some(rating) if rating > 0 => container(
            row![
                name_widget,
                text(format!("({})", rating))
                    .font(fonts::MONO)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_MUTED)
                    .width(Length::Shrink)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .clip(true)
        .into(),
        _ => name_widget.into(),
    };

    if display_name != name {
        with_tooltip(cell_content, full_label, tooltip::Position::Top)
    } else {
        cell_content
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

    pub fn view<'a>(
        &'a self,
        games: &'a [StoredGame],
        analyses: &'a HashMap<i64, GameAnalysis>,
    ) -> Element<'a, ImportMessage> {
        let user_input = text_input(
            "Chess.com username (e.g. GothamChess)",
            &self.username_input,
        )
        .font(fonts::UI)
        .size(type_scale::BODY)
        .on_input(ImportMessage::UsernameChanged)
        .on_submit(ImportMessage::FetchChessCom)
        .padding([spacing::XS, spacing::SM])
        .width(Length::FillPortion(3))
        .style(text_input_style);

        let fetch_content = row![
            icon_widget(icon_download(), 13.0, Some(palette::TEXT_PRIMARY)),
            text(if self.is_loading { "Fetching…" } else { "Fetch Archives" })
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        ]
        .spacing(spacing::XXS)
        .align_y(Alignment::Center);

        let fetch_btn = button(fetch_content)
            .padding([spacing::XS, spacing::SM])
            .style(primary_button_style)
            .on_press_maybe(if self.is_loading {
                None
            } else {
                Some(ImportMessage::FetchChessCom)
            });

        let sample_btn = with_tooltip(
            button(
                text("Sample PGN")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::BODY)
                    .wrapping(text::Wrapping::None),
            )
            .padding([spacing::XS, spacing::SM])
            .style(secondary_button_style)
            .on_press(ImportMessage::LoadSamplePgn),
            "Load Sample PGN Archive",
            tooltip::Position::Bottom,
        );

        let fetch_row = row![
            text("Chess.com:")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .color(palette::TEXT_SECONDARY)
                .wrapping(text::Wrapping::None),
            user_input,
            fetch_btn,
            sample_btn,
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let pgn_editor = text_editor(&self.pgn_content)
            .placeholder("Paste single or multi-game PGN (Ctrl+Enter to import)…")
            .font(fonts::MONO)
            .size(type_scale::CAPTION)
            .height(Length::Fixed(42.0))
            .padding([spacing::XXS, spacing::XS])
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
            text("Parse PGN")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        )
        .padding([spacing::XS, spacing::SM])
        .style(primary_button_style)
        .on_press(ImportMessage::LoadPgnText);

        let pgn_row = row![
            text("PGN Paste:")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .color(palette::TEXT_SECONDARY)
                .wrapping(text::Wrapping::None),
            container(pgn_editor).width(Length::Fill),
            parse_btn,
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let mut import_col = column![fetch_row, pgn_row]
            .spacing(spacing::XS)
            .width(Length::Fill);

        if let Some(msg) = &self.status_message {
            let short_msg = truncate(msg, 110);
            let msg_txt = text(short_msg.clone())
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::CAPTION)
                .color(palette::ACCENT)
                .wrapping(text::Wrapping::WordOrGlyph);
            let msg_el: Element<'a, ImportMessage> = if short_msg != *msg {
                with_tooltip(msg_txt, msg, tooltip::Position::Bottom)
            } else {
                msg_txt.into()
            };
            import_col = import_col.push(msg_el);
        }

        let import_panel = container(import_col)
            .padding([spacing::XS, spacing::SM])
            .width(Length::Fill)
            .clip(true)
            .style(card_container_style);

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
            icon_widget(icon_cpu(), 13.0, Some(palette::TEXT_PRIMARY)),
            text(analyze_all_label)
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .wrapping(text::Wrapping::None),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let can_analyze_all = !self.is_batch_analyzing && !games.is_empty();
        let analyze_all_btn = button(analyze_all_content)
            .padding([spacing::XXS, spacing::SM])
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
            Space::with_width(spacing::XS),
            filter_controls,
            Space::with_width(Length::Fill),
            analyze_all_btn,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill);

        // Column widths matched identically between header and rows
        let col_hdr = |label: &'static str| {
            text(label)
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::CAPTION)
                .color(palette::TEXT_MUTED)
                .wrapping(text::Wrapping::None)
        };

        let header_row = container(
            row![
                container(col_hdr("White")).width(Length::FillPortion(3)).clip(true),
                container(col_hdr("Black")).width(Length::FillPortion(3)).clip(true),
                container(col_hdr("Result")).width(Length::Fixed(56.0)).clip(true),
                container(col_hdr("Opening")).width(Length::FillPortion(4)).clip(true),
                container(col_hdr("Date")).width(Length::Fixed(90.0)).clip(true),
                container(col_hdr("Accuracy")).width(Length::Fixed(96.0)).clip(true),
                container(col_hdr("Actions")).width(Length::Fixed(190.0)).clip(true),
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center)
            .padding([spacing::XS, spacing::SM])
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .style(table_header_style);

        let total_pages = games.len().div_ceil(GAMES_PER_PAGE).max(1);
        let current_page = self.current_page.min(total_pages - 1);
        let page_start = current_page * GAMES_PER_PAGE;

        let mut games_list = column![].spacing(0).width(Length::Fill);
        for (row_idx, game) in games.iter().skip(page_start).take(GAMES_PER_PAGE).enumerate() {
            let white_el = format_player_cell(&game.white, game.white_elo);
            let black_el = format_player_cell(&game.black, game.black_elo);

            let (_resolved_eco, resolved_opening) =
                resolve_opening_display(&game.opening, &game.eco, &game.pgn);
            let opening_full = if resolved_opening.trim().is_empty() {
                "—".to_string()
            } else {
                resolved_opening
            };
            let opening_short = truncate(&opening_full, 30);

            let opening_el: Element<'a, ImportMessage> = if opening_short != opening_full {
                with_tooltip(
                    text(opening_short)
                        .font(fonts::UI)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_SECONDARY)
                        .wrapping(text::Wrapping::None),
                    opening_full,
                    tooltip::Position::Top,
                )
            } else {
                text(opening_short)
                    .font(fonts::UI)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_SECONDARY)
                    .wrapping(text::Wrapping::None)
                    .into()
            };

            let acc_str = match (game.accuracy_white, game.accuracy_black) {
                (Some(w), Some(b)) => format!("{:.0}% / {:.0}%", w, b),
                _ => "—".to_string(),
            };
            let acc_color = if acc_str == "—" {
                palette::TEXT_MUTED
            } else {
                palette::SUCCESS
            };

            let result_display = format_table_result(&game.result);
            let result_color = match game.result.trim() {
                "1-0" | "0-1" => palette::TEXT_PRIMARY,
                "1/2-1/2" => palette::TEXT_SECONDARY,
                _ => palette::TEXT_MUTED,
            };

            let game_id = game.id;
            let review_content = row![
                icon_widget(icon_eye(), 12.0, Some(palette::TEXT_PRIMARY)),
                text("Review")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let review_btn = button(review_content)
                .padding([4.0, 8.0])
                .style(secondary_button_style)
                .on_press(ImportMessage::SelectGameForReview(game_id));

            let is_analyzing = self.analyzing_games.contains(&game_id);
            let is_analyzed = analyses.contains_key(&game_id)
                || (game.accuracy_white.is_some() && game.accuracy_black.is_some());

            let analyze_btn: Element<'a, ImportMessage> = if is_analyzing {
                button(
                    row![
                        icon_widget(icon_cpu(), 12.0, Some(palette::TEXT_PRIMARY)),
                        text("Analyzing…")
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::CAPTION)
                            .wrapping(text::Wrapping::None),
                    ]
                    .spacing(spacing::XXS)
                    .align_y(Alignment::Center),
                )
                .padding([4.0, 8.0])
                .style(secondary_button_style)
                .into()
            } else if is_analyzed {
                let reanalyze_btn = button(
                    row![
                        icon_widget(icon_check(), 12.0, Some(palette::SUCCESS)),
                        text("Re-analyze")
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::CAPTION)
                            .color(palette::TEXT_PRIMARY)
                            .wrapping(text::Wrapping::None),
                    ]
                    .spacing(spacing::XXS)
                    .align_y(Alignment::Center),
                )
                .padding([4.0, 8.0])
                .style(secondary_button_style)
                .on_press(ImportMessage::StartGameAnalysis(game_id));

                with_tooltip(
                    reanalyze_btn,
                    "Analyzed — click to re-analyze with Stockfish",
                    tooltip::Position::Top,
                )
            } else {
                button(
                    row![
                        icon_widget(icon_cpu(), 12.0, Some(palette::TEXT_PRIMARY)),
                        text("Analyze")
                            .font(fonts::UI_SEMIBOLD)
                            .size(type_scale::CAPTION)
                            .wrapping(text::Wrapping::None),
                    ]
                    .spacing(spacing::XXS)
                    .align_y(Alignment::Center),
                )
                .padding([4.0, 8.0])
                .style(primary_button_style)
                .on_press(ImportMessage::StartGameAnalysis(game_id))
                .into()
            };

            let date_short = truncate(&game.date, 10);

            let row_item = container(
                row![
                    container(white_el).width(Length::FillPortion(3)).clip(true),
                    container(black_el).width(Length::FillPortion(3)).clip(true),
                    container(
                        text(result_display)
                            .font(fonts::MONO_BOLD)
                            .size(type_scale::BODY)
                            .color(result_color)
                            .wrapping(text::Wrapping::None),
                    )
                    .width(Length::Fixed(56.0))
                    .clip(true),
                    container(opening_el).width(Length::FillPortion(4)).clip(true),
                    container(
                        text(date_short)
                            .font(fonts::MONO)
                            .size(type_scale::CAPTION)
                            .color(palette::TEXT_MUTED)
                            .wrapping(text::Wrapping::None),
                    )
                    .width(Length::Fixed(90.0))
                    .clip(true),
                    container(
                        text(acc_str)
                            .font(fonts::MONO_BOLD)
                            .size(type_scale::CAPTION)
                            .color(acc_color)
                            .wrapping(text::Wrapping::None),
                    )
                    .width(Length::Fixed(96.0))
                    .clip(true),
                    container(
                        row![review_btn, analyze_btn]
                            .spacing(spacing::XXS)
                            .align_y(Alignment::Center),
                    )
                    .width(Length::Fixed(190.0))
                    .clip(true),
                ]
                .spacing(spacing::XS)
                .align_y(Alignment::Center)
                .padding([6.0, spacing::SM])
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .clip(true)
            .style(table_row_style(row_idx % 2 == 0));

            if row_idx > 0 {
                games_list = games_list.push(horizontal_rule(1).style(table_rule_style));
            }
            games_list = games_list.push(row_item);
        }

        let scrollable_table: Element<'a, ImportMessage> = if games.is_empty() {
            container(
                text("No games imported yet. Fetch Chess.com archives, load the sample PGN, or paste a PGN above.")
                    .font(fonts::UI)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_MUTED)
                    .wrapping(text::Wrapping::WordOrGlyph),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .padding(spacing::LG)
            .into()
        } else {
            scrollable(games_list).height(Length::Fill).into()
        };

        let table_card = container(
            column![
                header_row,
                horizontal_rule(1).style(table_rule_style),
                scrollable_table,
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .style(surface_container_style);

        let pagination_row = row![
            button(
                text("< Prev")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .wrapping(text::Wrapping::None)
            )
            .padding([spacing::XXS, spacing::SM])
            .style(secondary_button_style)
            .on_press_maybe(if current_page > 0 {
                Some(ImportMessage::PrevPage)
            } else {
                None
            }),
            Space::with_width(spacing::XS),
            text(format!("Page {} of {}", current_page + 1, total_pages))
                .font(fonts::MONO)
                .size(type_scale::CAPTION)
                .color(palette::TEXT_SECONDARY)
                .wrapping(text::Wrapping::None),
            Space::with_width(spacing::XS),
            button(
                text("Next >")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .wrapping(text::Wrapping::None)
            )
            .padding([spacing::XXS, spacing::SM])
            .style(secondary_button_style)
            .on_press_maybe(if current_page + 1 < total_pages {
                Some(ImportMessage::NextPage)
            } else {
                None
            }),
        ]
        .align_y(Alignment::Center);

        let content = column![
            import_panel,
            table_title_row,
            table_card,
            pagination_row,
        ]
        .spacing(spacing::XS)
        .padding([spacing::SM, spacing::MD])
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_table_result_handles_draws_and_empty() {
        assert_eq!(format_table_result("1-0"), "1-0");
        assert_eq!(format_table_result("0-1"), "0-1");
        assert_eq!(format_table_result("1/2-1/2"), "½-½");
        assert_eq!(format_table_result(""), "—");
    }
}

