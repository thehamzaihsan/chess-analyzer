use crate::ui::theme::{board as board_theme, fonts, palette};
use iced::alignment::{Horizontal, Vertical};
use iced::event::Status;
use iced::mouse::{self, Cursor};
use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Program, Stroke, Text};
use iced::widget::svg::Handle;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use shakmaty::{Chess, Color as PieceColor, File, Position, Rank, Role, Square};
use std::collections::HashMap;
use std::sync::OnceLock;

// Embedded Lichess cardinal pieces (CC BY-NC-SA 4.0 sadsnake1)
const SVG_WK: &[u8] = include_bytes!("../../assets/pieces/cardinal/wK.svg");
const SVG_WQ: &[u8] = include_bytes!("../../assets/pieces/cardinal/wQ.svg");
const SVG_WR: &[u8] = include_bytes!("../../assets/pieces/cardinal/wR.svg");
const SVG_WB: &[u8] = include_bytes!("../../assets/pieces/cardinal/wB.svg");
const SVG_WN: &[u8] = include_bytes!("../../assets/pieces/cardinal/wN.svg");
const SVG_WP: &[u8] = include_bytes!("../../assets/pieces/cardinal/wP.svg");

const SVG_BK: &[u8] = include_bytes!("../../assets/pieces/cardinal/bK.svg");
const SVG_BQ: &[u8] = include_bytes!("../../assets/pieces/cardinal/bQ.svg");
const SVG_BR: &[u8] = include_bytes!("../../assets/pieces/cardinal/bR.svg");
const SVG_BB: &[u8] = include_bytes!("../../assets/pieces/cardinal/bB.svg");
const SVG_BN: &[u8] = include_bytes!("../../assets/pieces/cardinal/bN.svg");
const SVG_BP: &[u8] = include_bytes!("../../assets/pieces/cardinal/bP.svg");

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowColor {
    Green,   // Best move / engine recommendation
    Red,     // Threat / mistake
    Blue,    // Candidate / alternate
    Orange,  // Warning / caution
}

#[derive(Debug, Clone)]
pub struct BoardArrow {
    pub from: Square,
    pub to: Square,
    pub color: ArrowColor,
}

#[derive(Debug, Clone)]
pub struct BoardWidget {
    pub position: Chess,
    pub flipped: bool,
    pub selected_square: Option<Square>,
    pub last_move: Option<(Square, Square)>,
    pub best_move_arrow: Option<(Square, Square)>,
    pub custom_arrows: Vec<BoardArrow>,
    pub interactive: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoardCanvasState {
    dragging: Option<DragState>,
}

#[derive(Debug, Clone, Copy)]
struct DragState {
    from: Square,
    press_pos: Point,
    cursor_pos: Point,
    is_dragging: bool,
}

fn board_metrics(bounds: Rectangle) -> (f32, Point) {
    let board_size = bounds.width.min(bounds.height).max(0.0);
    let origin = Point::new(
        ((bounds.width - board_size) / 2.0).max(0.0),
        ((bounds.height - board_size) / 2.0).max(0.0),
    );
    (board_size, origin)
}

fn square_is_light(file_idx: u8, rank_idx: u8) -> bool {
    (file_idx + rank_idx) % 2 != 0
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
            custom_arrows: Vec::new(),
            interactive: true,
        }
    }

    pub fn view<'a>(&'a self, size: f32) -> Element<'a, BoardMessage> {
        canvas::Canvas::new(self)
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into()
    }

    pub fn view_responsive<'a>(&'a self) -> Element<'a, BoardMessage> {
        canvas::Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Converts canvas (x, y) relative to bounds into chess Square
    fn point_to_square_in_board(&self, point: Point, board_size: f32, origin: Point) -> Option<Square> {
        if board_size <= 0.0 {
            return None;
        }
        let local_x = point.x - origin.x;
        let local_y = point.y - origin.y;
        if local_x < 0.0 || local_x >= board_size || local_y < 0.0 || local_y >= board_size {
            return None;
        }

        let sq_size = board_size / 8.0;
        let col = (local_x / sq_size).floor() as u8;
        let row = (local_y / sq_size).floor() as u8;

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
    fn square_to_rect_in_board(&self, square: Square, sq_size: f32, origin: Point) -> Rectangle {
        let file = square.file() as u8;
        let rank = square.rank() as u8;

        let col = if self.flipped { 7 - file } else { file };
        let row = if self.flipped { rank } else { 7 - rank };

        Rectangle {
            x: origin.x + col as f32 * sq_size,
            y: origin.y + row as f32 * sq_size,
            width: sq_size,
            height: sq_size,
        }
    }
}

impl Program<BoardMessage> for BoardWidget {
    type State = BoardCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (Status, Option<BoardMessage>) {
        if !self.interactive {
            state.dragging = None;
            return (Status::Ignored, None);
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_pos) = cursor.position_in(bounds) {
                    let (board_size, origin) = board_metrics(bounds);
                    if let Some(sq) = self.point_to_square_in_board(cursor_pos, board_size, origin) {
                        if self.position.board().color_at(sq) == Some(self.position.turn()) {
                            state.dragging = Some(DragState {
                                from: sq,
                                press_pos: cursor_pos,
                                cursor_pos,
                                is_dragging: false,
                            });
                        } else {
                            state.dragging = None;
                        }
                        return (Status::Captured, Some(BoardMessage::SquareClicked(sq)));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(ref mut drag) = state.dragging {
                    if let Some(abs_pos) = cursor.position() {
                        let rel = Point::new(abs_pos.x - bounds.x, abs_pos.y - bounds.y);
                        drag.cursor_pos = rel;
                        let dx = rel.x - drag.press_pos.x;
                        let dy = rel.y - drag.press_pos.y;
                        if !drag.is_dragging && (dx * dx + dy * dy) > 9.0 {
                            drag.is_dragging = true;
                        }
                        return (Status::Captured, None);
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(drag) = state.dragging.take() {
                    if drag.is_dragging {
                        if let Some(cursor_pos) = cursor.position_in(bounds) {
                            let (board_size, origin) = board_metrics(bounds);
                            if let Some(to_sq) = self.point_to_square_in_board(cursor_pos, board_size, origin) {
                                if to_sq != drag.from {
                                    return (Status::Captured, Some(BoardMessage::SquareClicked(to_sq)));
                                }
                            }
                        }
                    }
                    return (Status::Captured, None);
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if state.dragging.take().is_some() {
                    return (Status::Captured, None);
                }
            }
            _ => {}
        }
        (Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if !self.interactive {
            return mouse::Interaction::default();
        }

        if state.dragging.as_ref().is_some_and(|d| d.is_dragging) {
            return mouse::Interaction::Grabbing;
        }

        if let Some(cursor_pos) = cursor.position_in(bounds) {
            let (board_size, origin) = board_metrics(bounds);
            if let Some(sq) = self.point_to_square_in_board(cursor_pos, board_size, origin) {
                let legal_moves = self.position.legal_moves();
                let is_movable_piece = self.position.board().color_at(sq) == Some(self.position.turn())
                    && legal_moves.iter().any(|m| m.from() == Some(sq));
                if is_movable_piece {
                    return mouse::Interaction::Grab;
                }
                if self
                    .selected_square
                    .is_some_and(|sel| legal_moves.iter().any(|m| m.from() == Some(sel) && m.to() == sq))
                {
                    return mouse::Interaction::Pointer;
                }
            }
        }

        mouse::Interaction::default()
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let (board_size, origin) = board_metrics(bounds);
        if board_size < 16.0 {
            return vec![frame.into_geometry()];
        }
        let sq_size = board_size / 8.0;

        let pieces = get_piece_handles();
        let legal_moves = self.position.legal_moves();

        let active_drag = state.dragging.filter(|d| d.is_dragging);
        let drag_from = active_drag.map(|d| d.from);
        let drag_hover_sq = active_drag.and_then(|d| {
            cursor
                .position_in(bounds)
                .and_then(|pos| self.point_to_square_in_board(pos, board_size, origin))
                .filter(|sq| *sq != d.from)
        });

        let in_check = self.position.is_check();
        let is_mate = self.position.is_checkmate();
        let turn = self.position.turn();

        // 1. Draw Squares & Layered Highlights
        // Layer order: Base square -> Last move -> Check / Mate -> Selected (over last-move) -> Drag hover border
        for rank_idx in 0..8u8 {
            for file_idx in 0..8u8 {
                let file = File::new(file_idx as u32);
                let rank = Rank::new(rank_idx as u32);
                let square = Square::from_coords(file, rank);
                let rect = self.square_to_rect_in_board(square, sq_size, origin);

                let bg_color = if square_is_light(file_idx, rank_idx) {
                    board_theme::LIGHT_SQUARE
                } else {
                    board_theme::DARK_SQUARE
                };
                frame.fill_rectangle(rect.position(), rect.size(), bg_color);

                // Last-move highlight
                if let Some((from, to)) = self.last_move {
                    if square == from || square == to {
                        frame.fill_rectangle(rect.position(), rect.size(), board_theme::LAST_MOVE);
                    }
                }

                // Check / Checkmate highlight on the checked/mated King's square
                let is_checked_king = in_check
                    && self
                        .position
                        .board()
                        .piece_at(square)
                        .is_some_and(|p| p.role == Role::King && p.color == turn);
                if is_checked_king {
                    if is_mate {
                        frame.fill_rectangle(
                            rect.position(),
                            rect.size(),
                            Color::from_rgba(220.0 / 255.0, 38.0 / 255.0, 38.0 / 255.0, 0.90),
                        );
                        let inset = (sq_size * 0.05).max(2.0);
                        let mate_border = Path::rectangle(
                            Point::new(rect.x + inset / 2.0, rect.y + inset / 2.0),
                            Size::new(rect.width - inset, rect.height - inset),
                        );
                        frame.stroke(
                            &mate_border,
                            Stroke::default()
                                .with_color(Color::from_rgb8(255, 110, 110))
                                .with_width(inset),
                        );
                    } else {
                        frame.fill_rectangle(rect.position(), rect.size(), board_theme::CHECK);
                    }
                }

                // Selected-square highlight (layered over last-move; preserves check glow if King is in check)
                if self.selected_square == Some(square) {
                    if !is_checked_king {
                        frame.fill_rectangle(rect.position(), rect.size(), board_theme::SELECTED);
                    }
                    let inset = (sq_size * 0.04).max(1.5);
                    let border_path = Path::rectangle(
                        Point::new(rect.x + inset / 2.0, rect.y + inset / 2.0),
                        Size::new(rect.width - inset, rect.height - inset),
                    );
                    let border_color = if is_checked_king {
                        board_theme::SELECTED
                    } else {
                        board_theme::SELECTED_BORDER
                    };
                    frame.stroke(
                        &border_path,
                        Stroke::default().with_color(border_color).with_width(inset),
                    );
                }

                // Drag hover target highlight
                if drag_hover_sq == Some(square) {
                    let bw = (sq_size * 0.05).max(2.0);
                    let hover_path = Path::rectangle(
                        Point::new(rect.x + bw / 2.0, rect.y + bw / 2.0),
                        Size::new(rect.width - bw, rect.height - bw),
                    );
                    frame.stroke(
                        &hover_path,
                        Stroke::default()
                            .with_color(board_theme::DRAG_TARGET_BORDER)
                            .with_width(bw),
                    );
                }
            }
        }

        // Subtle outer border hugging the square board
        let board_border = Path::rectangle(origin, Size::new(board_size, board_size));
        frame.stroke(
            &board_border,
            Stroke::default().with_color(palette::BORDER).with_width(2.0),
        );

        // 2. Draw Coordinates (scaled with square size, opposite square color, flipped-safe)
        let coord_size = (sq_size * 0.19).clamp(9.0, 18.0);
        let coord_margin = (sq_size * 0.06).max(2.0);

        for i in 0..8u8 {
            // File labels on bottom row (row = 7, col = i)
            let file_idx = if self.flipped { 7 - i } else { i };
            let bottom_rank_idx = if self.flipped { 7 } else { 0 };
            let file_is_light = square_is_light(file_idx, bottom_rank_idx);
            let file_char = (b'a' + file_idx) as char;

            frame.fill_text(Text {
                content: file_char.to_string(),
                position: Point::new(
                    origin.x + (i as f32 + 1.0) * sq_size - coord_margin,
                    origin.y + board_size - coord_margin,
                ),
                color: if file_is_light {
                    board_theme::DARK_SQUARE
                } else {
                    board_theme::LIGHT_SQUARE
                },
                size: coord_size.into(),
                font: fonts::MONO_BOLD,
                horizontal_alignment: Horizontal::Right,
                vertical_alignment: Vertical::Bottom,
                ..Default::default()
            });

            // Rank labels on left column (col = 0, row = i)
            let left_file_idx = if self.flipped { 7 } else { 0 };
            let rank_idx = if self.flipped { i } else { 7 - i };
            let rank_is_light = square_is_light(left_file_idx, rank_idx);
            let rank_char = (b'1' + rank_idx) as char;

            frame.fill_text(Text {
                content: rank_char.to_string(),
                position: Point::new(
                    origin.x + coord_margin,
                    origin.y + i as f32 * sq_size + coord_margin,
                ),
                color: if rank_is_light {
                    board_theme::DARK_SQUARE
                } else {
                    board_theme::LIGHT_SQUARE
                },
                size: coord_size.into(),
                font: fonts::MONO_BOLD,
                horizontal_alignment: Horizontal::Left,
                vertical_alignment: Vertical::Top,
                ..Default::default()
            });
        }

        // 3. Draw Stationary Pieces (hide origin piece while actively dragging)
        for square in shakmaty::Square::ALL {
            if drag_from == Some(square) {
                continue;
            }
            if let Some(piece) = self.position.board().piece_at(square) {
                if let Some(handle) = pieces.get(&(piece.color, piece.role)) {
                    let rect = self.square_to_rect_in_board(square, sq_size, origin);
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

        // 4. Draw Legal Move Hints (deduplicated for promotions)
        if let Some(selected) = self.selected_square {
            let mut seen_dests = [false; 64];
            for m in &legal_moves {
                if m.from() == Some(selected) {
                    let dest = m.to();
                    let dest_idx = dest as usize;
                    if seen_dests[dest_idx] {
                        continue;
                    }
                    seen_dests[dest_idx] = true;

                    let dest_rect = self.square_to_rect_in_board(dest, sq_size, origin);
                    let center = Point::new(dest_rect.x + sq_size / 2.0, dest_rect.y + sq_size / 2.0);
                    let is_capture = self.position.board().piece_at(dest).is_some() || m.is_en_passant();

                    if is_capture {
                        let path = Path::circle(center, sq_size * 0.42);
                        frame.stroke(
                            &path,
                            Stroke::default()
                                .with_color(board_theme::LEGAL_DOT)
                                .with_width((sq_size * 0.085).max(2.0)),
                        );
                    } else {
                        let path = Path::circle(center, sq_size * 0.16);
                        frame.fill(&path, board_theme::LEGAL_DOT);
                    }
                }
            }
        }

        // 5. Draw Engine & Custom Arrows
        if let Some((from, to)) = self.best_move_arrow {
            draw_chess_arrow(&mut frame, from, to, ArrowColor::Green, self.flipped, sq_size, origin);
        }
        for arrow in &self.custom_arrows {
            draw_chess_arrow(&mut frame, arrow.from, arrow.to, arrow.color, self.flipped, sq_size, origin);
        }

        // 6. Draw Dragged Piece Under Cursor (on top of everything)
        if let Some(drag) = active_drag {
            if let Some(piece) = self.position.board().piece_at(drag.from) {
                if let Some(handle) = pieces.get(&(piece.color, piece.role)) {
                    let drag_size = sq_size * 1.04;
                    let piece_rect = Rectangle {
                        x: drag.cursor_pos.x - drag_size / 2.0,
                        y: drag.cursor_pos.y - drag_size / 2.0,
                        width: drag_size,
                        height: drag_size,
                    };
                    frame.draw_svg(piece_rect, handle);
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

fn draw_chess_arrow(
    frame: &mut Frame,
    from: Square,
    to: Square,
    color_type: ArrowColor,
    flipped: bool,
    sq_size: f32,
    origin: Point,
) {
    let file1 = from.file() as u8;
    let rank1 = from.rank() as u8;
    let col1 = if flipped { 7 - file1 } else { file1 };
    let row1 = if flipped { rank1 } else { 7 - rank1 };

    let file2 = to.file() as u8;
    let rank2 = to.rank() as u8;
    let col2 = if flipped { 7 - file2 } else { file2 };
    let row2 = if flipped { rank2 } else { 7 - rank2 };

    let p1 = Point::new(
        origin.x + col1 as f32 * sq_size + sq_size / 2.0,
        origin.y + row1 as f32 * sq_size + sq_size / 2.0,
    );
    let p2 = Point::new(
        origin.x + col2 as f32 * sq_size + sq_size / 2.0,
        origin.y + row2 as f32 * sq_size + sq_size / 2.0,
    );

    let arrow_fill = match color_type {
        ArrowColor::Green => board_theme::ARROW_GREEN,
        ArrowColor::Red => board_theme::ARROW_RED,
        ArrowColor::Blue => board_theme::ARROW_BLUE,
        ArrowColor::Orange => board_theme::ARROW_ORANGE,
    };

    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1.0 {
        return;
    }

    let df = (file1 as i32 - file2 as i32).abs();
    let dr = (rank1 as i32 - rank2 as i32).abs();
    let is_knight = (df == 1 && dr == 2) || (df == 2 && dr == 1);

    let head_len = sq_size * 0.34;
    let head_width = sq_size * 0.44;
    let shaft_width = sq_size * 0.17;
    let hw = shaft_width / 2.0;
    let start_offset = sq_size * 0.25;
    let tip_inset = sq_size * 0.08;

    if is_knight {
        // L-shaped knight arrow: 2 squares along the long axis first, then 1 square perpendicular
        let corner = if dr == 2 {
            Point::new(p1.x, p2.y)
        } else {
            Point::new(p2.x, p1.y)
        };

        let d1x = corner.x - p1.x;
        let d1y = corner.y - p1.y;
        let dist1 = (d1x * d1x + d1y * d1y).sqrt();
        let d2x = p2.x - corner.x;
        let d2y = p2.y - corner.y;
        let dist2 = (d2x * d2x + d2y * d2y).sqrt();
        if dist1 < 1.0 || dist2 < 1.0 {
            return;
        }

        let u1 = (d1x / dist1, d1y / dist1);
        let perp1 = (-u1.1, u1.0);
        let u2 = (d2x / dist2, d2y / dist2);
        let perp2 = (-u2.1, u2.0);

        let shaft_start = Point::new(p1.x + u1.0 * start_offset, p1.y + u1.1 * start_offset);
        let tip = Point::new(p2.x - u2.0 * tip_inset, p2.y - u2.1 * tip_inset);
        let base_c = Point::new(tip.x - u2.0 * head_len, tip.y - u2.1 * head_len);

        let plus_corner = Point::new(
            corner.x + (perp1.0 + perp2.0) * hw,
            corner.y + (perp1.1 + perp2.1) * hw,
        );
        let minus_corner = Point::new(
            corner.x - (perp1.0 + perp2.0) * hw,
            corner.y - (perp1.1 + perp2.1) * hw,
        );

        let arrow_path = Path::new(|builder| {
            builder.move_to(Point::new(shaft_start.x + perp1.0 * hw, shaft_start.y + perp1.1 * hw));
            builder.line_to(plus_corner);
            builder.line_to(Point::new(base_c.x + perp2.0 * hw, base_c.y + perp2.1 * hw));
            builder.line_to(Point::new(
                base_c.x + perp2.0 * (head_width / 2.0),
                base_c.y + perp2.1 * (head_width / 2.0),
            ));
            builder.line_to(tip);
            builder.line_to(Point::new(
                base_c.x - perp2.0 * (head_width / 2.0),
                base_c.y - perp2.1 * (head_width / 2.0),
            ));
            builder.line_to(Point::new(base_c.x - perp2.0 * hw, base_c.y - perp2.1 * hw));
            builder.line_to(minus_corner);
            builder.line_to(Point::new(shaft_start.x - perp1.0 * hw, shaft_start.y - perp1.1 * hw));
            builder.close();
        });
        frame.fill(&arrow_path, arrow_fill);
    } else {
        let u = (dx / dist, dy / dist);
        let perp = (-u.1, u.0);

        let shaft_start = Point::new(p1.x + u.0 * start_offset, p1.y + u.1 * start_offset);
        let tip = Point::new(p2.x - u.0 * tip_inset, p2.y - u.1 * tip_inset);
        let base_c = Point::new(tip.x - u.0 * head_len, tip.y - u.1 * head_len);

        let arrow_path = Path::new(|builder| {
            builder.move_to(Point::new(shaft_start.x + perp.0 * hw, shaft_start.y + perp.1 * hw));
            builder.line_to(Point::new(base_c.x + perp.0 * hw, base_c.y + perp.1 * hw));
            builder.line_to(Point::new(
                base_c.x + perp.0 * (head_width / 2.0),
                base_c.y + perp.1 * (head_width / 2.0),
            ));
            builder.line_to(tip);
            builder.line_to(Point::new(
                base_c.x - perp.0 * (head_width / 2.0),
                base_c.y - perp.1 * (head_width / 2.0),
            ));
            builder.line_to(Point::new(base_c.x - perp.0 * hw, base_c.y - perp.1 * hw));
            builder.line_to(Point::new(shaft_start.x - perp.0 * hw, shaft_start.y - perp.1 * hw));
            builder.close();
        });
        frame.fill(&arrow_path, arrow_fill);
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

/// The standard chess board layout used by every screen that shows a board:
/// a captured-piece tray above, the board itself, and a captured-piece tray
/// below — keeps Game Review, Play Bots, Explorer, and Puzzles visually
/// consistent instead of each screen hand-rolling its own wrapping.
///
/// `board_el` is the caller's already-mapped `BoardWidget::view(...)`
/// output (mapping is screen-specific, so it's built by the caller);
/// `position` is read separately to compute the capture trays.
pub fn board_panel<'a, Message: 'a>(position: &Chess, board_el: Element<'a, Message>) -> Element<'a, Message> {
    let (captured_by_white, captured_by_black, white_advantage) = captured_pieces(position);

    let black_tray = captured_tray(
        &captured_by_black,
        PieceColor::White,
        if white_advantage < 0 { Some(-white_advantage) } else { None },
        18.0,
    );
    let white_tray = captured_tray(
        &captured_by_white,
        PieceColor::Black,
        if white_advantage > 0 { Some(white_advantage) } else { None },
        18.0,
    );

    iced::widget::column![
        black_tray,
        iced::widget::Space::with_height(4.0),
        board_el,
        iced::widget::Space::with_height(4.0),
        white_tray,
    ]
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_metrics_centered_square() {
        let (size, origin) = board_metrics(Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)));
        assert_eq!(size, 600.0);
        assert_eq!(origin, Point::new(100.0, 0.0));

        let (size2, origin2) = board_metrics(Rectangle::new(Point::ORIGIN, Size::new(450.0, 650.0)));
        assert_eq!(size2, 450.0);
        assert_eq!(origin2, Point::new(0.0, 100.0));
    }

    #[test]
    fn test_square_point_roundtrip_unflipped_and_flipped() {
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(720.0, 560.0));
        let (board_size, origin) = board_metrics(bounds);
        let sq_size = board_size / 8.0;

        for flipped in [false, true] {
            let mut widget = BoardWidget::new(Chess::default());
            widget.flipped = flipped;

            for sq in Square::ALL {
                let rect = widget.square_to_rect_in_board(sq, sq_size, origin);
                let center = Point::new(rect.x + sq_size / 2.0, rect.y + sq_size / 2.0);
                assert_eq!(
                    widget.point_to_square_in_board(center, board_size, origin),
                    Some(sq),
                    "roundtrip failed for {:?} (flipped={})",
                    sq,
                    flipped
                );
            }

            // Outside the square board area returns None
            assert_eq!(
                widget.point_to_square_in_board(Point::new(10.0, 10.0), board_size, origin),
                None
            );
        }
    }

    #[test]
    fn test_coordinate_square_parity() {
        // a1 is dark, a8 is light, h1 is light, h8 is dark
        assert!(!square_is_light(0, 0)); // a1
        assert!(square_is_light(0, 7));  // a8
        assert!(square_is_light(7, 0));  // h1
        assert!(!square_is_light(7, 7)); // h8
    }

    #[test]
    fn test_all_twelve_piece_handles_loaded() {
        let handles = get_piece_handles();
        assert_eq!(handles.len(), 12);
    }
}
