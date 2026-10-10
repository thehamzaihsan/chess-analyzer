use iced::event::Status;
use iced::mouse::{self, Cursor};
use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Program, Stroke, Text};
use iced::widget::svg::Handle;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use shakmaty::{Chess, Color as PieceColor, File, Position, Rank, Role, Square};
use std::collections::HashMap;
use std::sync::OnceLock;

// Embedded Lichess cburnett pieces (CC BY-SA 3.0 Colin M.L. Burnett)
const SVG_WK: &[u8] = include_bytes!("../../assets/pieces/cburnett/wK.svg");
const SVG_WQ: &[u8] = include_bytes!("../../assets/pieces/cburnett/wQ.svg");
const SVG_WR: &[u8] = include_bytes!("../../assets/pieces/cburnett/wR.svg");
const SVG_WB: &[u8] = include_bytes!("../../assets/pieces/cburnett/wB.svg");
const SVG_WN: &[u8] = include_bytes!("../../assets/pieces/cburnett/wN.svg");
const SVG_WP: &[u8] = include_bytes!("../../assets/pieces/cburnett/wP.svg");

const SVG_BK: &[u8] = include_bytes!("../../assets/pieces/cburnett/bK.svg");
const SVG_BQ: &[u8] = include_bytes!("../../assets/pieces/cburnett/bQ.svg");
const SVG_BR: &[u8] = include_bytes!("../../assets/pieces/cburnett/bR.svg");
const SVG_BB: &[u8] = include_bytes!("../../assets/pieces/cburnett/bB.svg");
const SVG_BN: &[u8] = include_bytes!("../../assets/pieces/cburnett/bN.svg");
const SVG_BP: &[u8] = include_bytes!("../../assets/pieces/cburnett/bP.svg");

static PIECE_HANDLES: OnceLock<HashMap<(PieceColor, Role), Handle>> = OnceLock::new();

fn get_piece_handles() -> &'static HashMap<(PieceColor, Role), Handle> {
    PIECE_HANDLES.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert((PieceColor::White, Role::King), Handle::from_memory(SVG_WK));
        map.insert((PieceColor::White, Role::Queen), Handle::from_memory(SVG_WQ));
        map.insert((PieceColor::White, Role::Rook), Handle::from_memory(SVG_WR));
        map.insert((PieceColor::White, Role::Bishop), Handle::from_memory(SVG_WB));
        map.insert((PieceColor::White, Role::Knight), Handle::from_memory(SVG_WN));
        map.insert((PieceColor::White, Role::Pawn), Handle::from_memory(SVG_WP));

        map.insert((PieceColor::Black, Role::King), Handle::from_memory(SVG_BK));
        map.insert((PieceColor::Black, Role::Queen), Handle::from_memory(SVG_BQ));
        map.insert((PieceColor::Black, Role::Rook), Handle::from_memory(SVG_BR));
        map.insert((PieceColor::Black, Role::Bishop), Handle::from_memory(SVG_BB));
        map.insert((PieceColor::Black, Role::Knight), Handle::from_memory(SVG_BN));
        map.insert((PieceColor::Black, Role::Pawn), Handle::from_memory(SVG_BP));
        map
    })
}

#[derive(Debug, Clone)]
pub enum BoardMessage {
    SquareClicked(Square),
}

#[derive(Debug, Clone)]
pub struct BoardWidget {
    pub position: Chess,
    pub flipped: bool,
    pub selected_square: Option<Square>,
    pub last_move: Option<(Square, Square)>,
    pub best_move_arrow: Option<(Square, Square)>,
}

impl BoardWidget {
    pub fn position_fen(&self) -> String {
        shakmaty::fen::Fen::from_position(&self.position, shakmaty::EnPassantMode::Legal).to_string()
    }

    pub fn new(position: Chess) -> Self {
        Self {
            position,
            flipped: false,
            selected_square: None,
            last_move: None,
            best_move_arrow: None,
        }
    }

    pub fn view<'a>(&'a self, size: f32) -> Element<'a, BoardMessage> {
        canvas::Canvas::new(self)
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into()
    }

    /// Converts canvas (x, y) relative to board into chess Square
    fn point_to_square(&self, point: Point, board_size: f32) -> Option<Square> {
        if point.x < 0.0 || point.x >= board_size || point.y < 0.0 || point.y >= board_size {
            return None;
        }

        let sq_size = board_size / 8.0;
        let col = (point.x / sq_size).floor() as u8;
        let row = (point.y / sq_size).floor() as u8;

        if col >= 8 || row >= 8 {
            return None;
        }

        let file = if self.flipped { 7 - col } else { col };
        let rank = if self.flipped { row } else { 7 - row };

        let file = File::new(file as u32);
        let rank = Rank::new(rank as u32);
        Some(Square::from_coords(file, rank))
    }

    /// Converts chess Square into rectangle on canvas
    fn square_to_rect(&self, square: Square, sq_size: f32) -> Rectangle {
        let file = square.file() as u8;
        let rank = square.rank() as u8;

        let col = if self.flipped { 7 - file } else { file };
        let row = if self.flipped { rank } else { 7 - rank };

        Rectangle {
            x: col as f32 * sq_size,
            y: row as f32 * sq_size,
            width: sq_size,
            height: sq_size,
        }
    }
}

impl Program<BoardMessage> for BoardWidget {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (Status, Option<BoardMessage>) {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(cursor_pos) = cursor.position_in(bounds) {
                let board_size = bounds.width.min(bounds.height);
                if let Some(sq) = self.point_to_square(cursor_pos, board_size) {
                    return (Status::Captured, Some(BoardMessage::SquareClicked(sq)));
                }
            }
        }
        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let board_size = bounds.width.min(bounds.height);
        let sq_size = board_size / 8.0;

        let mut frame = Frame::new(renderer, Size::new(board_size, board_size));

        // Colors
        let light_color = Color::from_rgb8(235, 236, 208);
        let dark_color = Color::from_rgb8(119, 149, 86);
        let selected_color = Color::from_rgba8(186, 202, 68, 0.7);
        let last_move_color = Color::from_rgba8(245, 246, 130, 0.5);
        let check_color = Color::from_rgba8(224, 86, 91, 0.7);
        let dot_color = Color::from_rgba8(20, 20, 20, 0.25);
        let coord_light = Color::from_rgb8(119, 149, 86);
        let coord_dark = Color::from_rgb8(235, 236, 208);

        let pieces = get_piece_handles();
        let legal_moves = self.position.legal_moves();

        // 1. Draw Squares & Highlights
        for rank_idx in 0..8u8 {
            for file_idx in 0..8u8 {
                let file = File::new(file_idx as u32);
                let rank = Rank::new(rank_idx as u32);
                let square = Square::from_coords(file, rank);
                let rect = self.square_to_rect(square, sq_size);

                let is_light = (file_idx + rank_idx) % 2 != 0;
                let bg_color = if is_light { light_color } else { dark_color };
                frame.fill_rectangle(rect.position(), rect.size(), bg_color);

                // Highlight last move
                if let Some((from, to)) = self.last_move {
                    if square == from || square == to {
                        frame.fill_rectangle(rect.position(), rect.size(), last_move_color);
                    }
                }

                // Highlight selected square
                if self.selected_square == Some(square) {
                    frame.fill_rectangle(rect.position(), rect.size(), selected_color);
                }

                // Check highlight
                if self.position.is_check() {
                    if let Some(piece) = self.position.board().piece_at(square) {
                        if piece.role == Role::King && piece.color == self.position.turn() {
                            frame.fill_rectangle(rect.position(), rect.size(), check_color);
                        }
                    }
                }
            }
        }

        // 2. Draw Coordinates
        for i in 0..8u8 {
            let file_char = if self.flipped {
                (b'h' - i) as char
            } else {
                (b'a' + i) as char
            };
            let rank_char = if self.flipped {
                (b'1' + i) as char
            } else {
                (b'8' - i) as char
            };

            // Files on bottom row
            let file_is_light = (i + 0) % 2 != 0;
            frame.fill_text(Text {
                content: file_char.to_string(),
                position: Point::new(i as f32 * sq_size + sq_size - 12.0, board_size - 14.0),
                color: if file_is_light { coord_light } else { coord_dark },
                size: 11.0.into(),
                font: crate::ui::theme::fonts::MONO_BOLD,
                ..Default::default()
            });

            // Ranks on left column
            let rank_is_light = (0 + i) % 2 != 0;
            frame.fill_text(Text {
                content: rank_char.to_string(),
                position: Point::new(3.0, i as f32 * sq_size + 2.0),
                color: if rank_is_light { coord_light } else { coord_dark },
                size: 11.0.into(),
                font: crate::ui::theme::fonts::MONO_BOLD,
                ..Default::default()
            });
        }

        // 3. Draw Pieces
        for square in shakmaty::Square::ALL {
            if let Some(piece) = self.position.board().piece_at(square) {
                if let Some(handle) = pieces.get(&(piece.color, piece.role)) {
                    let rect = self.square_to_rect(square, sq_size);
                    let pad = sq_size * 0.04;
                    let piece_rect = Rectangle {
                        x: rect.x + pad,
                        y: rect.y + pad,
                        width: sq_size - pad * 2.0,
                        height: sq_size - pad * 2.0,
                    };
                    frame.draw_svg(piece_rect, handle);
                }
            }
        }

        // 4. Draw Legal Move Hints (Dots / Rings)
        if let Some(selected) = self.selected_square {
            for m in &legal_moves {
                if m.from() == Some(selected) {
                    let dest = m.to();
                    let dest_rect = self.square_to_rect(dest, sq_size);
                    let center = Point::new(dest_rect.x + sq_size / 2.0, dest_rect.y + sq_size / 2.0);

                    let is_capture = self.position.board().piece_at(dest).is_some() || m.is_en_passant();

                    if is_capture {
                        let path = Path::circle(center, sq_size * 0.42);
                        frame.stroke(&path, Stroke::default().with_color(dot_color).with_width(sq_size * 0.08));
                    } else {
                        let path = Path::circle(center, sq_size * 0.16);
                        frame.fill(&path, dot_color);
                    }
                }
            }
        }

        // 5. Draw Engine Best Move Arrow
        if let Some((from, to)) = self.best_move_arrow {
            let from_rect = self.square_to_rect(from, sq_size);
            let to_rect = self.square_to_rect(to, sq_size);
            let p1 = Point::new(from_rect.x + sq_size / 2.0, from_rect.y + sq_size / 2.0);
            let p2 = Point::new(to_rect.x + sq_size / 2.0, to_rect.y + sq_size / 2.0);

            let arrow_color = Color::from_rgba8(40, 180, 80, 0.75);
            let arrow_stroke = Stroke::default().with_color(arrow_color).with_width(sq_size * 0.14);

            let path = Path::line(p1, p2);
            frame.stroke(&path, arrow_stroke);

            let angle = (p2.y - p1.y).atan2(p2.x - p1.x);
            let head_len = sq_size * 0.28;
            let head_angle = std::f32::consts::PI / 6.0;

            let h1 = Point::new(
                p2.x - head_len * (angle - head_angle).cos(),
                p2.y - head_len * (angle - head_angle).sin(),
            );
            let h2 = Point::new(
                p2.x - head_len * (angle + head_angle).cos(),
                p2.y - head_len * (angle + head_angle).sin(),
            );

            let head_path = Path::new(|builder| {
                builder.move_to(p2);
                builder.line_to(h1);
                builder.line_to(h2);
                builder.close();
            });
            frame.fill(&head_path, arrow_color);
        }

        vec![frame.into_geometry()]
    }
}

fn role_value(role: Role) -> i32 {
    match role {
        Role::Pawn => 1,
        Role::Knight | Role::Bishop => 3,
        Role::Rook => 5,
        Role::Queen => 9,
        Role::King => 0,
    }
}

const STARTING_COUNTS: [(Role, u32); 5] = [
    (Role::Pawn, 8),
    (Role::Knight, 2),
    (Role::Bishop, 2),
    (Role::Rook, 2),
    (Role::Queen, 1),
];

/// Returns (pieces White has captured, pieces Black has captured, White's
/// net material lead in points — negative means Black is ahead).
pub fn captured_pieces(position: &Chess) -> (Vec<Role>, Vec<Role>, i32) {
    let mut white_on_board: HashMap<Role, u32> = HashMap::new();
    let mut black_on_board: HashMap<Role, u32> = HashMap::new();

    for square in Square::ALL {
        if let Some(piece) = position.board().piece_at(square) {
            let counts = if piece.color == PieceColor::White {
                &mut white_on_board
            } else {
                &mut black_on_board
            };
            *counts.entry(piece.role).or_insert(0) += 1;
        }
    }

    let mut captured_by_white = Vec::new(); // black pieces no longer on the board
    let mut captured_by_black = Vec::new(); // white pieces no longer on the board
    let mut white_advantage = 0i32;

    for (role, start) in STARTING_COUNTS {
        let white_missing = start.saturating_sub(*white_on_board.get(&role).unwrap_or(&0));
        let black_missing = start.saturating_sub(*black_on_board.get(&role).unwrap_or(&0));
        captured_by_white.extend(std::iter::repeat(role).take(black_missing as usize));
        captured_by_black.extend(std::iter::repeat(role).take(white_missing as usize));
        white_advantage += role_value(role) * (black_missing as i32 - white_missing as i32);
    }

    let by_value_desc = |r: &Role| std::cmp::Reverse(role_value(*r));
    captured_by_white.sort_by_key(by_value_desc);
    captured_by_black.sort_by_key(by_value_desc);

    (captured_by_white, captured_by_black, white_advantage)
}

/// A small row of captured-piece icons plus a "+N" material lead badge,
/// mirroring the capture tray Chess.com shows above/below the board.
/// `captured` holds the roles of the pieces this tray's owner has captured;
/// `captured_color` is the color those captured pieces were (i.e. the
/// opponent's color) so the right icon set is used.
pub fn captured_tray<'a, Message: 'a>(
    captured: &[Role],
    captured_color: PieceColor,
    advantage: Option<i32>,
    icon_size: f32,
) -> Element<'a, Message> {
    let handles = get_piece_handles();
    let mut tray = iced::widget::row![].spacing(2.0).align_y(iced::Alignment::Center);

    for role in captured {
        if let Some(handle) = handles.get(&(captured_color, *role)) {
            tray = tray.push(
                iced::widget::svg(handle.clone())
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            );
        }
    }

    if let Some(adv) = advantage {
        if adv > 0 {
            tray = tray.push(iced::widget::Space::with_width(4.0)).push(
                iced::widget::text(format!("+{}", adv))
                    .size(13)
                    .color(Color::from_rgba8(170, 170, 170, 1.0)),
            );
        }
    }

    tray.into()
}
