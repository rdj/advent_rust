// -*- compile-command: "cargo test -- --show-output" -*-

use std::fs;

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() {
}

pub fn part2() {
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn run_part1() {
        part1();
    }

    #[test]
    fn run_part2() {
        part2();
    }
}
