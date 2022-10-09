// -*- compile-command: "cargo test -- --show-output" -*-

use std::fs;

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> usize {
    0
}

pub fn part2() -> usize {
    0
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn run_part1() {
        assert_eq!(usize::MAX, part1());
    }

    #[test]
    fn run_part2() {
        assert_eq!(usize::MAX, part2());
    }
}
