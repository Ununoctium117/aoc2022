use std::{
    fs::File,
    io::{BufRead, BufReader},
};

struct TreeGrid {
    trees: Vec<u8>,
    width: usize,
}
impl TreeGrid {
    fn parse(lines: &[String]) -> TreeGrid {
        let width = lines[0].len();
        let mut trees = Vec::new();
        for line in lines {
            trees.extend(line.as_bytes().iter());
        }

        TreeGrid { trees, width }
    }

    fn val_at(&self, x: usize, y: usize) -> u8 {
        self.trees[y * self.width + x]
    }

    fn height(&self) -> usize {
        self.trees.len() / self.width
    }

    fn is_visible(&self, x: usize, y: usize) -> bool {
        let height = self.val_at(x, y);

        let visible_from_top = (0..y).all(|test_y| self.val_at(x, test_y) < height);
        let visible_from_left = (0..x).all(|test_x| self.val_at(test_x, y) < height);
        let visible_from_bottom =
            ((y + 1)..self.height()).all(|test_y| self.val_at(x, test_y) < height);
        let visible_from_right =
            ((x + 1)..self.width).all(|test_x| self.val_at(test_x, y) < height);

        visible_from_top || visible_from_bottom || visible_from_left || visible_from_right
    }

    fn scenic_score(&self, x: usize, y: usize) -> usize {
        let height = self.val_at(x, y);

        let mut visible_up = 0;
        for test_y in (0..y).rev() {
            visible_up += 1;
            if self.val_at(x, test_y) >= height {
                break;
            }
        }


        let mut visible_down = 0;
        for test_y in (y + 1)..self.height() {
            visible_down += 1;
            if self.val_at(x, test_y) >= height {
                break;
            }
        }

        let mut visible_left = 0;
        for test_x in (0..x).rev() {
            visible_left += 1;
            if self.val_at(test_x, y) >= height {
                break;
            }
        }

        let mut visible_right = 0;
        for test_x in (x + 1)..self.width {
            visible_right += 1;
            if self.val_at(test_x, y) >= height {
                break;
            }
        }

        visible_up * visible_down * visible_left * visible_right
    }
}

fn main() {
    let lines: Result<Vec<_>, _> = BufReader::new(File::open("input.txt").unwrap())
        .lines()
        .collect();
    let lines = lines.unwrap();

    let trees = TreeGrid::parse(&lines[..]);

    let mut visible_trees = 0;
    for (x, y) in (0..trees.width).flat_map(|x| (0..trees.height()).map(move |y| (x, y))) {
        if trees.is_visible(x, y) {
            visible_trees += 1;
        }
    }

    println!("{}", visible_trees);

    let mut max_scenic_score = 0;
    for (x, y) in (0..trees.width).flat_map(|x| (0..trees.height()).map(move |y| (x, y))) {
        max_scenic_score = max_scenic_score.max(trees.scenic_score(x, y));
    }

    println!("{}", max_scenic_score);
}
