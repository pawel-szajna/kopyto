use std::str::Chars;

pub fn coords_to_mask(file: usize, rank: usize) -> u64 {
    1u64 << coords_to_idx(file, rank)
}

pub fn coords_to_idx(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

#[allow(dead_code)]
pub fn print_mask(title: &str, mask: u64) {
    let mut str = String::new();
    let mut line = String::new();

    for i in 0..65 {
        if i % 8 == 0 {
            str = format!("{}\n{}", line, str);
            line = String::new();
        }
        if i == 64 {
            break
        }
        line = format!("{} {}", line, if (1u64 << i) & mask != 0 { '1' } else { '.' });
    }

    println!("{}", title);
    println!("{}", str);
}

pub fn idx_to_str(idx: usize) -> String {
    let mut result = String::new();
    let file = idx % 8;
    let rank = (idx / 8) as u8;
    result.push(match file {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => panic!("invalid file"),
    });
    result.push(('0' as u8 + rank + 1) as char);
    result
}

#[allow(dead_code)]
pub fn mask_to_str(mask: u64) -> String {
    idx_to_str(mask.trailing_zeros() as usize)
}

pub fn str_to_idx(pos: &str) -> usize {
    fn get_file(pos: &mut Chars) -> usize {
        match pos.next() {
            Some('a') => 0,
            Some('b') => 1,
            Some('c') => 2,
            Some('d') => 3,
            Some('e') => 4,
            Some('f') => 5,
            Some('g') => 6,
            Some('h') => 7,
            _ => panic!("Invalid file"),
        }
    }
    fn get_rank(pos: &mut Chars) -> usize {
        match pos.next() {
            Some('1') => 0,
            Some('2') => 1,
            Some('3') => 2,
            Some('4') => 3,
            Some('5') => 4,
            Some('6') => 5,
            Some('7') => 6,
            Some('8') => 7,
            _ => panic!("Invalid rank"),
        }
    }
    let mut pos = pos.chars();
    let file = get_file(&mut pos);
    let rank = get_rank(&mut pos);
    coords_to_idx(file, rank)
}

pub fn eval_to_str(eval: i64) -> String {
    if eval.abs() > 9000 {
        format!("mate {}", eval.signum() * (1 + (10000 - eval.abs())) / 2)
    } else {
        format!("cp {}", eval)
    }
}
