use crate::app::{AppMessage, Tab};
use crate::db::StoredGame;
use crate::ui::shell_theme::{
    active_accent_bar_style, dot_style, header_container_style, pill_container_style,
    shell_scrollable_style, sidebar_container_style, sidebar_nav_button_style,
    sidebar_toggle_button_style, status_bar_container_style, subtle_rule_style,
    tooltip_container_style, HEADER_HEIGHT, MAX_CONTENT_WIDTH, SIDEBAR_WIDTH_COLLAPSED,
    SIDEBAR_WIDTH_EXPANDED, STATUS_BAR_HEIGHT, SUCCESS_PILL_BG, SUCCESS_PILL_BORDER,
    ACTIVE_NAV_BG, ACTIVE_NAV_BORDER,
};
use crate::ui::theme::{
    base_container_style, fonts, icon_book_open, icon_bot, icon_chevrons_left,
    icon_chevrons_right, icon_folder_open, icon_insights, icon_logo, icon_puzzle, icon_search,
    icon_widget, palette, spacing, truncate, type_scale, with_tooltip,
};
use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, tooltip, vertical_rule,
    Space,
};
use iced::{Alignment, Color, Element, Length};

pub struct ShellContext<'a> {
    pub active_tab: Tab,
    pub sidebar_collapsed: bool,
    pub player_username: &'a str,
    pub games_count: usize,
    pub analyzed_count: usize,
    pub engine_available: bool,
    pub import_loading: bool,
    pub import_progress: Option<(usize, usize)>,
    pub import_status: Option<&'a str>,
    pub selected_game: Option<&'a StoredGame>,
    pub selected_game_accuracy: Option<(f64, f64)>,
    pub insights_total_games: usize,
    pub insights_win_rate: f64,
    pub puzzle_rating: Option<i32>,
    pub puzzle_source: Option<&'a str>,
    pub puzzle_filter: Option<&'a str>,
    pub bot_name: &'a str,
    pub bot_elo: u32,
    pub bot_title: &'a str,
    pub explorer_opening: &'a str,
    pub explorer_eco: &'a str,
    pub explorer_plies: usize,
}

pub fn board_workspace_available_size(window_size: iced::Size, sidebar_collapsed: bool) -> iced::Size {
    let sidebar_w = if sidebar_collapsed {
        SIDEBAR_WIDTH_COLLAPSED
    } else {
        SIDEBAR_WIDTH_EXPANDED
    } + 1.0;
    let content_w = (window_size.width - sidebar_w - spacing::XS * 2.0).max(400.0);
    let content_h =
        (window_size.height - HEADER_HEIGHT - STATUS_BAR_HEIGHT - 2.0 - spacing::XS * 2.0)
            .max(320.0);
    iced::Size::new(content_w, content_h)
}

pub fn render_shell<'a>(
    ctx: ShellContext<'a>,
    main_view: Element<'a, AppMessage>,
) -> Element<'a, AppMessage> {
    let sidebar = render_sidebar(&ctx);
    let page_header = render_page_header(&ctx);
    let content_area = render_content_area(ctx.active_tab, main_view);
    let status_bar = render_status_bar(&ctx);

    let main_column = column![
        page_header,
        horizontal_rule(1).style(subtle_rule_style),
        content_area,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let workspace_row = row![
        sidebar,
        vertical_rule(1).style(subtle_rule_style),
        main_column,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    column![
        workspace_row,
        horizontal_rule(1).style(subtle_rule_style),
        status_bar,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn render_sidebar<'a>(ctx: &ShellContext<'a>) -> Element<'a, AppMessage> {
    let collapsed = ctx.sidebar_collapsed;
    let width = if collapsed {
        SIDEBAR_WIDTH_COLLAPSED
    } else {
        SIDEBAR_WIDTH_EXPANDED
    };

    // Brand header aligned with page header height
    let brand_inner: Element<'a, AppMessage> = if collapsed {
        let logo = container(icon_widget(icon_logo(), 28.0, None))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        tooltip(
            logo,
            container(
                text("Chess Analyzer")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_PRIMARY),
            )
            .padding([4.0, 8.0])
            .style(tooltip_container_style),
            tooltip::Position::Right,
        )
        .into()
    } else {
        container(
            row![
                icon_widget(icon_logo(), 28.0, None),
                column![
                    text("Chess Analyzer")
                        .font(fonts::UI_BOLD)
                        .size(14.5)
                        .color(palette::TEXT_PRIMARY)
                        .wrapping(text::Wrapping::None),
                    text("Analysis Studio")
                        .font(fonts::UI)
                        .size(10.5)
                        .color(palette::TEXT_MUTED)
                        .wrapping(text::Wrapping::None),
                ]
                .spacing(1.0)
                .width(Length::Fill),
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .padding([0.0, spacing::SM])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Center)
        .clip(true)
        .into()
    };

    let brand_header = container(brand_inner)
        .width(Length::Fill)
        .height(Length::Fixed(HEADER_HEIGHT));

    let section_label = |label: &'static str| -> Element<'a, AppMessage> {
        if collapsed {
            container(horizontal_rule(1).style(subtle_rule_style))
                .padding([spacing::XS, spacing::SM])
                .width(Length::Fill)
                .into()
        } else {
            container(
                text(label)
                    .font(fonts::UI_BOLD)
                    .size(10.5)
                    .color(palette::TEXT_MUTED)
                    .wrapping(text::Wrapping::None),
            )
            .padding(iced::Padding {
                top: spacing::SM,
                right: spacing::SM,
                bottom: spacing::XXS,
                left: spacing::SM,
            })
            .width(Length::Fill)
            .clip(true)
            .into()
        }
    };

    let games_badge = if !collapsed && ctx.games_count > 0 {
        Some(format!("{}", ctx.games_count))
    } else {
        None
    };

    let analyze_group = column![
        section_label("ANALYZE"),
        nav_item(
            Tab::Import,
            "Games & Import",
            icon_folder_open(),
            ctx.active_tab == Tab::Import,
            collapsed,
            games_badge,
        ),
        nav_item(
            Tab::GameReview,
            "Game Review",
            icon_search(),
            ctx.active_tab == Tab::GameReview,
            collapsed,
            None,
        ),
        nav_item(
            Tab::Insights,
            "Insights",
            icon_insights(),
            ctx.active_tab == Tab::Insights,
            collapsed,
            None,
        ),
    ]
    .spacing(spacing::XXS);

    let train_group = column![
        section_label("TRAIN"),
        nav_item(
            Tab::Puzzles,
            "Puzzles",
            icon_puzzle(),
            ctx.active_tab == Tab::Puzzles,
            collapsed,
            None,
        ),
        nav_item(
            Tab::Bots,
            "Play Bots",
            icon_bot(),
            ctx.active_tab == Tab::Bots,
            collapsed,
            None,
        ),
        nav_item(
            Tab::Explorer,
            "Explorer",
            icon_book_open(),
            ctx.active_tab == Tab::Explorer,
            collapsed,
            None,
        ),
    ]
    .spacing(spacing::XXS);

    let nav_scroll = scrollable(
        column![
            analyze_group,
            Space::with_height(spacing::XS),
            train_group,
        ]
        .padding([spacing::XS, spacing::XS]),
    )
    .height(Length::Fill)
    .style(shell_scrollable_style);

    // Bottom collapse/expand toggle
    let toggle_btn: Element<'a, AppMessage> = if collapsed {
        let btn = button(
            container(icon_widget(
                icon_chevrons_right(),
                16.0,
                Some(palette::TEXT_SECONDARY),
            ))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
        )
        .padding([spacing::XS, spacing::XS])
        .width(Length::Fill)
        .style(sidebar_toggle_button_style)
        .on_press(AppMessage::ToggleSidebar);

        tooltip(
            btn,
            container(
                text("Expand Sidebar")
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_PRIMARY),
            )
            .padding([4.0, 8.0])
            .style(tooltip_container_style),
            tooltip::Position::Right,
        )
        .into()
    } else {
        button(
            container(
                row![
                    icon_widget(icon_chevrons_left(), 15.0, Some(palette::TEXT_MUTED)),
                    text("Collapse Sidebar")
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::CAPTION)
                        .wrapping(text::Wrapping::None),
                ]
                .spacing(spacing::XS)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .clip(true),
        )
        .padding([spacing::XS, spacing::SM])
        .width(Length::Fill)
        .style(sidebar_toggle_button_style)
        .on_press(AppMessage::ToggleSidebar)
        .into()
    };

    let footer = container(toggle_btn)
        .padding(spacing::XS)
        .width(Length::Fill);

    container(
        column![
            brand_header,
            horizontal_rule(1).style(subtle_rule_style),
            nav_scroll,
            horizontal_rule(1).style(subtle_rule_style),
            footer,
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .clip(true)
    .style(sidebar_container_style)
    .into()
}

fn nav_item<'a>(
    tab: Tab,
    label: &'static str,
    icon_handle: iced::widget::svg::Handle,
    selected: bool,
    collapsed: bool,
    badge: Option<String>,
) -> Element<'a, AppMessage> {
    let icon_color = if selected {
        palette::ACCENT_HOVER
    } else {
        palette::TEXT_SECONDARY
    };

    let accent_bar = container(Space::new(Length::Fixed(3.0), Length::Fixed(18.0)))
        .style(active_accent_bar_style(selected));

    if collapsed {
        let content = row![
            accent_bar,
            container(icon_widget(icon_handle, 17.0, Some(icon_color)))
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        ]
        .spacing(2.0)
        .align_y(Alignment::Center);

        let btn = button(content)
            .padding([spacing::XS, 4.0])
            .width(Length::Fill)
            .style(sidebar_nav_button_style(selected))
            .on_press(AppMessage::ChangeTab(tab));

        tooltip(
            btn,
            container(
                text(label)
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_PRIMARY),
            )
            .padding([4.0, 8.0])
            .style(tooltip_container_style),
            tooltip::Position::Right,
        )
        .into()
    } else {
        let label_text = container(
            text(label)
                .font(if selected {
                    fonts::UI_SEMIBOLD
                } else {
                    fonts::UI
                })
                .size(13.5)
                .wrapping(text::Wrapping::None),
        )
        .width(Length::Fill)
        .clip(true);

        let mut content = row![
            accent_bar,
            Space::with_width(spacing::XS),
            icon_widget(icon_handle, 16.0, Some(icon_color)),
            Space::with_width(spacing::XS),
            label_text,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill);

        if let Some(badge_str) = badge {
            let badge_bg = if selected {
                ACTIVE_NAV_BG
            } else {
                palette::BG_ELEVATED
            };
            let badge_border = if selected {
                ACTIVE_NAV_BORDER
            } else {
                palette::BORDER_SUBTLE
            };
            let pill = container(
                text(badge_str)
                    .font(fonts::MONO_BOLD)
                    .size(10.5)
                    .color(if selected {
                        palette::ACCENT_HOVER
                    } else {
                        palette::TEXT_SECONDARY
                    })
                    .wrapping(text::Wrapping::None),
            )
            .padding([1.0, 6.0])
            .style(pill_container_style(badge_bg, badge_border));

            content = content.push(pill);
        }

        button(content)
            .padding([spacing::XS, spacing::XS])
            .width(Length::Fill)
            .style(sidebar_nav_button_style(selected))
            .on_press(AppMessage::ChangeTab(tab))
            .into()
    }
}

fn render_page_header<'a>(ctx: &ShellContext<'a>) -> Element<'a, AppMessage> {
    let (title, subtitle) = match ctx.active_tab {
        Tab::Import => (
            "Games & Import",
            format!("Tracked player: {}", ctx.player_username),
        ),
        Tab::GameReview => {
            let sub = match ctx.selected_game {
                Some(g) => {
                    let (_, resolved_opening) =
                        crate::pgn::resolve_opening_display(&g.opening, &g.eco, &g.pgn);
                    let opening_part = if !resolved_opening.is_empty() {
                        format!("  •  {}", resolved_opening)
                    } else {
                        String::new()
                    };
                    format!("{} vs {}  •  {}{}", g.white, g.black, g.date, opening_part)
                }
                None => "Select a game from Import to start move-by-move review".to_string(),
            };
            ("Game Review", sub)
        }
        Tab::Insights => (
            "Player Insights",
            format!("Performance analytics for {}", ctx.player_username),
        ),
        Tab::Puzzles => {
            let sub = match (ctx.puzzle_rating, ctx.puzzle_source) {
                (Some(r), Some(src)) => format!("Rated {}  •  Source: {}", r, src),
                _ => "Tactics trainer from Lichess & your game blunders".to_string(),
            };
            ("Tactical Puzzles", sub)
        }
        Tab::Bots => (
            "Play Bots",
            format!("Opponent: {} ({})", ctx.bot_name, ctx.bot_title),
        ),
        Tab::Explorer => {
            let eco_prefix = if ctx.explorer_eco.is_empty() {
                String::new()
            } else {
                format!("[{}] ", ctx.explorer_eco)
            };
            (
                "Opening Explorer",
                format!(
                    "{}{}  •  {} plies",
                    eco_prefix, ctx.explorer_opening, ctx.explorer_plies
                ),
            )
        }
    };

    let short_sub = truncate(&subtitle, 56);
    let sub_text = text(short_sub.clone())
        .font(fonts::UI)
        .size(12.5)
        .color(palette::TEXT_SECONDARY)
        .wrapping(text::Wrapping::None);
    let sub_el: Element<'a, AppMessage> = if short_sub != subtitle {
        with_tooltip(sub_text, subtitle, tooltip::Position::Bottom)
    } else {
        sub_text.into()
    };

    let left = row![
        text(title)
            .font(fonts::UI_BOLD)
            .size(16.0)
            .color(palette::TEXT_PRIMARY)
            .wrapping(text::Wrapping::None),
        container(Space::new(Length::Fixed(1.0), Length::Fixed(14.0))).style(|_| {
            container::Style {
                background: Some(palette::BORDER.into()),
                ..Default::default()
            }
        }),
        container(sub_el).width(Length::Fill).clip(true),
    ]
    .spacing(spacing::SM)
    .width(Length::Fill)
    .align_y(Alignment::Center);

    let right_pills = render_header_pills(ctx);

    let header_row = row![left, Space::with_width(spacing::SM), right_pills]
        .padding([0.0, spacing::LG])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Center);

    container(header_row)
        .width(Length::Fill)
        .height(Length::Fixed(HEADER_HEIGHT))
        .clip(true)
        .style(header_container_style)
        .into()
}

fn render_header_pills<'a>(ctx: &ShellContext<'a>) -> Element<'a, AppMessage> {
    let mut pills = row![].spacing(spacing::XS).align_y(Alignment::Center);

    match ctx.active_tab {
        Tab::Import => {
            if ctx.import_loading {
                let label = match ctx.import_progress {
                    Some((done, total)) if total > 0 => format!("Syncing {}/{}...", done, total),
                    _ => "Syncing...".to_string(),
                };
                pills = pills.push(header_pill(
                    label,
                    palette::ACCENT_HOVER,
                    ACTIVE_NAV_BG,
                    ACTIVE_NAV_BORDER,
                ));
            } else {
                pills = pills.push(header_pill(
                    format!("{} stored", ctx.games_count),
                    palette::TEXT_SECONDARY,
                    palette::BG_ELEVATED,
                    palette::BORDER_SUBTLE,
                ));
            }
        }
        Tab::GameReview => {
            if let Some(g) = ctx.selected_game {
                pills = pills.push(header_pill(
                    truncate(&g.result, 10),
                    palette::TEXT_PRIMARY,
                    palette::BG_ELEVATED,
                    palette::BORDER,
                ));
            }
            if let Some((w, b)) = ctx.selected_game_accuracy {
                pills = pills.push(header_pill(
                    format!("Accuracy {:.0}% / {:.0}%", w, b),
                    palette::SUCCESS,
                    SUCCESS_PILL_BG,
                    SUCCESS_PILL_BORDER,
                ));
            }
        }
        Tab::Insights => {
            pills = pills.push(header_pill(
                format!(
                    "{} games  •  {:.1}% win rate",
                    ctx.insights_total_games, ctx.insights_win_rate
                ),
                palette::TEXT_SECONDARY,
                palette::BG_ELEVATED,
                palette::BORDER_SUBTLE,
            ));
        }
        Tab::Puzzles => {
            if let Some(filter) = ctx.puzzle_filter {
                pills = pills.push(header_pill(
                    format!("Filter: {}", truncate(filter, 14)),
                    palette::ACCENT_HOVER,
                    ACTIVE_NAV_BG,
                    ACTIVE_NAV_BORDER,
                ));
            }
            if let Some(rating) = ctx.puzzle_rating {
                pills = pills.push(header_pill(
                    format!("{} Elo", rating),
                    palette::TEXT_PRIMARY,
                    palette::BG_ELEVATED,
                    palette::BORDER_SUBTLE,
                ));
            }
        }
        Tab::Bots => {
            pills = pills.push(header_pill(
                format!("{} Elo", ctx.bot_elo),
                palette::ACCENT_HOVER,
                ACTIVE_NAV_BG,
                ACTIVE_NAV_BORDER,
            ));
        }
        Tab::Explorer => {
            let label = if ctx.explorer_eco.is_empty() {
                "Starting Position".to_string()
            } else {
                format!("ECO {}", truncate(ctx.explorer_eco, 10))
            };
            pills = pills.push(header_pill(
                label,
                palette::ACCENT_HOVER,
                ACTIVE_NAV_BG,
                ACTIVE_NAV_BORDER,
            ));
        }
    }

    pills.into()
}

fn header_pill<'a>(
    label: String,
    text_color: Color,
    bg: Color,
    border_color: Color,
) -> Element<'a, AppMessage> {
    container(
        text(label)
            .font(fonts::MONO_BOLD)
            .size(type_scale::CAPTION)
            .color(text_color)
            .wrapping(text::Wrapping::None),
    )
    .padding([3.0, 9.0])
    .clip(true)
    .style(pill_container_style(bg, border_color))
    .into()
}

fn render_content_area<'a>(
    active_tab: Tab,
    main_view: Element<'a, AppMessage>,
) -> Element<'a, AppMessage> {
    match active_tab {
        // List & dashboard screens manage their own internal Length::Fill scrollables.
        // Constrain max width and center with consistent padding.
        Tab::Import | Tab::Insights => {
            let constrained = container(main_view)
                .max_width(MAX_CONTENT_WIDTH)
                .width(Length::Fill)
                .height(Length::Fill);

            container(constrained)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .padding([spacing::XS, spacing::SM])
                .style(base_container_style)
                .into()
        }
        // Board-centric screens size themselves responsively to fill all available space.
        Tab::GameReview | Tab::Puzzles | Tab::Bots | Tab::Explorer => container(main_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([spacing::XS, spacing::XS])
            .clip(true)
            .style(base_container_style)
            .into(),
    }
}

fn render_status_bar<'a>(ctx: &ShellContext<'a>) -> Element<'a, AppMessage> {
    let status_dot = |color: Color| {
        container(Space::new(Length::Fixed(7.0), Length::Fixed(7.0))).style(dot_style(color))
    };

    let divider = || {
        container(Space::new(Length::Fixed(1.0), Length::Fixed(12.0))).style(|_| {
            container::Style {
                background: Some(palette::BORDER_SUBTLE.into()),
                ..Default::default()
            }
        })
    };

    let player_full = if ctx.player_username.trim().is_empty() {
        "All Players".to_string()
    } else {
        ctx.player_username.to_string()
    };
    let player_short = truncate(&player_full, 18);
    let player_text = text(format!("Player: {}", player_short))
        .font(fonts::UI_SEMIBOLD)
        .size(11.5)
        .color(palette::TEXT_PRIMARY)
        .wrapping(text::Wrapping::None);
    let player_label: Element<'a, AppMessage> = if player_short != player_full {
        with_tooltip(player_text, player_full, tooltip::Position::Top)
    } else {
        player_text.into()
    };

    let player_item = row![status_dot(palette::ACCENT), player_label]
        .spacing(6.0)
        .align_y(Alignment::Center);

    let games_item = text(format!(
        "{} games  •  {} analyzed",
        ctx.games_count, ctx.analyzed_count
    ))
    .font(fonts::MONO)
    .size(11.5)
    .color(palette::TEXT_SECONDARY)
    .wrapping(text::Wrapping::None);

    let (engine_color, engine_label) = if ctx.engine_available {
        (palette::SUCCESS, "Stockfish Ready")
    } else {
        (palette::WARNING, "Engine Unavailable")
    };

    let engine_item = row![
        status_dot(engine_color),
        text(engine_label)
            .font(fonts::UI_SEMIBOLD)
            .size(11.5)
            .color(if ctx.engine_available {
                palette::TEXT_SECONDARY
            } else {
                palette::WARNING
            })
            .wrapping(text::Wrapping::None),
    ]
    .spacing(6.0)
    .align_y(Alignment::Center);

    let status_el: Element<'a, AppMessage> = if let Some(msg) = ctx.import_status {
        let msg_color = if ctx.import_loading {
            palette::ACCENT_HOVER
        } else {
            palette::TEXT_MUTED
        };
        let dot_color = if ctx.import_loading {
            palette::ACCENT
        } else {
            palette::TEXT_MUTED
        };

        let short_msg = truncate(msg, 58);
        let msg_text = text(short_msg.clone())
            .font(fonts::UI)
            .size(11.5)
            .color(msg_color)
            .wrapping(text::Wrapping::None);
        let msg_widget: Element<'a, AppMessage> = if short_msg != msg {
            with_tooltip(msg_text, msg, tooltip::Position::Top)
        } else {
            msg_text.into()
        };

        let status_item = row![status_dot(dot_color), msg_widget]
            .spacing(6.0)
            .align_y(Alignment::Center);

        container(status_item)
            .width(Length::Fill)
            .align_x(Alignment::End)
            .clip(true)
            .into()
    } else {
        Space::with_width(Length::Fill).into()
    };

    let bar = row![
        player_item,
        divider(),
        games_item,
        divider(),
        engine_item,
        status_el,
    ]
    .spacing(spacing::SM)
    .padding([0.0, spacing::MD])
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(STATUS_BAR_HEIGHT))
        .clip(true)
        .style(status_bar_container_style)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text;

    fn sample_game() -> StoredGame {
        StoredGame {
            id: 1,
            pgn: "1. e4 e5".to_string(),
            white: "GothamChess".to_string(),
            black: "Hikaru".to_string(),
            white_elo: Some(2420),
            black_elo: Some(2810),
            result: "0-1".to_string(),
            date: "2026.02.15".to_string(),
            time_control: "180+0".to_string(),
            eco: "C50".to_string(),
            opening: "Italian Game".to_string(),
            accuracy_white: Some(86.4),
            accuracy_black: Some(93.1),
            analysis_json: None,
        }
    }

    #[test]
    fn test_shell_renders_all_tabs_expanded_and_collapsed() {
        let game = sample_game();
        let tabs = [
            Tab::Import,
            Tab::GameReview,
            Tab::Insights,
            Tab::Puzzles,
            Tab::Bots,
            Tab::Explorer,
        ];

        for collapsed in [false, true] {
            for tab in tabs {
                let ctx = ShellContext {
                    active_tab: tab,
                    sidebar_collapsed: collapsed,
                    player_username: "GothamChess",
                    games_count: 12,
                    analyzed_count: 4,
                    engine_available: tab != Tab::Bots,
                    import_loading: tab == Tab::Import,
                    import_progress: Some((2, 10)),
                    import_status: Some("Fetching month 2/10..."),
                    selected_game: Some(&game),
                    selected_game_accuracy: Some((86.4, 93.1)),
                    insights_total_games: 12,
                    insights_win_rate: 58.3,
                    puzzle_rating: Some(1650),
                    puzzle_source: Some("lichess"),
                    puzzle_filter: Some("blunder"),
                    bot_name: "Club Tactician",
                    bot_elo: 1600,
                    bot_title: "Intermediate",
                    explorer_opening: "Italian Game",
                    explorer_eco: "C50",
                    explorer_plies: 4,
                };

                let _el = render_shell(ctx, text("Screen Body").into());
            }
        }
    }
}
