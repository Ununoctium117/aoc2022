use std::cmp::Ordering;

fn split_at_next_comma(s: &[u8]) -> (&[u8], &[u8]) {
    let mut depth = 0;
    let mut idx = 0;
    loop {
        if idx >= s.len() {
            return (s, &[]);
        }

        if s[idx] == b'[' {
            depth += 1;
        } else if s[idx] == b']' {
            depth -= 1;
        }

        if s[idx] == b',' && depth == 0 {
            return (&s[0..idx], &s[idx + 1..]);
        }

        idx += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Packet {
    Number(u32),
    List(Vec<Packet>),
}
impl Packet {
    fn parse(s: &[u8]) -> Option<Self> {
        if s.is_empty() {
            None
        } else if s[0] == b'[' {
            let mut result = Vec::new();

            let mut substr = &s[1..s.len() - 1];
            while !substr.is_empty() {
                let (child_element, remainder) = split_at_next_comma(substr);
                result.push(Packet::parse(child_element).unwrap());
                substr = remainder;
            }

            Some(Packet::List(result))
        } else {
            Some(Packet::Number(
                std::str::from_utf8(s).unwrap().parse().unwrap(),
            ))
        }
    }
}
impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Packet::*;

        let mut idx = 0;
        loop {
            let comparison_result = match (self, other) {
                (Number(left), Number(right)) => {
                    return left.partial_cmp(right);
                }
                (Number(left), List(_)) => {
                    return (Packet::List(vec![Packet::Number(*left)])).partial_cmp(other);
                }
                (List(_), Number(right)) => {
                    return self.partial_cmp(&Packet::List(vec![Packet::Number(*right)]));
                }
                (List(left), List(right)) => {
                    if idx >= left.len() && idx < right.len() {
                        return Some(Ordering::Less);
                    } else if idx < left.len() && idx >= right.len() {
                        return Some(Ordering::Greater);
                    } else if idx >= left.len() || idx >= right.len() {
                        return Some(Ordering::Equal);
                    }

                    left[idx].partial_cmp(&right[idx])
                }
            };

            match comparison_result {
                Some(Ordering::Equal) => idx += 1,
                _ => return comparison_result,
            }
        }
    }
}
impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let mut packets = input
        .lines()
        .filter_map(|line| Packet::parse(line.as_bytes()))
        .collect::<Vec<_>>();

    let divider_0 = Packet::List(vec![Packet::List(vec![Packet::Number(2)])]);
    let divider_1 = Packet::List(vec![Packet::List(vec![Packet::Number(6)])]);

    packets.push(divider_0.clone());
    packets.push(divider_1.clone());

    packets.sort_unstable();

    let divider_0_idx = packets.binary_search(&divider_0).unwrap();
    let divider_1_idx = packets.binary_search(&divider_1).unwrap();

    println!("{}", (divider_0_idx + 1) * (divider_1_idx + 1));
}
