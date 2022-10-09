// -*- compile-command: "cargo test -- --show-output" -*-

use std::fs;
use std::ops::RangeInclusive;

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn input_range() -> RangeInclusive<usize> {
    let bounds: Vec<usize> = input()
        .trim()
        .split('-')
        .map(|n| n.parse().unwrap())
        .collect();
    assert_eq!(2, bounds.len());
    let lower = *bounds.first().unwrap();
    let upper = *bounds.get(1).unwrap();
    lower..=upper
}

pub fn part1() -> usize {
    let range = input_range();
    let valid: Vec<_> = range
        .into_iter()
        .filter(|n| is_valid(&n.to_string()))
        .collect();
    valid.len()
}

pub fn part2() -> usize {
    let range = input_range();
    let valid: Vec<_> = range
        .into_iter()
        .filter(|n| is_valid2(&n.to_string()))
        .collect();
    valid.len()
}

pub fn is_valid(s: &str) -> bool {
    let mut has_dupe = false;

    let mut chars = s.chars();
    let mut last = chars.next().unwrap();

    for c in chars {
        if c < last {
            return false;
        }
        if c == last {
            has_dupe = true;
        }
        last = c;
    }

    has_dupe
}

pub fn is_valid2(s: &str) -> bool {
    let mut has_run = false;
    let mut run_length = 1;

    let mut chars = s.chars();
    let mut last = chars.next().unwrap();

    for c in chars {
        if c < last {
            return false;
        }
        if c == last {
            run_length += 1;
        } else {
            if run_length == 2 {
                has_run = true
            }
            run_length = 1;
        }
        last = c;
    }
    if run_length == 2 {
        has_run = true
    }

    has_run
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_valid() {
        assert!(is_valid("111111"));
        assert!(!is_valid("223450"));
        assert!(!is_valid("123789"));
    }

    #[test]
    fn test_is_valid2() {
        assert!(is_valid2("112233"));
        assert!(!is_valid2("123444"));
        assert!(is_valid2("111122"));
    }

    #[test]
    fn run_part1() {
        assert_eq!(1063, part1());
    }

    #[test]
    fn run_part2() {
        assert_eq!(686, part2());
    }
}
