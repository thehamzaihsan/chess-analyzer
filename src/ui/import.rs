use crate::db::StoredGame;
use crate::ui::theme::{
    card_container_style, fonts, icon_cpu, icon_download, icon_eye, icon_widget, palette,
    primary_button_style, secondary_button_style, spacing, text_input_style, type_scale,
};
use iced::widget::{
    button, column, container, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub enum ImportMessage {
    UsernameChanged(String),
    FetchChessCom,
    PgnInputChanged(String),
    LoadPgnText,
    LoadSamplePgn,
    SelectGameForReview(i64),
    StartGameAnalysis(i64),
    AnalyzeAllGames,
}

#[derive(Debug, Clone, Default)]
pub struct ImportScreen {
    pub username_input: String,
    pub pgn_input: String,
    pub status_message: Option<String>,
    pub is_loading: bool,
}

impl ImportScreen {
    pub fn new() -> Self {
        Self {
            username_input: String::new(),
            pgn_input: String::new(),
            status_message: None,
            is_loading: false,
        }
    }

    pub fn view<'a>(&'a self, games: &'a [StoredGame]) -> Element<'a, ImportMessage> {
        let title = text("Import Games & Archives")
            .font(fonts::UI_BOLD)
            .size(type_scale::TITLE)
            .color(palette::TEXT_PRIMARY);

        let user_label = text("Fetch from Chess.com Public API:")
            .font(fonts::UI_SEMIBOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_SECONDARY);

        let user_input = text_input(
            "Enter Chess.com username (e.g. ibn-ehsan)",
            &self.username_input,
        )
        .font(fonts::UI)
        .on_input(ImportMessage::UsernameChanged)
        .padding(spacing::SM)
        .width(Length::Fixed(340.0))
        .style(text_input_style);

        let fetch_content = row![
            icon_widget(icon_download(), 14.0, Some(palette::TEXT_PRIMARY)),
            text(if self.is_loading { "Fetching..." } else { "Fetch Archives" })
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY),
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
        )
        .padding([spacing::SM, spacing::MD])
        .style(secondary_button_style)
        .on_press(ImportMessage::LoadSamplePgn);

        let fetch_row = row![user_input, fetch_btn, sample_btn]
            .spacing(spacing::SM)
            .align_y(Alignment::Center);

        let pgn_label = text("Or paste PGN text directly:")
            .font(fonts::UI_SEMIBOLD)
            .size(type_scale::HEADING)
            .color(palette::TEXT_SECONDARY);

        let pgn_text_input = text_input("Paste PGN here...", &self.pgn_input)
            .font(fonts::MONO)
            .on_input(ImportMessage::PgnInputChanged)
            .padding(spacing::SM)
            .width(Length::Fill)
            .style(text_input_style);

        let parse_btn = button(
            text("Parse & Import")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
        )
        .padding([spacing::SM, spacing::MD])
        .style(primary_button_style)
        .on_press(ImportMessage::LoadPgnText);

        let pgn_row = row![pgn_text_input, parse_btn]
            .spacing(spacing::SM)
            .align_y(Alignment::Center);

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
        )
        .padding(spacing::MD)
        .style(card_container_style);

        let status_view: Element<'a, ImportMessage> = match &self.status_message {
            Some(msg) => text(msg)
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::BODY)
                .color(palette::ACCENT)
                .into(),
            None => Space::new(Length::Shrink, Length::Shrink).into(),
        };

        // Table header
        let analyze_all_content = row![
            icon_widget(icon_cpu(), 14.0, Some(palette::TEXT_PRIMARY)),
            text("Analyze All with Stockfish").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let table_title_row = row![
            text(format!("Stored Games ({})", games.len()))
                .font(fonts::UI_BOLD)
                .size(type_scale::HEADING)
                .color(palette::TEXT_PRIMARY),
            Space::with_width(Length::Fill),
            button(analyze_all_content)
                .padding([spacing::XS, spacing::SM])
                .style(primary_button_style)
                .on_press(ImportMessage::AnalyzeAllGames),
        ]
        .align_y(Alignment::Center);

        let header_row = row![
            text("White").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(3)),
            text("Black").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(3)),
            text("Result").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(1)),
            text("Opening").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(4)),
            text("Date").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
            text("Accuracy").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
            text("Actions").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(3)),
        ]
        .spacing(spacing::SM)
        .padding([spacing::XS, spacing::SM]);

        let mut games_list = column![].spacing(spacing::XXS);
        for game in games.iter().take(100) {
            let white_str = format!("{} ({})", game.white, game.white_elo.unwrap_or(0));
            let black_str = format!("{} ({})", game.black, game.black_elo.unwrap_or(0));
            let acc_str = match (game.accuracy_white, game.accuracy_black) {
                (Some(w), Some(b)) => format!("{:.0}% / {:.0}%", w, b),
                _ => "—".to_string(),
            };

            let game_id = game.id;
            let review_content = row![
                icon_widget(icon_eye(), 13.0, Some(palette::TEXT_PRIMARY)),
                text("Review").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let review_btn = button(review_content)
                .padding([spacing::XXS, spacing::SM])
                .style(secondary_button_style)
                .on_press(ImportMessage::SelectGameForReview(game_id));

            let analyze_content = row![
                icon_widget(icon_cpu(), 13.0, Some(palette::TEXT_PRIMARY)),
                text("Analyze").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let analyze_btn = button(analyze_content)
                .padding([spacing::XXS, spacing::SM])
                .style(primary_button_style)
                .on_press(ImportMessage::StartGameAnalysis(game_id));

            let row_item = container(
                row![
                    text(white_str).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(3)),
                    text(black_str).font(fonts::UI_SEMIBOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::FillPortion(3)),
                    text(&game.result).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::TEXT_SECONDARY).width(Length::FillPortion(1)),
                    text(&game.opening).font(fonts::UI).size(type_scale::BODY).color(palette::TEXT_SECONDARY).width(Length::FillPortion(4)),
                    text(&game.date).font(fonts::MONO).size(type_scale::BODY).color(palette::TEXT_MUTED).width(Length::FillPortion(2)),
                    text(acc_str).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::SUCCESS).width(Length::FillPortion(2)),
                    row![review_btn, analyze_btn].spacing(spacing::XS).width(Length::FillPortion(3)),
                ]
                .spacing(spacing::SM)
                .align_y(Alignment::Center)
                .padding([spacing::XS, spacing::SM])
            )
            .width(Length::Fill)
            .style(card_container_style);

            games_list = games_list.push(row_item);
        }

        let scrollable_table = scrollable(games_list).height(Length::Fill);

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
        ]
        .spacing(spacing::XS)
        .padding(spacing::MD);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
