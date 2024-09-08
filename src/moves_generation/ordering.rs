use crate::board::Board;
use crate::search;
use crate::types::{Bitboard, Move, Piece, Side};

type PieceValues = [Option<Piece>; 64];
type MoveWeight = i32;
pub type Weights = Vec<MoveWeight>;
type KillerMoveSet = [Move; search::KILLER_MOVES_STORED];

const HASH_MOVE_VALUE: MoveWeight = MoveWeight::MAX - 1;
const MVV_LVA_VALUE: MoveWeight = MoveWeight::MAX - 1024;
const KILLER_MOVE_VALUE: MoveWeight = MoveWeight::MAX - 4096;

pub fn order(board: &Board, moves: &Vec<Move>, hash_move: Option<Move>, killer_moves: &KillerMoveSet) -> Weights {
    let side = board.side_to_move();
    let attacks = board.occupied[!side];
    let pieces = cache_piece_values(board, side, moves);

    moves.iter().map(|m| {
        match hash_move {
            Some(hashed) if &hashed == m => HASH_MOVE_VALUE,
            _ => mvv_lva(m, attacks, &pieces, killer_moves),
        }
    }).collect()
}

fn mvv_lva(m: &Move, attacks: Bitboard, pieces: &PieceValues, killer_moves: &KillerMoveSet) -> MoveWeight {
    let target_mask = Bitboard::from(m.get_to());
    match (target_mask & attacks).not_empty() {
        false => {
            match killer_moves.contains(m) {
                true => KILLER_MOVE_VALUE,
                false => 0,
            }
        },
        true => {
            let defender_value = piece_value(pieces[m.get_to() as usize]);
            let attacker_value = piece_value(pieces[m.get_from() as usize]);
            MVV_LVA_VALUE + defender_value * 10 - attacker_value
        }
    }
}

fn piece_value(p: Option<Piece>) -> MoveWeight {
    match p {
        None => 0,
        Some(Piece::Pawn) => 10,
        Some(Piece::Knight) => 30,
        Some(Piece::Bishop) => 32,
        Some(Piece::Rook) => 50,
        Some(Piece::Queen) => 90,
        Some(Piece::King) => 50,
    }
}

fn cache_piece_values(board: &Board, side: Side, moves: &Vec<Move>) -> PieceValues {
    let mut verified = [false; 64];
    let mut pieces = [None; 64];

    for m in &*moves {
        let source = m.get_from();
        let target = m.get_to();
        if !verified[source] {
            pieces[source] = board.check_piece(side, Bitboard::from(source));
            verified[source] = true;
        }
        if !verified[target] {
            pieces[target] = board.check_piece(!side, Bitboard::from(target));
            verified[target] = true;
        }
    }

    pieces
}
