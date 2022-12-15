#![feature(btree_drain_filter)]

use std::{
    collections::{BTreeSet, HashSet},
    str::FromStr,
};

#[derive(Debug, Clone)]
struct Sensor {
    x: i64,
    y: i64,
    beacon_x: i64,
    beacon_y: i64,
}
impl Sensor {
    fn distance_to_beacon(&self) -> i64 {
        (self.x - self.beacon_x).abs() + (self.y - self.beacon_y).abs()
    }

    fn visible_tiles_in_row(&self, y: i64) -> Option<(i64, i64)> {
        let distance = self.distance_to_beacon();
        let dy = (self.y - y).abs();

        let remaining_distance_for_x = distance - dy;
        if remaining_distance_for_x < 0 {
            None
        } else {
            let min = self.x - remaining_distance_for_x;
            let max = self.x + remaining_distance_for_x;
            Some((min, max))
        }
    }
}
impl FromStr for Sensor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_ascii_whitespace();
        let x = words
            .nth(2)
            .unwrap()
            .strip_prefix("x=")
            .unwrap()
            .strip_suffix(",")
            .unwrap()
            .parse()
            .unwrap();
        let y = words
            .nth(0)
            .unwrap()
            .strip_prefix("y=")
            .unwrap()
            .strip_suffix(":")
            .unwrap()
            .parse()
            .unwrap();
        let beacon_x = words
            .nth(4)
            .unwrap()
            .strip_prefix("x=")
            .unwrap()
            .strip_suffix(",")
            .unwrap()
            .parse()
            .unwrap();
        let beacon_y = words
            .nth(0)
            .unwrap()
            .strip_prefix("y=")
            .unwrap()
            .parse()
            .unwrap();

        Ok(Sensor {
            x,
            y,
            beacon_x,
            beacon_y,
        })
    }
}

#[derive(Debug, Default, Clone)]
struct RangeGroup {
    ranges: BTreeSet<(i64, i64)>,
}
impl RangeGroup {
    fn add_range(&mut self, min: i64, max: i64) {
        self.ranges.insert((min, max));
        // while self.simplify() {}
    }

    fn contains(&self, test: i64) -> bool {
        for (start, end) in &self.ranges {
            if test >= *start && test <= *end {
                return true;
            }
        }
        false
    }

    fn find_first_uncovered(&self, min: i64, max: i64) -> Option<i64> {
        dbg!(self);
        for test in min..=max {
            if !self.contains(test) {
                return Some(test);
            }
        }
        None
    }

    fn range_count(&self, min: i64, max: i64) -> usize {
        let mut count = 0;

        for (rmin, rmax) in &self.ranges {
            if *rmax >= min && *rmin <= max {
                count += 1;
            } else if *rmin <= min && *rmax >= max {
                count += 1;
            }
        }

        count
    }

    fn len(&self) -> usize {
        let mut len = 0;
        for (start, end) in &self.ranges {
            len += (end - start) + 1; // inclusive
        }
        len as usize
    }

    fn simplify(&mut self) -> bool {
        let mut changed = false;
        let mut new_ranges = BTreeSet::new();
        // For each first range, find everything that starts inside it. Remove those entries, then extend
        // the first range to the maximum of everything that started inside.
        while let Some(first_range) = self.ranges.pop_first() {
            let mut new_max = first_range.1;
            for entry in self.ranges.drain_filter(|(min, _)| first_range.1 >= *min) {
                new_max = new_max.max(entry.1);
                changed = true;
            }

            new_ranges.insert((first_range.0, new_max));
        }

        self.ranges = new_ranges;
        changed
    }
}

struct Map {
    sensors: Vec<Sensor>,
}
impl Map {
    fn new(sensors: Vec<Sensor>) -> Self {
        Self { sensors }
    }

    fn count_beaconless_tiles_in_row(&self, y: i64) -> usize {
        let mut seen_xs = RangeGroup::default();

        for sensor in &self.sensors {
            if let Some((min, max)) = sensor.visible_tiles_in_row(y) {
                seen_xs.add_range(min, max);
            }
        }
        while seen_xs.simplify() {}

        let mut beacon_xs = HashSet::new();
        for sensor in &self.sensors {
            if sensor.beacon_y == y && seen_xs.contains(sensor.beacon_x) {
                beacon_xs.insert(sensor.beacon_x);
            }
        }

        seen_xs.len() - beacon_xs.len()
    }

    fn get_hidden_beacon_x(&self, test_y: i64, min_x: i64, max_x: i64) -> Option<i64> {
        let mut seen_xs = RangeGroup::default();

        for sensor in &self.sensors {
            if let Some((min, max)) = sensor.visible_tiles_in_row(test_y) {
                seen_xs.add_range(min, max);
            }
        }
        while seen_xs.simplify() {}

        if seen_xs.range_count(min_x, max_x) > 1 {
            seen_xs.find_first_uncovered(min_x, max_x)
        } else {
            None
        }
    }
}

fn main() {
    let max_coord = 4_000_000;
    let input = std::fs::read_to_string("input.txt").unwrap();

    let sensors = input
        .lines()
        .map(|line| line.parse().unwrap())
        .collect::<Vec<_>>();
    let map = Map::new(sensors);

    println!("{}", map.count_beaconless_tiles_in_row(2_000_000));

    let mut beacon_x = 0;
    let mut beacon_y = 0;
    for y in 0..max_coord {
        if let Some(x) = map.get_hidden_beacon_x(y, 0, max_coord) {
            beacon_x = x;
            beacon_y = y;
            println!("found tile: {}, {}", x, y);
            break;
        }
    }

    println!("{}", beacon_x * max_coord + beacon_y);
}
