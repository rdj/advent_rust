// -*- compile-command: "cargo test -- --show-output" -*-

type AdventResult = usize;

use std::fs;

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> AdventResult {
    0
}

pub fn part2() -> AdventResult {
    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_example() {
        panic!("part 1 example");
    }

    #[test]
    fn part2_example() {
        panic!("part 2 example");
    }
    
    #[test]
    fn part1_solution() {
        assert_eq!(AdventResult::MAX, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
