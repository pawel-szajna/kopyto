use std::sync::LazyLock;
// use std::time::SystemTime;
// use rand::RngCore;
use crate::masks;
use crate::types::{Bitboard, Square};

// There is some magics search code commented out
// The best programming practices out there
// Some magics values were taken from chessprogramming.org

const MAGICS_BISHOP: [(u64, usize); 64] = [
    (0xcad28b1d1b6ffe00, 5),
    (0x866cdb595d07fd98, 4),
    (0x92c1d444ffec4900, 5),
    (0x81cc2d0a059f5dfe, 5),
    (0x9fb206100f57cede, 5),
    (0x98930b9ffe4e2ce0, 5),
    (0xf6a17114e7ff6e90, 4),
    (0xb6b86a2efba07ffa, 5),
    (0x78057426f9fd3fe8, 4),
    (0xe3cc05704d4267fc, 4),
    (0x81d7384384108760, 5),
    (0x7217044044839f24, 5),
    (0x0e508c0ce145bcc8, 5),
    (0x81a4be4709ff17a0, 5),
    (0x8d8abd4271a47f20, 4),
    (0x21a9471cdd9c3fea, 4),
    (0xf040060bd30adfe8, 4),
    (0xe816429f87c8efe0, 5),
    (0xc3a402e214097200, 7),
    (0x6b1c04980161a1a0, 7),
    (0xe95a0394021109e8, 7),
    (0x9bca00d10b722000, 7),
    (0xf63c0016f2fc8611, 5),
    (0x4a2e01939d9a9fe4, 4),
    (0xaccf4f4f3e29aa20, 5),
    (0xe3d01033c93e8548, 5),
    (0xec220802d10ec400, 7),
    (0x9e6e440042002e00, 10),
    (0x3f979c00006ffcc4, 10),
    (0x1793c1007201192a, 7),
    (0x431d074c650dbfa0, 5),
    (0xf223e3c645712400, 5),
    (0xbf0e2894c620204c, 5),
    (0x0bec1f287da03a10, 5),
    (0xe976f80203240400, 7),
    (0x1aa57ba00423de00, 10),
    (0x0ac6c9b2007c0050, 10),
    (0x3b0f1007001480e2, 7),
    (0xd32f082102ea0b03, 5),
    (0xdb10940f08698180, 5),
    (0xe62feb534a284050, 4),
    (0xeb3ff589a5d56000, 4),
    (0xa7fffebbd015f800, 7),
    (0x7e5d376011072808, 7),
    (0xb45161061400ae00, 7),
    (0x543c21075b00da00, 7),
    (0x1cbf59bd94a6bc08, 4),
    (0xaa9fccc6c8599200, 4),
    (0xa937fb155e3426c8, 4),
    (0xfeb3fd6b0fd96b2c, 4),
    (0xad1c9b49cc30276c, 5),
    (0xe1b8f4cae0882548, 5),
    (0xfbd4af6a9af40054, 5),
    (0xfb0ee8711b0e0360, 5),
    (0x67fdd9ebe4f6bb7c, 4),
    (0x8d7fece5d68939c0, 4),
    (0x8f47fefe342ee800, 5),
    (0xf613bff4ecfaf9a0, 4),
    (0x0d028d04c05c1058, 5),
    (0x468ae3e19d175800, 5),
    (0xbc0e0a3b1eb3cc00, 5),
    (0x9487554c7cc85e00, 5),
    (0x4960ffdbae958ea8, 4),
    (0x367f6652c0c1ac6c, 5),
];

const MAGICS_ROOK: [(u64, usize); 64] = [
    (0xc6aefc26577e46a0, 13),
    (0xa99c009b3aafd140, 12),
    (0x43dffe19b54c6ca0, 13),
    (0xf77b8227620a3bc8, 13),
    (0x0181817d780d8a90, 12),
    (0x869599118415425e, 13),
    (0xcff57fbd9c827180, 12),
    (0x6d96c547d78dd900, 13),
    (0xd1a58001ad4004d4, 12),
    (0x970b9fffe1ad8664, 11),
    (0x1cd3e4515721b8f8, 11),
    (0xea9ffde17d6a6d40, 11),
    (0x31660006549ffe00, 10),
    (0xec9749fdd449d74a, 11),
    (0xcab759f2ad9de550, 11),
    (0x8926a75efaa7e405, 12),
    (0x921888311342fae0, 12),
    (0xbd0443e700f82000, 11),
    (0xbfc93bde0354a8c8, 11),
    (0x8ceda1fff3664200, 11),
    (0x4b634600186a3600, 11),
    (0x744c37ff5ee49124, 11),
    (0x32c2540008197250, 10),
    (0x40ccc60001017584, 11),
    (0x1d833b9a17f403b0, 12),
    (0x8661b4bd3c323080, 11),
    (0xef17e90d1bc13330, 11),
    (0x4439cd15d122765c, 11),
    (0xe9c990ff001c7800, 11),
    (0x0639f9d6cf7ebc00, 11),
    (0xc724d983c41d67f0, 11),
    (0xf953a61229cdd9f0, 12),
    (0x404eb5f03a9d6b80, 12),
    (0xd30fb7ae01948580, 11),
    (0x011b2255f6f4ac70, 11),
    (0x1b7f388494901000, 11),
    (0xc372615b26a49800, 11),
    (0xe19d2bfa12121460, 11),
    (0xbf68f80324007e30, 10),
    (0xe1a0c93000d4d820, 12),
    (0x76ce302065a06000, 12),
    (0x546f5e4f681ffc00, 11),
    (0xb709a221607ae000, 11),
    (0x986ea56f21f0f000, 11),
    (0x096d88c3fba27800, 11),
    (0xf492bbac249fac00, 11),
    (0xdfa8aac56b02a8e0, 11),
    (0x99f4343090eee100, 12),
    (0x48FFFE99FECFAA00, 10), // (0x8f92047154310200, 11),
    (0x48FFFE99FECFAA00, 9),  // (0xb7e2030a89a3c200, 10),
    (0x497FFFADFF9C2E00, 9),  // (0xba9600be5683e600, 10),
    (0x613FFFDDFFCE9200, 9),  // (0x5e39cdfa40e03600, 10),
    (0xffffffe9ffe7ce00, 9),  // (0xc2374e351b1afa00, 11),
    (0xfffffff5fff3e600, 9),  // (0xc86df75c08707a00, 11),
    (0x0003ff95e5e6a4c0, 9),  // (0x8eabff6ed9cff400, 10),
    (0x510FFFF5F63C96A0, 10), // (0x39947b0587cc1e00, 11),
    (0xEBFFFFB9FF9FC526, 11), // (0x0d130180c2013522, 12),
    (0x61FFFEDDFEEDAEAE, 10), // (0x3029dc35f669feee, 12),
    (0x53BFFFEDFFDEB1A2, 10), // (0x01205310764d881e, 12),
    (0x127FFFB9FFDFB5F6, 10), // (0xfa807876bc52cade, 12),
    (0x411FFFDDFFDBF4D6, 10), // (0x0f3bb71e186a0026, 12),
    (0xc27425c119020012, 12),
    (0x0003ffef27eebe74, 10), // (0xabb29910020aa804, 11),
    (0x7645fffecbfea79e, 11), // (0xf10d9401c0e5068a, 12),
];

#[derive(Copy, Clone)]
struct Entry {
    shift: usize,
    offset: usize,
    magic: u64,
    mask: Bitboard,
}

impl Entry {
    const fn new() -> Self {
        Self {
            shift: 0,
            offset: 0,
            magic: 0,
            mask: Bitboard::EMPTY,
        }
    }

    fn get_index(&self, occupied: Bitboard) -> usize {
        self.offset + (((self.mask & occupied).bitboard * self.magic) as usize >> (64 - self.shift))
    }
}

pub struct Magics {
    table: Vec<u64>,
    magic: [Entry; 64],
}

impl Magics {
    const fn new() -> Self {
        Self {
            table: vec![],
            magic: [Entry::new(); 64],
        }
    }

    pub fn get(&self, idx: usize, occupied: Bitboard) -> u64 {
        self.table[self.magic[idx].get_index(occupied)]
    }
}

fn slide(idx: Square, diff: isize, boundary: Bitboard, occupied: Bitboard) -> Bitboard {
    let mut slider = Bitboard::from(idx);
    let mut attacks = Bitboard::EMPTY;

    while (slider & boundary).not_empty() {
        if diff >= 0 {
            slider <<= diff;
        } else {
            slider >>= -diff;
        }
        attacks |= slider;
        if (occupied & slider).not_empty() {
            break;
        }
    }

    attacks
}

pub fn bishop(idx: Square, occupied: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    attacks |= slide(idx, -7, !(masks::RANKS[0] | masks::FILES[7]), occupied);
    attacks |= slide(idx, -9, !(masks::RANKS[0] | masks::FILES[0]), occupied);
    attacks |= slide(idx, 7, !(masks::RANKS[7] | masks::FILES[0]), occupied);
    attacks |= slide(idx, 9, !(masks::RANKS[7] | masks::FILES[7]), occupied);
    attacks
}

pub fn rook(idx: Square, occupied: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    attacks |= slide(idx, -1, !masks::FILES[0], occupied);
    attacks |= slide(idx, 1, !masks::FILES[7], occupied);
    attacks |= slide(idx, 8, !masks::RANKS[7], occupied);
    attacks |= slide(idx, -8, !masks::RANKS[0], occupied);
    attacks
}

fn create_entry<F>(piece_idx: Square, magic: u64, shift: usize, entry: &mut Entry, generator: &F) -> Option<Vec<u64>>
where F: Fn(Square, Bitboard) -> Bitboard {
    let mut result = vec![];

    let file = piece_idx.file();
    let rank = piece_idx.rank();
    let edges = ((masks::RANKS[0] | masks::RANKS[7]) & !masks::RANKS[rank]) | ((masks::FILES[0] | masks::FILES[7]) & !masks::FILES[file]);
    let mask = generator(piece_idx, Bitboard::EMPTY) & !edges;

    let mut new_entry = Entry::new();
    new_entry.magic = magic;
    new_entry.mask = mask;
    new_entry.shift = shift;

    result.resize(1 << shift, 0);

    let mut occupied = Bitboard::EMPTY;

    loop {
        let idx = new_entry.get_index(occupied);
        let desired = generator(piece_idx, occupied);

        if result[idx] == 0 {
            result[idx] = desired.bitboard;
        }

        if result[idx] != desired.bitboard {
            return None;
        }

        occupied = Bitboard::from_u64(occupied.bitboard - mask.bitboard) & mask;

        if occupied.empty() {
            break;
        }
    }

    *entry = new_entry;
    Some(result)
}

fn create_entry_outer<F>(idx: usize, /*mut*/ shift: usize, magic: u64, entry: &mut Entry, table: &mut Vec<u64>, generator: &F)
where F: Fn(Square, Bitboard) -> Bitboard {
    // let start = SystemTime::now();
    // let mut last_report = 0;
    // let mut rng = rand::thread_rng();
    // let mut total_attempts = 0;
    // let mut best_attempt = None;
    //
    // loop {
    //     total_attempts += 1;
    //     if total_attempts % 100 == 0 {
    //         let duration = start.elapsed().unwrap().as_secs();
    //         if duration > last_report {
    //             last_report = duration;
    //             println!("info string idx {} time {} sec next shift {} current magic {:#018x} attempts {}", idx, duration, shift, entry.magic, total_attempts);
    //         }
    //     }
    //     let magic = rng.next_u64() * rng.next_u64() * rng.next_u64() * rng.next_u64();
    //     let attempt = create_entry(idx, magic, shift, entry, generator);
    //     if attempt.is_some() {
    //         best_attempt = attempt;
    //         println!("info string idx {} found magic: {:#018x} with shift {} after {} attempts", idx, magic, shift, total_attempts);
    //         shift -= 1;
    //     }
    //     if best_attempt.is_some() && (/*total_attempts > 1000*/ last_report > 200) {
    //         break;
    //     }
    // }

    let best_attempt = create_entry(Square::from(idx), magic, shift, entry, generator);

    let mut best_attempt = best_attempt.unwrap();
    entry.offset = table.len();
    table.append(&mut best_attempt);
}

fn create_magics<F>(/*kind: &str, shift: usize,*/ generator: &F, magics_source: &[(u64, usize); 64]) -> Magics
where F: Fn(Square, Bitboard) -> Bitboard {
    // let start = SystemTime::now();
    let mut result = Magics::new();

    for piece_idx in 0..64 {
        let (magic, shift) = magics_source[piece_idx];
        create_entry_outer(piece_idx, shift, magic, &mut result.magic[piece_idx], &mut result.table, generator);
        // create_entry_outer(piece_idx, shift, &mut result.magic[piece_idx], &mut result.table, generator);
    }

    // println!("info string {} magics generation took {} ms, data size {} kB", kind, start.elapsed().unwrap().as_millis(), result.table.len() * 8 / 1024);
    result
}

fn create_rook_magics() -> Magics {
    create_magics(/*"rook", 13,*/ &rook, &MAGICS_ROOK)
}

fn create_bishop_magics() -> Magics {
    create_magics(/*"bishop", 10,*/ &bishop, &MAGICS_BISHOP)
}

pub static ROOK_MAGICS: LazyLock<Magics> = LazyLock::new(|| create_rook_magics());
pub static BISHOP_MAGICS: LazyLock<Magics> = LazyLock::new(|| create_bishop_magics());
