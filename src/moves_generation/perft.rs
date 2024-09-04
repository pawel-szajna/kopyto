use std::time::SystemTime;
use crate::board::Board;
use crate::moves_generation::generate_all;

pub fn perft(board: &mut Board, depth: usize) -> u64 {
    let start = SystemTime::now();
    let nodes = perft_impl(board, depth, true);
    let time_taken = start.elapsed().unwrap();
    let nps = if time_taken.as_nanos() > 0 {
        1000000000 * nodes as u128 / time_taken.as_nanos()
    } else {
        0
    };
    println!(
        "depth {} nodes {} time {} nps {}",
        depth,
        nodes,
        time_taken.as_millis(),
        if nps != 0 {
            format!("{}", nps)
        } else {
            String::from("?")
        }
    );
    nodes
}

fn perft_impl(board: &mut Board, depth: usize, init: bool) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let moves = generate_all(board);
    for m in moves {
        board.make_move(m.clone());
        let res = perft_impl(board, depth - 1, false);
        if init {
            println!("{:?}: {}", m, res);
        }
        nodes += res;
        board.unmake_move();
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::FenConsumer;

    fn perft_run(fen: &str, depth: usize, expected: u64) {
        let mut board = Board::from_fen(fen);
        assert_eq!(perft(&mut board, depth), expected);
    }

    fn perft_initial(depth: usize, expected: u64) {
        perft_run("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", depth, expected);
    }

    #[test]
    fn perft_initial_0() {
        perft_initial(0, 1);
    }

    #[test]
    fn perft_initial_1() {
        perft_initial(1, 20);
    }

    #[test]
    fn perft_initial_2() {
        perft_initial(2, 400);
    }

    #[test]
    fn perft_initial_3() {
        perft_initial(3, 8902);
    }

    #[test]
    fn perft_initial_4() {
        perft_initial(4, 197281);
    }

    #[test]
    fn perft_initial_5() {
        perft_initial(5, 4865609);
    }

    #[test]
    fn perft_initial_6() {
        perft_initial(6, 119060324);
    }

    fn perft_kiwipete(depth: usize, expected: u64) {
        perft_run("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0", depth, expected);
    }

    #[test]
    fn perft_kiwipete_1() {
        perft_kiwipete(1, 48);
    }

    #[test]
    fn perft_kiwipete_2() {
        perft_kiwipete(2, 2039);
    }

    #[test]
    fn perft_kiwipete_3() {
        perft_kiwipete(3, 97862);
    }

    #[test]
    fn perft_kiwipete_4() {
        perft_kiwipete(4, 4085603);
    }

    #[test]
    fn perft_kiwipete_5() {
        perft_kiwipete(5, 193690690);
    }

    fn perft_endgame(depth: usize, expected: u64) {
        perft_run("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 0", depth, expected);
    }

    #[test]
    fn perft_endgame_1() {
        perft_endgame(1, 14);
    }

    #[test]
    fn perft_endgame_2() {
        perft_endgame(2, 191);
    }

    #[test]
    fn perft_endgame_3() {
        perft_endgame(3, 2812);
    }

    #[test]
    fn perft_endgame_4() {
        perft_endgame(4, 43238);
    }

    #[test]
    fn perft_endgame_5() {
        perft_endgame(5, 674624);
    }

    #[test]
    fn perft_endgame_6() {
        perft_endgame(6, 11030083);
    }

    fn perft_pos4(depth: usize, expected: u64) {
        perft_run("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", depth, expected);
    }

    #[test]
    fn perft_pos4_1() {
        perft_pos4(1, 6);
    }

    #[test]
    fn perft_pos4_2() {
        perft_pos4(2, 264);
    }

    #[test]
    fn perft_pos4_3() {
        perft_pos4(3, 9467);
    }

    #[test]
    fn perft_pos4_4() {
        perft_pos4(4, 422333);
    }

    #[test]
    fn perft_pos4_5() {
        perft_pos4(5, 15833292);
    }

    fn perft_pos5(depth: usize, expected: u64) {
        perft_run("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", depth, expected);
    }

    #[test]
    fn perft_pos5_1() {
        perft_pos5(1, 44);
    }

    #[test]
    fn perft_pos5_2() {
        perft_pos5(2, 1486);
    }

    #[test]
    fn perft_pos5_3() {
        perft_pos5(3, 62379);
    }

    #[test]
    fn perft_pos5_4() {
        perft_pos5(4, 2103487);
    }

    #[test]
    fn perft_pos5_5() {
        perft_pos5(5, 89941194);
    }
}
