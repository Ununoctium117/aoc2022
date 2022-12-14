use std::{collections::HashMap, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Air,
    Rock,
    Sand,
}

#[derive(Debug)]
struct Map {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
    tiles: HashMap<(i32, i32), Tile>,
}
impl Default for Map {
    fn default() -> Self {
        Self {
            min_x: i32::MAX,
            max_x: i32::MIN,
            min_y: i32::MAX,
            max_y: i32::MIN,
            tiles: Default::default(),
        }
    }
}
impl Map {
    fn drop_sand_from(&mut self, mut x: i32, mut y: i32) -> (i32, i32) {
        'falling: loop {
            let new_y = y + 1;

            for new_x in [x, x - 1, x + 1] {
                if self.get_tile_at(new_x, new_y) == Tile::Air {
                    x = new_x;
                    y = new_y;
                    continue 'falling;
                }
            }

            // none of the new spaces were empty
            self.set_tile_at(x, y, Tile::Sand, false);
            return (x, y);
        }
    }

    fn set_tile_at(&mut self, x: i32, y: i32, tile: Tile, update_bounds: bool) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);

        if update_bounds {
            self.min_y = self.min_y.min(y);
            self.max_y = self.max_y.max(y);
        }

        self.tiles.insert((x, y), tile);
    }

    fn get_tile_at(&self, x: i32, y: i32) -> Tile {
        if y == (self.max_y + 2) {
            Tile::Rock
        } else {
            self.tiles.get(&(x, y)).copied().unwrap_or(Tile::Air)
        }
    }

    fn print(&self) {
        for y in self.min_y..=(self.max_y + 2) {
            let mut text = String::new();
            for x in self.min_x..=self.max_x {
                text.push(match self.get_tile_at(x, y) {
                    Tile::Air => ' ',
                    Tile::Rock => '#',
                    Tile::Sand => '.',
                });
            }
            println!("{}", text);
        }
    }
}
impl FromStr for Map {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Map::default();

        for line in s.lines() {
            let points = line
                .split(" -> ")
                .map(|s| {
                    let mut parts = s.split(",");
                    let x: i32 = parts.next().unwrap().parse().unwrap();
                    let y: i32 = parts.next().unwrap().parse().unwrap();
                    (x, y)
                })
                .collect::<Vec<_>>();

            for pair in points.windows(2) {
                let (ax, ay) = pair[0];
                let (bx, by) = pair[1];
                if ax == bx {
                    let start = ay.min(by);
                    let stop = ay.max(by);
                    for y in start..=stop {
                        result.set_tile_at(ax, y, Tile::Rock, true);
                    }
                } else if ay == by {
                    let start = ax.min(bx);
                    let stop = ax.max(bx);
                    for x in start..=stop {
                        result.set_tile_at(x, ay, Tile::Rock, true);
                    }
                } else {
                    panic!("diagonal path");
                }
            }
        }

        Ok(result)
    }
}

fn main() {
    let mut map: Map = std::fs::read_to_string("input.txt")
        .unwrap()
        .parse()
        .unwrap();
    map.print();

    let mut sand_count = 0;
    while map.drop_sand_from(500, 0) != (500, 0) {
        // map.print();
        sand_count += 1;
    }

    map.print();
    println!("{}", sand_count + 1);
}
