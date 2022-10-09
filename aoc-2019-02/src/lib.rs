// -*- compile-command: "cargo test -- --show-output" -*-

#![allow(dead_code)]

use std::fs;

const ADD: usize = 1; // apos bpos outpos
const MUL: usize = 2;
const HALT: usize = 99;
const STEP: usize = 4;

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn initial_state() -> Vec<usize> {
    input().trim().split(",").map(|s| s.parse().unwrap()).collect()
}

fn result(intcodes: &Vec<usize>) -> usize {
    *intcodes.get(0).unwrap()
}

pub fn part1() {
    let mut intcodes: Vec<usize> = initial_state();

    restore_state(&mut intcodes, 12, 2);

    let intcodes = compute(intcodes);

    println!("result = {}", result(&intcodes));
}

pub fn part2() {
    let target_output = 19690720;
    let initial = initial_state();

    for noun in 0..=99 {
        for verb in 0..=99 {
            let mut intcodes = initial.clone();
            restore_state(&mut intcodes, noun, verb);
            let intcodes = compute(intcodes);
            if result(&intcodes) == target_output {
                println!("noun = {noun}, verb = {verb}");
                println!("part 2 = {}", 100 * noun + verb);
                break;
            }
        }
    }
}

fn restore_state(intcodes: &mut Vec<usize>, noun: usize, verb: usize) {
    *intcodes.get_mut(1).unwrap() = noun;
    *intcodes.get_mut(2).unwrap() = verb;
}

fn compute(mut intcodes: Vec<usize>) -> Vec<usize> {
    let mut i = 0;
    loop {
        let op = match intcodes.get(i).unwrap() {
            &ADD => |a, b| a + b,
            &MUL => |a, b| a * b,
            &HALT => break,
            x => panic!("Unknown opcode {x}"),
        };

        let apos = *intcodes.get(i + 1).unwrap();
        let bpos = *intcodes.get(i + 2).unwrap();
        let outpos = *intcodes.get(i + 3).unwrap();

        let a = intcodes.get(apos).unwrap();
        let b = intcodes.get(bpos).unwrap();
        let result = op(a, b);

        let out = intcodes.get_mut(outpos).unwrap();
        *out = result;

        i += STEP;
    }
    intcodes
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute() {
        assert_eq!(compute(vec![1, 0, 0, 0, 99]), vec![2, 0, 0, 0, 99]);
        assert_eq!(compute(vec![2, 3, 0, 3, 99]), vec![2, 3, 0, 6, 99]);
        assert_eq!(compute(vec![2, 4, 4, 5, 99, 0]), vec![2, 4, 4, 5, 99, 9801]);
        assert_eq!(
            compute(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]),
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99]
        );
    }

    #[test]
    fn run_part1() {
        part1();
    }

    #[test]
    fn run_part2() {
        part2();
    }
}
