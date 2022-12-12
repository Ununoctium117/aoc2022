use std::{fs, io::{BufReader, BufRead}, collections::BTreeSet};

fn main() {
    let input = BufReader::new(fs::File::open("input.txt").unwrap());
    let input: Result<Vec<String>, _> = input.lines().collect();

    let mut elves = BTreeSet::new();
    let mut elf: Vec<u32> = Vec::new();
    for input in input.unwrap() {
        let input = input.trim();
        if input.is_empty() {
            let tmp = std::mem::replace(&mut elf, Vec::new());
            let sum: u32 = tmp.iter().sum();
            elves.insert((sum, tmp));
        } else {
            elf.push(input.parse().unwrap());
        }
    }

    let sum: u32 = elves.range(..).rev().take(3).map(|(x, _)| x).sum();
    println!("{}", sum);
}
