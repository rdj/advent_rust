#![allow(dead_code, unused_variables)]

mod computer;
use computer::Computer;

type AdventResult = usize;

use std::fs;

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    let initial = Computer::parse_program(input);
    let mut computer = Computer::new(initial);
    computer.buffer_inputs(Computer::ascii_to_intcodes("\
NOT A T
NOT B J
OR T J
NOT C T
OR T J
AND D J
WALK
").into_iter());
    computer.start();
    assert!(computer.is_halted());
    let output: Vec<_> = computer.consume_output_buffer().collect();

    if let Some(&n) = output.last() {
        if n > 255 {
            return n as usize;
        } 
    }
        
    println!("{}", Computer::intcodes_to_ascii(output.into_iter()));
    panic!();
}

fn do_part2(input: &str) -> AdventResult {
    let initial = Computer::parse_program(input);
    let mut computer = Computer::new(initial);
    computer.buffer_inputs(Computer::ascii_to_intcodes("\
NOT A T
NOT B J
OR T J
NOT C T
OR T J
AND D J
NOT J T
NOT T T
AND E T
OR H T
AND T J
RUN
").into_iter());
    computer.start();
    assert!(computer.is_halted());
    let output: Vec<_> = computer.consume_output_buffer().collect();

    if let Some(&n) = output.last() {
        if n > 255 {
            return n as usize;
        } 
    }
        
    println!("{}", Computer::intcodes_to_ascii(output.into_iter()));
    panic!();
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
    fn part1_solution() {
        assert_eq!(19355862, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(1140470745, part2());
    }
}
