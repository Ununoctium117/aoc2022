use std::{
    fs::File,
    io::{BufRead, BufReader},
};

// 29-82,89-90
// 89-90,29-82

fn overlaps((elf1_min, elf1_max): (u32, u32), (elf2_min, elf2_max): (u32, u32)) -> bool {
    (elf1_max >= elf2_min && elf1_min <= elf2_min) || (elf1_min <= elf2_min && elf1_max >= elf2_max)
}

fn main() {
    let result = BufReader::new(File::open("input.txt").unwrap()).lines().filter(|line| {
        let line = line.as_ref().unwrap();
        let mut line = line.split(",").map(|segment| {
            let mut range = segment.split("-").map(|num| num.parse::<u32>().unwrap());
            (range.next().unwrap(), range.next().unwrap())
        });

        let elf1 = line.next().unwrap();
        let elf2 = line.next().unwrap();

        overlaps(elf1, elf2) || overlaps(elf2, elf1)
    }).count();
    println!("{}", result);
}
