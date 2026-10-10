use crate::insights::OverallInsights;
use crate::ui::theme::{
    card_container_style, fonts, nav_button_style, palette, spacing, type_scale,
};
use iced::widget::{
    button, column, container, row, scrollable, text, Space,
};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub enum InsightsMessage {
    SelectSection(InsightsSection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InsightsSection {
    #[default]
    Overview,
    Openings,
    Habits,
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
            tab_btn(InsightsSection::Openings, "Openings Performance"),
            tab_btn(InsightsSection::Habits, "Habits & Patterns"),
        ]
        .spacing(spacing::XS);

        let section_content: Element<'a, InsightsMessage> = match self.current_section {
            InsightsSection::Overview => self.render_overview(insights),
            InsightsSection::Openings => self.render_openings(insights),
            InsightsSection::Habits => self.render_habits(insights),
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

    fn render_habits<'a>(&'a self, insights: &'a OverallInsights) -> Element<'a, InsightsMessage> {
        let title = text("Habits & Tactical Patterns")
            .font(fonts::UI_BOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_PRIMARY);

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

        let content = column![
            title,
            Space::with_height(spacing::MD),
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

        content.into()
    }
}
