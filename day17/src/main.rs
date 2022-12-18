use std::{collections::VecDeque, fmt::Debug, time::Instant};

struct RepeatingIterator<T> {
    idx: usize,
    items: Vec<T>,
}
impl<T: Clone> Iterator for RepeatingIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.items[self.idx].clone();
        self.idx = (self.idx + 1) % self.items.len();
        Some(result)
    }
}

mod rocks {
    use crate::RepeatingIterator;

    // 4x4 rock
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

    pub fn rock_iterator() -> impl Iterator<Item = Rock> {
        RepeatingIterator {
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
impl<R: Iterator<Item = rocks::Rock>> Board<R> {
    fn new(mut rock_iterator: R) -> Self {
        let falling_rock = rock_iterator.next().unwrap();
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

    fn apply_gravity(&mut self) -> bool {
        if self.falling_rock_row == 0 {
            self.finalize_falling_rock();
            true
        } else {
            if self.check_collision(
                &self.falling_rock,
                self.falling_rock_row - 1,
                self.falling_rock_col,
            ) {
                self.finalize_falling_rock();
                true
            } else {
                self.falling_rock_row -= 1;
                false
            }
        }
    }

    fn finalize_falling_rock(&mut self) {
        self.rock_count += 1;

        if self.falling_rock_row + self.falling_rock.height >= self.rows.len() {
            self.rows
                .resize(self.falling_rock_row + self.falling_rock.height, 0);
        }

        for rock_row in 0..self.falling_rock.height {
            let rock_mask = self.falling_rock.get_mask(rock_row) << self.falling_rock_col;
            self.rows[self.falling_rock_row + rock_row] |= rock_mask;
        }

        if self.rows.len() > 100_000 {
            self.trim_rows();
        }

        self.falling_rock = self.rock_iterator.next().unwrap();
        self.falling_rock_row = self.rows.len() + 3;
        self.falling_rock_col = 5 - self.falling_rock.width;
    }
}
impl<R> Board<R> {
    // true if the rock is colliding with any marked positions
    fn check_collision(&self, rock: &rocks::Rock, row: usize, col: usize) -> bool {
        for rock_row in 0..rock.height {
            if let Some(board_row_val) = self.rows.get(row + rock_row) {
                let mask = rock.get_mask(rock_row) << col;
                // println!(
                //     "{} {} {:08b} {:08b} {:08b}",
                //     row,
                //     col,
                //     board_row_val,
                //     rock.get_mask(rock_row),
                //     mask
                // );
                if board_row_val & mask != 0 {
                    return true;
                }
            }
        }

        false
    }

    fn trim_rows(&mut self) {
        let Some((idx, _)) = self.rows.iter().enumerate().rev().find(|(_, row)| **row == 0b0111_1111) else { return; };

        // println!("trimming off {} rows", idx);

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

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut gust_iterator = RepeatingIterator {
        idx: 0,
        items: input.trim().bytes().map(|b| Gust::parse(b)).collect(),
    };

    let rock_iterator = rocks::rock_iterator();
    let mut board = Board::new(rock_iterator);

    // println!("initial board state:\n{:?}", board);

    let start = Instant::now();
    loop {
        let gust = gust_iterator.next().unwrap();
        board.move_rock(gust);
        // println!("rock pushed {:?}:\n{:?}", gust, board);

        board.apply_gravity();
        // println!("rock falls:\n{:?}", board);

        if board.rock_count % 100_000_000 == 0 {
            println!("simulated {} rocks in {:?}", board.rock_count, Instant::now() - start);
        }

        if board.rock_count == 2022 {
            println!("{}", board.rows.len() + board.trimmed_rows);
        }

        if board.rock_count == 1000000000000 {
            println!("{}", board.rows.len() + board.trimmed_rows);
            return;
        }
    }
}
