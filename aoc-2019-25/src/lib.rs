#![allow(dead_code, unused_variables)]

type AdventResult = usize;

use std::fs;

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    todo!()
}

fn do_part2(input: &str) -> AdventResult {
    todo!()
}

fn part1() -> AdventResult {
    do_part1(&input())
}

fn part2() -> AdventResult {
    do_part2(&input())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part2_example() {
        todo!()
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
