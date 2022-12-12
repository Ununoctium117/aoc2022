use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Lose,
    Draw,
    Win,
}
impl Outcome {
    fn score(&self) -> u32 {
        use Outcome::*;
        match self {
            Lose => 0,
            Draw => 3,
            Win => 6,
        }
    }
}
impl FromStr for Outcome {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Outcome::*;

        Ok(match s {
            "X" => Lose,
            "Y" => Draw,
            "Z" => Win,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rps {
    Rock,
    Paper,
    Scissors,
}
impl Rps {
    fn outcome(&self, opp: &Self) -> Outcome {
        use Rps::*;

        if self == opp {
            Outcome::Draw
        } else {
            match (self, opp) {
                (Rock, Scissors) => Outcome::Win,
                (Scissors, Paper) => Outcome::Win,
                (Paper, Rock) => Outcome::Win,

                _ => Outcome::Lose,
            }
        }
    }

    fn what_to_play_for_outcome(&self, outcome: Outcome) -> Self {
        use Rps::*;

        match (self, outcome) {
            (Rock, Outcome::Lose) => Scissors,
            (Rock, Outcome::Win) => Paper,
            (Paper, Outcome::Lose) => Rock,
            (Paper, Outcome::Win) => Scissors,
            (Scissors, Outcome::Lose) => Paper,
            (Scissors, Outcome::Win) => Rock,
            (x, Outcome::Draw) => *x,
        }
    }

    fn score(&self) -> u32 {
        use Rps::*;
        match self {
            Rock => 1,
            Paper => 2,
            Scissors => 3,
        }
    }
}
impl FromStr for Rps {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Rps::*;
        Ok(match s {
            "A" | "X" => Rock,
            "B" | "Y" => Paper,
            "C" | "Z" => Scissors,
            _ => return Err(()),
        })
    }
}

fn main() {
    let input = BufReader::new(File::open("input.txt").unwrap()).lines();

    let result: u32 = input
        .map(|line| {
            let line = line.unwrap();
            let line: Vec<&str> = line.split_ascii_whitespace().collect();

            let opp: Rps = line[0].parse().unwrap();
            let outcome: Outcome = line[1].parse().unwrap();

            outcome.score() + opp.what_to_play_for_outcome(outcome).score()
        })
        .sum();

    println!("{}", result);
}
