use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

fn priority(item: &char) -> u32 {
    match item {
        'a'..='z' => *item as u32 - b'a' as u32 + 1,
        'A'..='Z' => *item as u32 - b'A' as u32 + 27,
        _ => panic!("fail"),
    }
}

fn main() {
    let lines: Result<Vec<_>, _> = BufReader::new(File::open("input.txt").unwrap())
        .lines()
        .collect();
    let lines = lines.unwrap();

    let mut priority_sum = 0;
    for group in lines.chunks(3) {
        priority_sum += group
            .iter()
            .map(|line| line.chars().collect::<HashSet<_>>())
            .reduce(|accum, item| accum.intersection(&item).cloned().collect())
            .unwrap()
            .iter()
            .map(priority)
            .sum::<u32>();
    }
    println!("{}", priority_sum);
}
