use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn parse_row(text: &String) -> Vec<Option<char>> {
    text.as_bytes()
        .chunks(4)
        .map(|chunk| {
            // chunk is "[x] " (last space optional)
            if chunk[1] == b' ' {
                None
            } else {
                Some(chunk[1] as char)
            }
        })
        .collect()
}

fn parse_stacks(lines: &[String]) -> Vec<Vec<char>> {
    let rows: Vec<_> = lines[..lines.len() - 1].iter().map(parse_row).collect();
    let mut stacks = vec![Vec::new(); rows[0].len()];

    for (stack_idx, stack) in stacks.iter_mut().enumerate() {
        for row in rows.iter().rev() {
            if let Some(char) = row[stack_idx] {
                stack.push(char);
            } else {
                break;
            }
        }
    }

    stacks
}

#[derive(Debug)]
struct Command {
    quantity: usize,
    from: usize,
    to: usize,
}
impl Command {
    fn apply_to_stacks(&self, stacks: &mut Vec<Vec<char>>) {
        for _ in 0..self.quantity {
            let char = stacks[self.from].pop().unwrap();
            stacks[self.to].push(char);
        }
    }

    fn apply_to_stacks2(&self, stacks: &mut Vec<Vec<char>>) {
        let from_len = stacks[self.from].len();
        let chars: Vec<char> = stacks[self.from].drain(from_len - self.quantity..).collect();
        stacks[self.to].extend(chars)
    }
}

fn parse_command(command: &String) -> Command {
    // "move X from Y to Z"
    let mut command = command.split_ascii_whitespace();
    Command {
        quantity: command.nth(1).unwrap().parse().unwrap(),
        from: command.nth(1).unwrap().parse::<usize>().unwrap() - 1,
        to: command.nth(1).unwrap().parse::<usize>().unwrap() - 1,
    }
}

fn main() {
    let lines: Result<Vec<_>, _> = BufReader::new(File::open("input.txt").unwrap())
        .lines()
        .collect();
    let lines = lines.unwrap();

    let mut input = lines.split(|str| str.is_empty());
    let mut stacks = parse_stacks(input.next().unwrap());
    let commands: Vec<Command> = input.next().unwrap().iter().map(parse_command).collect();

    for command in commands {
        command.apply_to_stacks2(&mut stacks);
    }

    let output: String = stacks.iter().map(|stack| *stack.last().unwrap()).collect();
    println!("{}", output);
}
