use crate::insights::OverallInsights;
use crate::ui::charts::{ChartMessage, RatingChart};
use crate::ui::theme::{
    card_container_style, fonts, nav_button_style, palette, spacing, type_scale,
};
use iced::border;
use iced::widget::{
    button, column, container, row, scrollable, text, Space,
};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub enum InsightsMessage {
    SelectSection(InsightsSection),
    Chart(ChartMessage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InsightsSection {
    #[default]
    Overview,
    RatingProgress,
    Openings,
    WhenYouPlay,
}

#[derive(Debug, Clone, Default)]
pub struct InsightsScreen {
    pub current_section: InsightsSection,
}

impl InsightsScreen {
    pub fn new() -> Self {
        Self {
            current_section: InsightsSection::Overview,
        }
    }

    pub fn view<'a>(&'a self, insights: &'a OverallInsights) -> Element<'a, InsightsMessage> {
        let title = text("Player Insights & Performance Dashboard")
            .font(fonts::UI_BOLD)
            .size(type_scale::TITLE)
            .color(palette::TEXT_PRIMARY);

        // Section tab buttons
        let tab_btn = |section: InsightsSection, label: &'static str| {
            let is_selected = self.current_section == section;
            button(text(label).font(fonts::UI_SEMIBOLD).size(type_scale::HEADING))
                .padding([spacing::XS, spacing::MD])
                .style(nav_button_style(is_selected))
                .on_press(InsightsMessage::SelectSection(section))
        };

        let nav_tabs = row![
            tab_btn(InsightsSection::Overview, "Overview & Skills"),
            tab_btn(InsightsSection::RatingProgress, "Rating over Time"),
            tab_btn(InsightsSection::Openings, "Openings Performance"),
            tab_btn(InsightsSection::WhenYouPlay, "When You Play & Habits"),
        ]
        .spacing(spacing::XS);

        let section_content: Element<'a, InsightsMessage> = match self.current_section {
            InsightsSection::Overview => self.render_overview(insights),
            InsightsSection::RatingProgress => self.render_rating_progress(insights),
            InsightsSection::Openings => self.render_openings(insights),
            InsightsSection::WhenYouPlay => self.render_when_you_play(insights),
        };

        let content = column![
            title,
            Space::with_height(spacing::XS),
            nav_tabs,
            Space::with_height(spacing::MD),
            section_content,
        ]
        .spacing(spacing::XS)
        .padding(spacing::MD);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn render_overview<'a>(&'a self, insights: &'a OverallInsights) -> Element<'a, InsightsMessage> {
        // Skill cards row (Tactics, Opening, Endgame, Capitalization)
        let skill_card = |label: &'static str, score: u32, sub: &'static str| {
            let score_color = if score >= 80 {
                palette::SUCCESS
            } else if score >= 60 {
                palette::WARNING
            } else {
                palette::DANGER
            };

            container(
                column![
                    text(label)
                        .font(fonts::UI_SEMIBOLD)
                        .size(type_scale::BODY)
                        .color(palette::TEXT_SECONDARY),
                    Space::with_height(spacing::XXS),
                    text(format!("{}/100", score))
                        .font(fonts::MONO_BOLD)
                        .size(type_scale::DISPLAY)
                        .color(score_color),
                    Space::with_height(spacing::XXS),
                    text(sub)
                        .font(fonts::UI)
                        .size(type_scale::CAPTION)
                        .color(palette::TEXT_MUTED),
                ]
                .spacing(spacing::XXS)
            )
            .padding(spacing::MD)
            .width(Length::FillPortion(1))
            .style(card_container_style)
        };

        let skills_row = row![
            skill_card("Tactics Score", insights.skill_scores.tactics, "Tactical accuracy & sharpness"),
            skill_card("Opening Score", insights.skill_scores.opening, "Theory accuracy in moves 1-12"),
            skill_card("Endgame Score", insights.skill_scores.endgame, "Conversion technique (<7 pieces)"),
            skill_card("Capitalization", insights.skill_scores.capitalization, "Punishing opponent blunders"),
        ]
        .spacing(spacing::MD);

        let total_games = insights.total_games;

        let record_card = container(
            column![
                text("Overall Record & Accuracy")
                    .font(fonts::UI_BOLD)
                    .size(type_scale::HEADING)
                    .color(palette::TEXT_PRIMARY),
                Space::with_height(spacing::SM),
                row![
                    self.stat_box("Chess.com Player", &insights.player_id),
                    self.stat_box("Total Games", &format!("{}", total_games)),
                    self.stat_box("Win Rate", &format!("{:.1}%", insights.win_rate)),
                    self.stat_box("Wins / Draws / Losses", &format!("{} / {} / {}", insights.wins, insights.draws, insights.losses)),
                    self.stat_box("Avg Accuracy", &format!("{:.1}%", insights.avg_accuracy)),
                ]
                .spacing(spacing::MD),
            ]
        )
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(card_container_style);

        // Key takeaways card
        let mut takeaways_col = column![
            text("Key Takeaways & Improvement Areas")
                .font(fonts::UI_BOLD)
                .size(type_scale::HEADING)
                .color(palette::TEXT_PRIMARY),
            Space::with_height(spacing::XS),
            text(format!("• Blunder Rate: {:.2} blunders per game on average.", insights.avg_blunders_per_game))
                .font(fonts::UI)
                .size(type_scale::BODY)
                .color(palette::TEXT_PRIMARY),
            text(format!("• Opening Theory: Average accuracy across initial 12 plies is {}/100.", insights.skill_scores.opening))
                .font(fonts::UI)
                .size(type_scale::BODY)
                .color(palette::TEXT_PRIMARY),
        ]
        .spacing(spacing::XS);

        for t in &insights.takeaways {
            takeaways_col = takeaways_col.push(
                text(format!("• {}", t))
                    .font(fonts::UI)
                    .size(type_scale::BODY)
                    .color(palette::TEXT_PRIMARY)
            );
        }

        let takeaways_card = container(takeaways_col)
            .padding(spacing::MD)
            .width(Length::Fill)
            .style(card_container_style);

        column![
            skills_row,
            Space::with_height(spacing::MD),
            record_card,
            Space::with_height(spacing::MD),
            takeaways_card,
        ]
        .into()
    }

    fn render_rating_progress<'a>(&'a self, insights: &'a OverallInsights) -> Element<'a, InsightsMessage> {
        // Player header stat cards
        let net_change_color = if insights.rating_change >= 0 {
            palette::SUCCESS
        } else {
            palette::DANGER
        };

        let stat_cards = row![
            self.stat_box("Tracked Account", &insights.player_id),
            self.stat_box("Current Rating", &format!("{} Elo", insights.current_rating)),
            self.stat_box("Peak Rating", &format!("{} Elo", insights.peak_rating)),
            self.stat_box("Lowest Rating", &format!("{} Elo", insights.lowest_rating)),
            container(
                column![
                    text("Net Rating Change")
                        .font(fonts::UI)
                        .size(type_scale::CAPTION)
                        .color(palette::TEXT_MUTED),
                    Space::with_height(spacing::XXS),
                    text(format!("{:+} Elo", insights.rating_change))
                        .font(fonts::MONO_BOLD)
                        .size(type_scale::TITLE)
                        .color(net_change_color),
                ]
            )
            .padding(spacing::SM)
            .width(Length::FillPortion(1))
            .style(card_container_style),
        ]
        .spacing(spacing::MD);

        // Chart header and legend
        let legend = row![
            container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                .style(|_| container::Style {
                    background: Some(palette::SUCCESS.into()),
                    border: border::rounded(4.0),
                    ..Default::default()
                }),
            text("Win").font(fonts::UI).size(type_scale::CAPTION).color(palette::TEXT_MUTED),
            Space::with_width(spacing::XS),
            container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                .style(|_| container::Style {
                    background: Some(palette::TEXT_MUTED.into()),
                    border: border::rounded(4.0),
                    ..Default::default()
                }),
            text("Draw").font(fonts::UI).size(type_scale::CAPTION).color(palette::TEXT_MUTED),
            Space::with_width(spacing::XS),
            container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                .style(|_| container::Style {
                    background: Some(palette::DANGER.into()),
                    border: border::rounded(4.0),
                    ..Default::default()
                }),
            text("Loss").font(fonts::UI).size(type_scale::CAPTION).color(palette::TEXT_MUTED),
            Space::with_width(spacing::XS),
            container(Space::new(Length::Fixed(14.0), Length::Fixed(2.0)))
                .style(|_| container::Style {
                    background: Some(palette::ACCENT.into()),
                    ..Default::default()
                }),
            text("Rating Trend").font(fonts::UI).size(type_scale::CAPTION).color(palette::TEXT_MUTED),
        ]
        .spacing(spacing::XXS)
        .align_y(Alignment::Center);

        let chart_header = row![
            column![
                text("Rating Progression Over Time")
                    .font(fonts::UI_BOLD)
                    .size(type_scale::HEADING)
                    .color(palette::TEXT_PRIMARY),
                text("Hover any point to inspect individual game result, opponent, and rating change.")
                    .font(fonts::UI)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_SECONDARY),
            ]
            .spacing(spacing::XXS),
            Space::with_width(Length::Fill),
            legend,
        ]
        .align_y(Alignment::Center);

        let chart_canvas = RatingChart::new(&insights.rating_history)
            .view(Length::Fill, Length::Fixed(260.0))
            .map(InsightsMessage::Chart);

        let chart_card = container(
            column![
                chart_header,
                Space::with_height(spacing::SM),
                chart_canvas,
            ]
            .spacing(spacing::XS)
        )
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(card_container_style);

        // Rating Bands Breakdown
        let bands_title = text("Performance by Opponent Rating Band")
            .font(fonts::UI_BOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_PRIMARY);

        let bands_header = row![
            text("Rating Band").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(3)),
            text("Games").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
            text("Record (+W / =D / -L)").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(3)),
            text("Win Rate").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
            text("Win % Bar").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(4)),
        ]
        .padding([spacing::XS, spacing::SM]);

        let mut band_rows = column![].spacing(spacing::XXS);
        for band in &insights.rating_bands {
            let row_item = container(
                row![
                    text(&band.label).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(3)),
                    text(format!("{}", band.games_count)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(2)),
                    text(format!("+{} / ={} / -{}", band.wins, band.draws, band.losses)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_SECONDARY).width(Length::FillPortion(3)),
                    text(format!("{:.1}%", band.win_rate)).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::SUCCESS).width(Length::FillPortion(2)),
                    container(self.win_rate_bar(band.win_rate, 160.0)).width(Length::FillPortion(4)),
                ]
                .align_y(Alignment::Center)
                .padding([spacing::XS, spacing::SM])
            )
            .width(Length::Fill)
            .style(card_container_style);

            band_rows = band_rows.push(row_item);
        }

        let bands_card = container(
            column![
                bands_title,
                Space::with_height(spacing::XS),
                bands_header,
                band_rows,
            ]
            .spacing(spacing::XS)
        )
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(card_container_style);

        let scroll = scrollable(
            column![
                stat_cards,
                Space::with_height(spacing::MD),
                chart_card,
                Space::with_height(spacing::MD),
                bands_card,
            ]
            .spacing(spacing::XS)
        )
        .height(Length::Fill);

        scroll.into()
    }

    fn render_when_you_play<'a>(&'a self, insights: &'a OverallInsights) -> Element<'a, InsightsMessage> {
        // Weekday performance
        let weekday_title = text("Activity & Win Rate by Day of the Week")
            .font(fonts::UI_BOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_PRIMARY);

        let mut weekday_rows = column![].spacing(spacing::XXS);
        for stat in &insights.weekday_stats {
            let row_item = container(
                row![
                    text(&stat.label).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(3)),
                    text(format!("{} games", stat.games_count)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
                    text(format!("{} wins", stat.wins)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_SECONDARY).width(Length::FillPortion(2)),
                    text(format!("{:.1}% win rate", stat.win_rate)).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::SUCCESS).width(Length::FillPortion(3)),
                    container(self.win_rate_bar(stat.win_rate, 180.0)).width(Length::FillPortion(4)),
                ]
                .align_y(Alignment::Center)
                .padding([spacing::XS, spacing::SM])
            )
            .width(Length::Fill)
            .style(card_container_style);

            weekday_rows = weekday_rows.push(row_item);
        }

        let weekday_card = container(
            column![
                weekday_title,
                Space::with_height(spacing::XS),
                weekday_rows,
            ]
            .spacing(spacing::XS)
        )
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(card_container_style);

        // Time of day performance
        let tod_title = text("Activity & Win Rate by Time of Day")
            .font(fonts::UI_BOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_PRIMARY);

        let mut tod_rows = column![].spacing(spacing::XXS);
        for stat in &insights.time_of_day_stats {
            let row_item = container(
                row![
                    text(&stat.label).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(3)),
                    text(format!("{} games", stat.games_count)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
                    text(format!("{} wins", stat.wins)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_SECONDARY).width(Length::FillPortion(2)),
                    text(format!("{:.1}% win rate", stat.win_rate)).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::SUCCESS).width(Length::FillPortion(3)),
                    container(self.win_rate_bar(stat.win_rate, 180.0)).width(Length::FillPortion(4)),
                ]
                .align_y(Alignment::Center)
                .padding([spacing::XS, spacing::SM])
            )
            .width(Length::Fill)
            .style(card_container_style);

            tod_rows = tod_rows.push(row_item);
        }

        let tod_card = container(
            column![
                tod_title,
                Space::with_height(spacing::XS),
                tod_rows,
            ]
            .spacing(spacing::XS)
        )
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(card_container_style);

        // Habits card
        let habit_card = |name: &'static str, desc: &'static str, stat: String, is_good: bool| {
            let stat_color = if is_good { palette::SUCCESS } else { palette::DANGER };
            container(
                column![
                    row![
                        text(name).font(fonts::UI_BOLD).size(type_scale::HEADING).color(palette::TEXT_PRIMARY),
                        Space::with_width(Length::Fill),
                        text(stat).font(fonts::MONO_BOLD).size(type_scale::HEADING).color(stat_color),
                    ]
                    .align_y(Alignment::Center),
                    Space::with_height(spacing::XXS),
                    text(desc).font(fonts::UI).size(type_scale::BODY).color(palette::TEXT_SECONDARY),
                ]
            )
            .padding(spacing::MD)
            .width(Length::Fill)
            .style(card_container_style)
        };

        let habits_card = column![
            text("Tactical Patterns & In-Game Habits")
                .font(fonts::UI_BOLD)
                .size(type_scale::HEADING)
                .color(palette::TEXT_PRIMARY),
            Space::with_height(spacing::XS),
            habit_card(
                "Punishing Blunders",
                "How consistently you spot and exploit the opponent's blunders and tactical oversights.",
                format!("{}/100", insights.skill_scores.capitalization),
                insights.skill_scores.capitalization >= 50
            ),
            Space::with_height(spacing::SM),
            habit_card(
                "Tactical Discipline",
                "Frequency of blunders per game. Lower blunder count indicates solid board vision and safety checks.",
                format!("{:.1} blunders/game", insights.avg_blunders_per_game),
                insights.avg_blunders_per_game < 1.0
            ),
            Space::with_height(spacing::SM),
            habit_card(
                "Endgame Technique",
                "Converting positions when simplified down to 6 or fewer pieces.",
                format!("{}/100", insights.skill_scores.endgame),
                insights.skill_scores.endgame >= 60
            ),
        ];

        let scroll = scrollable(
            column![
                weekday_card,
                Space::with_height(spacing::MD),
                tod_card,
                Space::with_height(spacing::MD),
                container(habits_card).padding(spacing::MD).style(card_container_style),
            ]
            .spacing(spacing::XS)
        )
        .height(Length::Fill);

        scroll.into()
    }

    fn render_openings<'a>(&'a self, insights: &'a OverallInsights) -> Element<'a, InsightsMessage> {
        let title = text("Opening Repertoire Performance")
            .font(fonts::UI_BOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_PRIMARY);

        let header = row![
            text("Opening Name").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(5)),
            text("Games").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
            text("Win %").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
            text("Score").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(3)),
        ]
        .padding([spacing::XS, spacing::SM]);

        let mut opening_rows = column![].spacing(spacing::XXS);
        for op in &insights.top_openings {
            let row_item = container(
                row![
                    text(&op.name).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(5)),
                    text(format!("{}", op.count)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(2)),
                    text(format!("{:.1}%", op.win_rate)).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::SUCCESS).width(Length::FillPortion(2)),
                    text(format!("+{} / ={} / -{}", op.wins, op.draws, op.losses)).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_MUTED).width(Length::FillPortion(3)),
                ]
                .padding([spacing::XS, spacing::SM])
            )
            .width(Length::Fill)
            .style(card_container_style);

            opening_rows = opening_rows.push(row_item);
        }

        let scroll = scrollable(opening_rows).height(Length::Fixed(480.0));

        column![
            title,
            Space::with_height(spacing::SM),
            header,
            scroll,
        ]
        .spacing(spacing::XS)
        .into()
    }

    fn stat_box<'a>(&'a self, title: &'static str, val: &str) -> Element<'a, InsightsMessage> {
        container(
            column![
                text(title)
                    .font(fonts::UI)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_MUTED),
                Space::with_height(spacing::XXS),
                text(val.to_string())
                    .font(fonts::MONO_BOLD)
                    .size(type_scale::TITLE)
                    .color(palette::TEXT_PRIMARY),
            ]
        )
        .padding(spacing::SM)
        .width(Length::FillPortion(1))
        .style(card_container_style)
        .into()
    }

    fn win_rate_bar<'a>(&'a self, pct: f64, bar_width: f32) -> Element<'a, InsightsMessage> {
        let filled_w = (bar_width * (pct as f32 / 100.0)).clamp(0.0, bar_width);
        let empty_w = (bar_width - filled_w).max(0.0);

        let fill_color = if pct >= 60.0 {
            palette::SUCCESS
        } else if pct >= 45.0 {
            palette::ACCENT
        } else {
            palette::DANGER
        };

        let filled = container(Space::new(Length::Fixed(filled_w), Length::Fixed(8.0)))
            .style(move |_| container::Style {
                background: Some(fill_color.into()),
                border: border::rounded(3.0),
                ..Default::default()
            });

        let empty = container(Space::new(Length::Fixed(empty_w), Length::Fixed(8.0)))
            .style(|_| container::Style {
                background: Some(palette::BG_SURFACE.into()),
                border: border::rounded(3.0),
                ..Default::default()
            });

        row![filled, empty].width(Length::Fixed(bar_width)).into()
    }
}
