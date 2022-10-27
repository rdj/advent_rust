#![allow(dead_code, unused_variables)]

type AdventResult = usize;

use std::fs;

mod computer;
use computer::{Computer, Intcode};

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    let mut count = 0;

    let prog = Computer::parse_program(input);
    
    for y in 0..50 {
        for x in 0..50 {
            if check(&prog, x, y) {
                count += 1;
            }
        }
    }

    count
}

fn check(prog: &Vec<Intcode>, x: usize, y: usize) -> bool {
    let mut computer = Computer::new(prog.clone());
    computer.buffer_input(x as Intcode);
    computer.buffer_input(y as Intcode);
    computer.start();
    computer.consume_output().unwrap() != 0
}

fn check_rect_top_left(prog: &Vec<Intcode>, x: usize, y: usize, dim: usize) -> bool {
    let dim = dim - 1;
    // x,y top left
    check(&prog, x, y) &&
        check(&prog, x + dim, y) &&
        check(&prog, x, y + dim) &&
        check(&prog, x + dim, y + dim)
}

fn check_rect_top_right(prog: &Vec<Intcode>, x: usize, y: usize, dim: usize) -> bool {
    let dim = dim - 1;
    // x,y upper right
    check(&prog, x, y) &&
        check(&prog, x - dim, y) &&
        check(&prog, x, y + dim) &&
        check(&prog, x - dim, y + dim)
}

fn do_part2(input: &str) -> AdventResult {
    let prog = Computer::parse_program(input);

    // Find the right edge at y=100
    let mut y = 100;
    let mut x = 0;
    while !check(&prog, x, y) {
        x += 1;
    }
    while check(&prog, x, y) {
        x += 1;
    }
    x -= 1;

    // Hug the right edge while advancing each line, checking the four
    // corners based on the top-right being on the edge
    loop {
        if check_rect_top_right(&prog, x, y, 100) {
            break;
        }

        y += 1;
        while check(&prog, x, y) {
            x += 1;
        }
        x -= 1;
    }
    println!("Top Right corner at {} {} works", x, y);

    // Calculate the top-left
    x -= 99;
    assert!(check_rect_top_left(&prog, x, y, 100));

    println!("Top Left corner at {} {} works", x, y);
    x * 10_000 + y
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
    fn part1_example() {
        // no example
    }

    #[test]
    fn part2_example() {
        // no example
    }
    
    #[test]
    fn part1_solution() {
        assert_eq!(147, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(13_280_865, part2());
    }
}
