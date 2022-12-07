use std::{fs, collections::HashSet};

// returns number of characters needed to be read until first marker found
fn find_substring_of_unique_chars(input: &String, len: usize) -> usize {
    for (window_idx, window) in input.as_bytes().windows(len).enumerate() {
        if window.iter().collect::<HashSet<_>>().len() == len {
            return window_idx + len;
        }
    }
    panic!("no marker");
}

fn main() {
    let input = fs::read_to_string("input.txt").unwrap();
    let start_idx = find_substring_of_unique_chars(&input, 14);
    println!("{}", start_idx);
}
