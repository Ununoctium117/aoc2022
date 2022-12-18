use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

struct RepeatingIterator<T> {
    idx: usize,
    items: Vec<T>,
}
impl<T: Clone> Iterator for RepeatingIterator<T> {
    type Item = (T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.items[self.idx].clone();
        let idx = self.idx;
        self.idx = (self.idx + 1) % self.items.len();
        Some((result, idx))
    }
}

mod rocks {
    #[derive(Clone)]
    pub struct Rock {
        // each byte is a row; columns are bits 3210, in that order
        positions: [u8; 4],
        pub width: usize,
        pub height: usize,
    }
    impl Rock {
        pub fn get_mask(&self, row: usize) -> u8 {
            self.positions[row]
        }
    }

    const FLAT: Rock = Rock {
        positions: [0b1111, 0b0000, 0b0000, 0b0000],
        width: 4,
        height: 1,
    };

    const CROSS: Rock = Rock {
        positions: [0b0010, 0b0111, 0b0010, 0b0000],
        width: 3,
        height: 3,
    };

    const L: Rock = Rock {
        positions: [0b0111, 0b0001, 0b0001, 0b0000],
        width: 3,
        height: 3,
    };

    const VERTICAL: Rock = Rock {
        positions: [0b0001, 0b0001, 0b0001, 0b0001],
        width: 1,
        height: 4,
    };

    const SQUARE: Rock = Rock {
        positions: [0b0011, 0b0011, 0b0000, 0b0000],
        width: 2,
        height: 2,
    };

    pub fn rock_iterator() -> impl Iterator<Item = (Rock, usize)> {
        crate::RepeatingIterator {
            idx: 0,
            items: vec![FLAT, CROSS, L, VERTICAL, SQUARE],
        }
    }
}

struct Board<R> {
    // 0 is the bottom row, highest bit is ignored
    rows: VecDeque<u8>,
    rock_count: usize,
    falling_rock: rocks::Rock,
    falling_rock_row: usize, // 0 bottom
    falling_rock_col: usize, // right edge of rock
    rock_iterator: R,
    trimmed_rows: usize,
}
impl<R> Debug for Board<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in (0..(self.falling_rock_row + self.falling_rock.height)).rev() {
            writeln!(
                f,
                "|{}{}{}{}{}{}{}|",
                self.get_debug_char(row, 6),
                self.get_debug_char(row, 5),
                self.get_debug_char(row, 4),
                self.get_debug_char(row, 3),
                self.get_debug_char(row, 2),
                self.get_debug_char(row, 1),
                self.get_debug_char(row, 0),
            )?;
        }
        writeln!(f, "+-------+")
    }
}
impl<R: Iterator<Item = (rocks::Rock, usize)>> Board<R> {
    fn new(mut rock_iterator: R) -> Self {
        let (falling_rock, _) = rock_iterator.next().unwrap();
        let spawn_col = 5 - falling_rock.width;
        Self {
            rows: VecDeque::with_capacity(500),
            rock_count: 0,
            falling_rock,
            falling_rock_row: 3,
            falling_rock_col: spawn_col,
            rock_iterator,
            trimmed_rows: 0,
        }
    }

    // returns true if the rock landed
    fn move_rock(&mut self, gust: Gust) {
        let new_col = match gust {
            Gust::Left => (self.falling_rock_col + 1).min(7 - self.falling_rock.width),
            Gust::Right => self.falling_rock_col.checked_sub(1).unwrap_or(0),
        };
        if !self.check_collision(&self.falling_rock, self.falling_rock_row, new_col) {
            self.falling_rock_col = new_col;
        }
    }

    fn apply_gravity(&mut self) -> Option<usize> {
        if self.falling_rock_row == 0 {
            Some(self.finalize_falling_rock())
        } else {
            if self.check_collision(
                &self.falling_rock,
                self.falling_rock_row - 1,
                self.falling_rock_col,
            ) {
                Some(self.finalize_falling_rock())
            } else {
                self.falling_rock_row -= 1;
                None
            }
        }
    }

    fn finalize_falling_rock(&mut self) -> usize {
        self.rock_count += 1;

        if self.falling_rock_row + self.falling_rock.height >= self.rows.len() {
            self.rows
                .resize(self.falling_rock_row + self.falling_rock.height, 0);
        }

        for rock_row in 0..self.falling_rock.height {
            let rock_mask = self.falling_rock.get_mask(rock_row) << self.falling_rock_col;
            self.rows[self.falling_rock_row + rock_row] |= rock_mask;
        }

        if self.rows.len() > 1_000 {
            self.trim_rows();
        }

        let (new_rock, idx) = self.rock_iterator.next().unwrap();
        self.falling_rock = new_rock;
        self.falling_rock_row = self.rows.len() + 3;
        self.falling_rock_col = 5 - self.falling_rock.width;

        idx
    }
}
impl<R> Board<R> {
    fn height(&self) -> usize {
        self.rows.len() + self.trimmed_rows
    }

    fn get_top_rows(&self) -> [u8; 32] {
        let idx = self.rows.len();
        let mut result = [0xFF; 32];
        for i in 0..32 {
            let Some(idx) = idx.checked_sub(1) else {
                return result;
            };
            result[i] = self.rows[idx];
        }
        result
    }

    // true if the rock is colliding with any marked positions
    fn check_collision(&self, rock: &rocks::Rock, row: usize, col: usize) -> bool {
        for rock_row in 0..rock.height {
            if let Some(board_row_val) = self.rows.get(row + rock_row) {
                let mask = rock.get_mask(rock_row) << col;
                if board_row_val & mask != 0 {
                    return true;
                }
            }
        }

        false
    }

    fn trim_rows(&mut self) {
        let Some((idx, _)) = self.rows.iter().enumerate().rev().find(|(_, row)| **row == 0b0111_1111) else { return; };

        // We can drop everything up until idx
        self.trimmed_rows += idx;
        self.rows.drain(0..idx);
        // no need to update falling_rock_row, as it will be updated by finalize_falling_rock later
    }

    fn get_debug_char(&self, row: usize, col: usize) -> char {
        if let Some(rock_row) = row
            .checked_sub(self.falling_rock_row)
            .filter(|rock_row| *rock_row < self.falling_rock.height)
        {
            let rock_mask = self.falling_rock.get_mask(rock_row) << self.falling_rock_col;
            if rock_mask & (1 << col) != 0 {
                return '@';
            }
        }

        if let Some(val) = self.rows.get(row) {
            if *val & (1 << col) != 0 {
                return '#';
            }
        }

        '.'
    }
}

#[derive(Debug, Clone, Copy)]
enum Gust {
    Left,
    Right,
}
impl Gust {
    fn parse(byte: u8) -> Self {
        match byte {
            b'<' => Gust::Left,
            b'>' => Gust::Right,
            _ => panic!(),
        }
    }
}

const P1_ROCKS: usize = 2022;
const P2_ROCKS: usize = 1_000_000_000_000;

// We're looking for two states where:
// * the top 32 rows are identical (heuristic; not guaranteed but very likely)
// * the rock index is the same
// * the wind index is the same
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct CycleDetetionState {
    last_rows: [u8; 32],
    rock_idx: usize,
    gust_idx: usize,
}

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut gust_iterator = RepeatingIterator {
        idx: 0,
        items: input.trim().bytes().map(|b| Gust::parse(b)).collect(),
    };

    let rock_iterator = rocks::rock_iterator();
    let mut board = Board::new(rock_iterator);

    // println!("initial board state:\n{:?}", board);

    let mut cycle_detection = HashMap::new();

    let mut loops_simulated = 0;
    loop {
        let (gust, gust_idx) = gust_iterator.next().unwrap();
        board.move_rock(gust);

        if let Some(rock_idx) = board.apply_gravity() {
            let cycle_detection_state = CycleDetetionState {
                last_rows: board.get_top_rows(),
                rock_idx,
                gust_idx,
            };

            if let Some((rock_count_last_time, height_last_time)) = cycle_detection.insert(
                cycle_detection_state.clone(),
                (board.rock_count, board.height()),
            ) {
                println!("loop found! {loops_simulated}");
                cycle_detection.clear();

                if loops_simulated == 1 {
                    let rocks_in_loop = board.rock_count - rock_count_last_time;
                    let height_in_loop = board.height() - height_last_time;
                    let repetitions_to_simulate = (P2_ROCKS - board.rock_count) / rocks_in_loop;

                    println!("simulating repetitions: {repetitions_to_simulate}");

                    board.rock_count += dbg!(rocks_in_loop * repetitions_to_simulate);
                    board.trimmed_rows += dbg!(height_in_loop * repetitions_to_simulate);
                } else {
                    loops_simulated += 1;
                    cycle_detection
                        .insert(cycle_detection_state, (board.rock_count, board.height()));
                }
            }
        }

        if board.rock_count == P1_ROCKS {
            println!("part 1 solution: {}", board.height());
        }

        if board.rock_count == P2_ROCKS {
            println!("part 2 solution: {}", board.height());
            return;
        }
    }
}
