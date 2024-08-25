use crate::chess::board::Side;
use crate::chess::moves::{Move, Piece, Promotion};
use crate::chess::moves_generation::MoveGenerator;
use crate::chess::search::Search;
use crate::chess::{board, search, util};
use raylib::prelude::*;

type TexturePerColor = [Option<Texture2D>; 2];
const NO_TEXTURE: TexturePerColor = [None, None];

pub struct UI {
    board: board::Board,

    t_pawn: TexturePerColor,
    t_king: TexturePerColor,
    t_queen: TexturePerColor,
    t_rook: TexturePerColor,
    t_bishop: TexturePerColor,
    t_knight: TexturePerColor,

    legal_moves: Vec<Move>,

    evaluation: i64,
    side_cpu: Side,
}

const SQUARE: [Color; 2] = [
    Color::new(188, 143, 143, 255),
    Color::new(245, 233, 220, 255),
];
const SQUARE_LAST_MOVE: [Color; 2] = [
    Color::new(183, 188, 143, 255),
    Color::new(217, 222, 177, 255),
];
const SQUARE_LEGAL: [Color; 2] = [
    Color::new(158, 143, 188, 255),
    Color::new(207, 188, 214, 255),
];
const PROMOTION_BACKGROUND: Color = Color::new(255, 255, 255, 192);

#[derive(Copy, Clone)]
struct PieceInfo {
    x: i32,
    y: i32,
    file: usize,
    rank: usize,
    side: Side,
    kind: Piece,
}

impl UI {
    pub fn new() -> Self {
        Self {
            board: board::Board::from_starting_position(),

            t_pawn: NO_TEXTURE,
            t_king: NO_TEXTURE,
            t_queen: NO_TEXTURE,
            t_rook: NO_TEXTURE,
            t_bishop: NO_TEXTURE,
            t_knight: NO_TEXTURE,

            legal_moves: Vec::new(),

            evaluation: 0,
            side_cpu: board::BLACK,
        }
    }

    fn load_textures(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let mut load = |filename| {
            Some(
                rl.load_texture(&thread, filename)
                    .unwrap_or_else(|s| panic!("Cannot load texture: {}", s)),
            )
        };
        self.t_pawn = [load("res/plt60.png"), load("res/pdt60.png")];
        self.t_king = [load("res/klt60.png"), load("res/kdt60.png")];
        self.t_queen = [load("res/qlt60.png"), load("res/qdt60.png")];
        self.t_rook = [load("res/rlt60.png"), load("res/rdt60.png")];
        self.t_bishop = [load("res/blt60.png"), load("res/bdt60.png")];
        self.t_knight = [load("res/nlt60.png"), load("res/ndt60.png")];
    }

    fn unload_textures(&mut self) {
        self.t_pawn = NO_TEXTURE;
        self.t_king = NO_TEXTURE;
        self.t_queen = NO_TEXTURE;
        self.t_rook = NO_TEXTURE;
        self.t_bishop = NO_TEXTURE;
        self.t_knight = NO_TEXTURE;
    }

    pub fn run(&mut self) {
        let (mut rl, thread) = raylib::init().size(800, 600).title("kopyto UI").build();
        rl.set_target_fps(60);
        let mut pieces = Vec::<PieceInfo>::new();
        let mut current_piece: Option<PieceInfo> = None;
        let mut promotion_window: Option<(usize, usize)> = None;

        self.load_textures(&mut rl, &thread);

        while !rl.window_should_close() {
            pieces.clear();
            let mouse_x = rl.get_mouse_x();
            let mouse_y = rl.get_mouse_y();

            if self.board.side_to_move() == self.side_cpu {
                let result = self.board.search(search::Options::new());
                self.board.make_move(result.m);
                self.evaluation = result.score;
            }

            {
                let mut d = rl.begin_drawing(&thread);

                d.clear_background(Color::BLACK);
                d.draw_text("kopyto", 16, 16, 16, Color::WHITE);
                d.draw_text(
                    self.board.export_fen().as_str(),
                    16,
                    d.get_screen_height() - 32,
                    16,
                    Color::WHITE,
                );
                d.draw_fps(d.get_screen_width() - 108, 16);

                self.draw_board(&mut d, &mut pieces, &current_piece);

                if current_piece.is_some() && promotion_window.is_none() {
                    let current_piece = current_piece.unwrap();
                    self.draw_piece_graphics(
                        mouse_x - 30,
                        mouse_y - 30,
                        &mut d,
                        current_piece.side,
                        Color::WHITE,
                        current_piece.kind,
                    );
                }

                if promotion_window.is_some() {
                    let (target_file, _) = promotion_window.unwrap();
                    self.draw_promotion_window(&mut d, target_file);
                }
            }

            if current_piece.is_none() && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            {
                for piece in &pieces {
                    if mouse_x > piece.x
                        && mouse_x < piece.x + 60
                        && mouse_y > piece.y
                        && mouse_y < piece.y + 60
                    {
                        current_piece = Some(piece.clone());
                        let mut moves = self.board.generate_moves_for(piece.file, piece.rank);
                        self.board.prune_checks(self.board.side_to_move(), &mut moves);
                        self.legal_moves = moves.0;
                        break;
                    }
                }
            }

            if promotion_window.is_some()
                && rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
            {
                let piece = current_piece.unwrap();
                let (target_file, target_rank) = promotion_window.unwrap();
                let x = 160 + (target_file as i32) * 60;
                let y = 60;
                if mouse_x > x && mouse_x < x + 60 && mouse_y > y && mouse_y < y + 60 * 4 {
                    let target_piece = match mouse_y - y {
                        y if y < 60 * 1 => Promotion::Queen,
                        y if y < 60 * 2 => Promotion::Rook,
                        y if y < 60 * 3 => Promotion::Bishop,
                        _ => Promotion::Knight,
                    };
                    self.board.make_move(Move::from_idx_prom(
                        piece.rank * 8usize + piece.file,
                        target_rank * 8usize + target_file,
                        target_piece,
                    ));
                }
                current_piece = None;
                promotion_window = None;
                self.legal_moves.clear();
            }

            if current_piece.is_some()
                && promotion_window.is_none()
                && rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
            {
                if mouse_x > 160
                    && mouse_x < 160 + (8 * 60)
                    && mouse_y > 60
                    && mouse_y < 60 + (8 * 60)
                {
                    let target_file = (mouse_x as usize - 160) / 60;
                    let target_rank = 7 - (mouse_y as usize - 60) / 60;
                    let piece = current_piece.unwrap();
                    if target_file != piece.file || target_rank != piece.rank {
                        if piece.kind == Piece::Pawn
                            && target_rank
                                == if self.board.side_to_move() == board::WHITE {
                                    7
                                } else {
                                    0
                                }
                        {
                            promotion_window = Some((target_file, target_rank));
                        } else {
                            self.board.make_move(Move::from_idx(
                                piece.rank * 8usize + piece.file,
                                target_rank * 8usize + target_file,
                            ));
                            current_piece = None;
                            self.legal_moves.clear();
                        }
                    } else {
                        current_piece = None;
                        self.legal_moves.clear();
                    }
                } else {
                    current_piece = None;
                    self.legal_moves.clear();
                }
            }
        }

        self.unload_textures();
    }

    fn draw_promotion_window(&self, d: &mut RaylibDrawHandle, file: usize) {
        let x = 160 + (file as i32) * 60;
        let y = 60;
        let side = self.board.side_to_move();

        d.draw_rectangle(x, y, 60, 60 * 4, PROMOTION_BACKGROUND);
        d.draw_rectangle_lines(x, y, 60, 60 * 4, Color::BLACK);
        self.draw_piece_graphics(x, y, d, side, Color::WHITE, Piece::Queen);
        self.draw_piece_graphics(x, y + 60, d, side, Color::WHITE, Piece::Rook);
        self.draw_piece_graphics(x, y + 60 * 2, d, side, Color::WHITE, Piece::Bishop);
        self.draw_piece_graphics(x, y + 60 * 3, d, side, Color::WHITE, Piece::Knight);
    }

    fn draw_board(
        &self,
        d: &mut RaylibDrawHandle,
        pieces: &mut Vec<PieceInfo>,
        current_piece: &Option<PieceInfo>,
    ) {
        for rank in 0i32..8 {
            for file in 0i32..8 {
                let current_square = util::coords_to_mask(file as usize, rank as usize);
                let square_shade = ((rank + file) % 2) as usize;
                let mut color = SQUARE[square_shade];
                let last_move = self.board.last_move();
                if last_move.is_some() {
                    let last_move = last_move.unwrap();
                    if last_move.0 == current_square || last_move.1 == current_square {
                        color = SQUARE_LAST_MOVE[square_shade];
                    }
                }
                if self
                    .legal_moves
                    .iter()
                    .any(|m| m.get_to() == util::coords_to_idx(file as usize, rank as usize) as u16)
                {
                    color = SQUARE_LEGAL[square_shade];
                }
                let x = 160 + file * 60;
                let y = 60 + (7 - rank) * 60;
                d.draw_rectangle(x, y, 60, 60, color);
                self.draw_piece(
                    x,
                    y,
                    file as usize,
                    rank as usize,
                    d,
                    self.board.check_square(current_square),
                    pieces,
                    current_piece,
                );
            }
        }
    }

    fn draw_piece(
        &self,
        x: i32,
        y: i32,
        file: usize,
        rank: usize,
        d: &mut RaylibDrawHandle,
        piece: Option<(Side, Piece)>,
        pieces: &mut Vec<PieceInfo>,
        current_piece: &Option<PieceInfo>,
    ) {
        if piece.is_none() {
            return;
        }

        let (side, piece) = piece.unwrap();
        let mut color = Color::WHITE;
        if current_piece.is_some_and(|piece| piece.file == file && piece.rank == rank) {
            color.a = 64;
        }
        self.draw_piece_graphics(x, y, d, side, color, piece);
        if side == self.board.side_to_move() {
            pieces.push(PieceInfo {
                x,
                y,
                file,
                rank,
                side,
                kind: piece,
            });
        }
    }

    fn get_texture(&self, side: Side, piece: Piece) -> &Option<Texture2D> {
        match piece {
            Piece::King => &self.t_king[side],
            Piece::Queen => &self.t_queen[side],
            Piece::Rook => &self.t_rook[side],
            Piece::Bishop => &self.t_bishop[side],
            Piece::Knight => &self.t_knight[side],
            Piece::Pawn => &self.t_pawn[side],
        }
    }

    fn draw_piece_graphics(
        &self,
        x: i32,
        y: i32,
        d: &mut RaylibDrawHandle,
        side: Side,
        color: Color,
        piece: Piece,
    ) {
        d.draw_texture(self.get_texture(side, piece).as_ref().unwrap(), x, y, color);
    }
}
