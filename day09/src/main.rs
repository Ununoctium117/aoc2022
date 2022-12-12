use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

fn update_tail_pos((tx, ty): &mut (i32, i32), (hx, hy): &(i32, i32)) {
    let dx = *hx - *tx;
    let dy = *hy - *ty;
    if dx.abs() <= 1 && dy.abs() <= 1 {
        return;
    }

    // vertically in line
    if dx == 0 {
        *ty = hy - dy.signum();
        return;
    }

    // horizontally in line
    if dy == 0 {
        *tx = hx - dx.signum();
        return;
    }

    // further away vertically than horizontally; equalize x and then make y update
    if dy.abs() > dx.abs() {
        *tx = *hx;
        *ty = hy - dy.signum();
        return;
    }

    // further away horizontally than vertically; equalize y and then make x update
    if dx.abs() > dy.abs() {
        *ty = *hy;
        *tx = hx - dx.signum();
        return;
    }

    // far away in both axes; correct both coordinates
    *tx = hx - dx.signum();
    *ty = hy - dy.signum();
}

fn dbg_points(points: &[(i32, i32)]) {
    let mut grid = [['.'; 30]; 30];
    for (i, (x, y)) in points.iter().enumerate() {
        grid[(15 - y) as usize][(x + 15) as usize] = std::char::from_digit(i as u32, 10).unwrap();
    }
    for row in grid {
        println!("{}", row.iter().collect::<String>());
    }
    println!("----")
}

fn main() {
    let lines = BufReader::new(File::open("input.txt").unwrap())
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut rope = [(0, 0); 10];
    let mut tail_history: HashSet<(i32, i32)> = HashSet::new();
    tail_history.insert((0, 0));

    for line in lines {
        let mut words = line.split_ascii_whitespace();
        let dir = words.next().unwrap();
        let amount: i32 = words.next().unwrap().parse().unwrap();

        let (dx, dy) = match dir {
            "D" => (0, -1),
            "U" => (0, 1),
            "L" => (-1, 0),
            "R" => (1, 0),
            _ => panic!("unknown direction"),
        };

        for _ in 0..amount {
            rope[0].0 += dx;
            rope[0].1 += dy;

            for tail_idx in 1..rope.len() {
                let head_pos = rope[tail_idx - 1];
                update_tail_pos(&mut rope[tail_idx], &head_pos);
            }

            tail_history.insert(rope[rope.len() - 1].clone());
        }
        // dbg_points(&rope[..]);
    }

    println!("{}", tail_history.len());
}
