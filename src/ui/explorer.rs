use crate::openings::identify_opening;
use crate::ui::board::{BoardMessage, BoardWidget};
use crate::ui::game_layout::{render_unified_board_column, PlayerInfo};
use crate::ui::theme::{
    card_container_style, fonts, icon_play, icon_refresh_cw, icon_undo, icon_widget, palette,
    secondary_button_style, spacing, type_scale,
};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};
use shakmaty::{Chess, Position, Square};

#[derive(Debug, Clone)]
pub enum ExplorerMessage {
    Board(BoardMessage),
    PlayMove(String), // SAN
    ResetBoard,
    UndoMove,
}

#[derive(Debug, Clone)]
pub struct MoveStats {
    pub san: String,
    pub count: usize,
    pub white_wins: usize,
    pub draws: usize,
    pub black_wins: usize,
}

pub struct ExplorerScreen {
    pub board_widget: BoardWidget,
    pub move_history: Vec<(Chess, String)>, // (Pos, SAN)
    pub current_opening: String,
    pub current_eco: String,
}

impl ExplorerScreen {
    pub fn new() -> Self {
        Self {
            board_widget: BoardWidget::new(Chess::default()),
            move_history: Vec::new(),
            current_opening: "Starting Position".to_string(),
            current_eco: "".to_string(),
        }
    }

    pub fn update_opening(&mut self) {
        let sans: Vec<String> = self.move_history.iter().map(|(_, s)| s.clone()).collect();
        let m = identify_opening(&sans);
        self.current_opening = if m.name.is_empty() {
            "Custom Position".to_string()
        } else {
            m.name
        };
        self.current_eco = m.eco;
    }

    pub fn play_san(&mut self, san_str: &str) {
        let moves = self.board_widget.position.legal_moves();
        for m in moves {
            let s = shakmaty::san::SanPlus::from_move(self.board_widget.position.clone(), m.clone()).to_string();
            if s == san_str || s.trim_end_matches('+').trim_end_matches('#') == san_str.trim_end_matches('+').trim_end_matches('#') {
                let prev_pos = self.board_widget.position.clone();
                let from = m.from().unwrap_or(Square::E2);
                let to = m.to();
                if let Ok(next_pos) = self.board_widget.position.clone().play(m.clone()) {
                    let sound = crate::audio::sound_for_move(&m, &next_pos);
                    crate::audio::play_sound(sound);
                    self.board_widget.position = next_pos;
                    self.board_widget.last_move = Some((from, to));
                    self.move_history.push((prev_pos, san_str.to_string()));
                    self.update_opening();
                    return;
                }
            }
        }
    }

    pub fn undo(&mut self) {
        if let Some((pos, _)) = self.move_history.pop() {
            crate::audio::play_sound(crate::audio::SoundType::Move);
            self.board_widget.position = pos;
            self.board_widget.last_move = None;
            self.update_opening();
        }
    }

    pub fn reset(&mut self) {
        crate::audio::play_sound(crate::audio::SoundType::GameStart);
        self.board_widget.position = Chess::default();
        self.board_widget.last_move = None;
        self.board_widget.selected_square = None;
        self.move_history.clear();
        self.update_opening();
    }

    pub fn view<'a>(&'a self) -> Element<'a, ExplorerMessage> {
        let is_white_turn = self.board_widget.position.turn() == shakmaty::Color::White;

        let black_player = PlayerInfo::new("Black", false)
            .with_active_turn(!is_white_turn);

        let white_player = PlayerInfo::new("White", true)
            .with_active_turn(is_white_turn);

        let (top_player, bottom_player) = if self.board_widget.flipped {
            (&white_player, &black_player)
        } else {
            (&black_player, &white_player)
        };

        let board_el = render_unified_board_column(
            top_player,
            bottom_player,
            &self.board_widget.position,
            &self.board_widget,
            ExplorerMessage::Board,
            None,
        );

        let title = text("Position & Opening Explorer")
            .font(fonts::UI_BOLD)
            .size(type_scale::TITLE)
            .color(palette::TEXT_PRIMARY);

        let eco_text = if self.current_eco.is_empty() {
            String::new()
        } else {
            format!("[{}] ", self.current_eco)
        };

        let op_card = container(
            column![
                text("Identified Opening")
                    .font(fonts::UI)
                    .size(type_scale::CAPTION)
                    .color(palette::TEXT_MUTED),
                Space::with_height(spacing::XXS),
                text(format!("{}{}", eco_text, self.current_opening))
                    .font(fonts::UI_SEMIBOLD)
                    .size(type_scale::HEADING)
                    .color(palette::TEXT_PRIMARY),
            ]
            .spacing(spacing::XXS)
        )
        .padding(spacing::SM)
        .width(Length::Fixed(360.0))
        .style(card_container_style);

        // Legal candidate moves from current position
        let legal_moves = self.board_widget.position.legal_moves();
        let mut candidate_moves_col = column![
            row![
                text("Move").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::Fixed(80.0)),
                text("Status").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::FillPortion(1)),
                text("Action").font(fonts::UI_SEMIBOLD).size(type_scale::CAPTION).color(palette::TEXT_MUTED).width(Length::Fixed(80.0)),
            ]
            .padding([spacing::XXS, spacing::XS])
        ]
        .spacing(spacing::XXS);

        for m in legal_moves.iter().take(20) {
            let san_str = shakmaty::san::SanPlus::from_move(self.board_widget.position.clone(), m.clone()).to_string();
            let san_btn_str = san_str.clone();

            let play_content = row![
                icon_widget(icon_play(), 12.0, Some(palette::TEXT_PRIMARY)),
                text("Play").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
            ]
            .spacing(spacing::XXS)
            .align_y(Alignment::Center);

            let row_item = container(
                row![
                    text(san_str).font(fonts::MONO_BOLD).size(type_scale::BODY).color(palette::TEXT_PRIMARY).width(Length::Fixed(80.0)),
                    text("Legal Move").font(fonts::UI).size(type_scale::CAPTION).color(palette::SUCCESS).width(Length::FillPortion(1)),
                    button(play_content)
                        .padding([spacing::XXS, spacing::SM])
                        .style(secondary_button_style)
                        .on_press(ExplorerMessage::PlayMove(san_btn_str)),
                ]
                .spacing(spacing::SM)
                .align_y(Alignment::Center)
                .padding([spacing::XXS, spacing::XS])
            )
            .width(Length::Fill)
            .style(card_container_style);

            candidate_moves_col = candidate_moves_col.push(row_item);
        }

        let moves_scroll = scrollable(candidate_moves_col).height(Length::Fixed(300.0));

        let undo_content = row![
            icon_widget(icon_undo(), 14.0, Some(palette::TEXT_PRIMARY)),
            text("Undo Move").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let reset_content = row![
            icon_widget(icon_refresh_cw(), 14.0, Some(palette::TEXT_PRIMARY)),
            text("Reset Board").font(fonts::UI_SEMIBOLD).size(type_scale::BODY),
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center);

        let controls_row = row![
            button(undo_content)
                .padding([spacing::XS, spacing::SM])
                .style(secondary_button_style)
                .on_press(ExplorerMessage::UndoMove),
            button(reset_content)
                .padding([spacing::XS, spacing::SM])
                .style(secondary_button_style)
                .on_press(ExplorerMessage::ResetBoard),
        ]
        .spacing(spacing::SM);

        let sidebar = column![
            title,
            Space::with_height(spacing::XS),
            op_card,
            Space::with_height(spacing::SM),
            controls_row,
            Space::with_height(spacing::SM),
            text("Candidate Continuations")
                .font(fonts::UI_SEMIBOLD)
                .size(type_scale::HEADING)
                .color(palette::TEXT_SECONDARY),
            Space::with_height(spacing::XXS),
            moves_scroll,
        ]
        .spacing(spacing::XS);

        let content = row![
            board_el,
            Space::with_width(spacing::LG),
            sidebar,
        ]
        .padding(spacing::MD)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content).into()
    }
}
