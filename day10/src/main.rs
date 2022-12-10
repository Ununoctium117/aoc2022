use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

enum Instruction {
    Noop,
    Addx(i64),
}
impl Instruction {
    fn execute(&self, cycle: &mut usize, x: &mut i64) {
        match self {
            Instruction::Noop => {
                *cycle += 1;
            }
            Instruction::Addx(val) => {
                *x += val;
                *cycle += 2;
            }
        }
    }
}
impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "noop" {
            Ok(Self::Noop)
        } else if let Some(suffix) = s.strip_prefix("addx ") {
            Ok(Self::Addx(suffix.parse().unwrap()))
        } else {
            Err(())
        }
    }
}

fn main() {
    let lines = BufReader::new(File::open("input.txt").unwrap())
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut cycle = 0;
    let mut x = 1;

    let mut crt = [['?'; 40]; 6];

    for line in lines {
        let instruction: Instruction = line.parse().unwrap();
        let prev_cycle = cycle;
        let prev_x = x;
        instruction.execute(&mut cycle, &mut x);

        for cycle in prev_cycle..cycle {
            let row = cycle / 40;
            let column = cycle % 40;

            if ((column as i64) - prev_x).abs() <= 1 {
                crt[row][column] = '#';
            } else {
                crt[row][column] = '.';
            }
        }
    }

    for line in crt {
        println!("{}", line.iter().collect::<String>());
    }
}
