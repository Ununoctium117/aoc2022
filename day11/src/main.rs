use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
enum Operation {
    Square,
    Multiply(u64),
    Add(u64),
}
impl Operation {
    fn parse(text: &str) -> Self {
        if text == "* old" {
            Self::Square
        } else if let Some(value) = text.strip_prefix("* ") {
            Self::Multiply(value.parse().unwrap())
        } else if let Some(value) = text.strip_prefix("+ ") {
            Self::Add(value.parse().unwrap())
        } else {
            panic!("unknown operatation {}", text);
        }
    }

    fn apply(&self, inp: u64) -> u64 {
        match self {
            Operation::Square => inp * inp,
            Operation::Multiply(val) => inp * val,
            Operation::Add(val) => inp + val,
        }
    }
}

#[derive(Debug)]
struct Monkey {
    items: Vec<u64>,
    operation: Operation,
    test_divisible_by: u64,
    test_pass_to: usize,
    test_fail_to: usize,

    inspect_count: usize,
}
impl Monkey {
    pub fn parse(lines: &[String]) -> Self {
        let items = lines[1]
            .trim()
            .strip_prefix("Starting items: ")
            .unwrap()
            .split(", ")
            .map(|item| item.parse().unwrap())
            .collect();
        let operation = Operation::parse(
            lines[2]
                .trim()
                .strip_prefix("Operation: new = old ")
                .unwrap(),
        );
        let test_divisible_by = lines[3]
            .trim()
            .strip_prefix("Test: divisible by ")
            .unwrap()
            .parse()
            .unwrap();
        let test_pass_to = lines[4]
            .trim()
            .strip_prefix("If true: throw to monkey ")
            .unwrap()
            .parse()
            .unwrap();
        let test_fail_to = lines[5]
            .trim()
            .strip_prefix("If false: throw to monkey ")
            .unwrap()
            .parse()
            .unwrap();

        Self {
            items,
            operation,
            test_divisible_by,
            test_pass_to,
            test_fail_to,

            inspect_count: 0,
        }
    }
}

fn do_round(monkeys: &mut [Monkey], modulus: u64) {
    for id in 0..monkeys.len() {
        let mut targets = HashMap::<_, Vec<_>>::new();
        {
            let monkey = &mut monkeys[id];
            monkey.inspect_count += monkey.items.len();

            for item in monkey.items.drain(..) {
                // let new_worry_level = monkey.operation.apply(item) / 3;
                let new_worry_level = monkey.operation.apply(item) % modulus;

                if new_worry_level % monkey.test_divisible_by == 0 {
                    targets
                        .entry(monkey.test_pass_to)
                        .or_default()
                        .push(new_worry_level);
                } else {
                    targets
                        .entry(monkey.test_fail_to)
                        .or_default()
                        .push(new_worry_level);
                }
            }
        }

        for (id, items) in targets {
            monkeys[id].items.extend(items.into_iter());
        }
    }
}

fn print_items(monkeys: &[Monkey]) {
    for (id, monkey) in monkeys.iter().enumerate() {
        println!(
            "Monkey {} (inspect count: {}): {:?}",
            id, monkey.inspect_count, monkey.items
        );
    }
    println!();
}

fn main() {
    let lines = BufReader::new(File::open("input.txt").unwrap()).lines();

    let mut cur_monkey = Vec::with_capacity(6);
    let mut monkeys = Vec::new();
    for line in lines {
        let line = line.unwrap();
        if !line.is_empty() {
            cur_monkey.push(line);
        } else {
            let monkey = Monkey::parse(&cur_monkey[..]);
            monkeys.push(monkey);
            cur_monkey.clear();
        }
    }
    let monkey = Monkey::parse(&cur_monkey[..]);
    monkeys.push(monkey);

    let modulus: u64 = monkeys.iter().map(|monkey| monkey.test_divisible_by).product();

    print_items(&monkeys[..]);

    for _ in 0..10_000 {
        do_round(&mut monkeys[..], modulus);
        // print_items(&monkeys[..]);
    }

    print_items(&monkeys[..]);

    monkeys.sort_by_cached_key(|monkey| monkey.inspect_count);
    let level = monkeys[monkeys.len() - 1].inspect_count * monkeys[monkeys.len() - 2].inspect_count;
    println!("{}", level);
}
