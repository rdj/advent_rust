// -*- compile-command: "cargo test -- --show-output" -*-

mod computer;

use computer::Computer;

type AdventResult = i64;

use std::fs;

fn initial_state() -> Vec<i64> {
    Computer::parse_program(&input())
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> AdventResult {
    let mut computer = Computer::new(initial_state());
    computer.buffer_input(1);
    computer.start();
    assert!(computer.is_halted());
    computer.result_last_output()
}

pub fn part2() -> AdventResult {
    let mut computer = Computer::new(initial_state());
    computer.buffer_input(2);
    computer.start();
    assert!(computer.is_halted());
    computer.result_last_output()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_ex1() {
        let input = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
        let mut computer = Computer::new(Computer::parse_program(input));
        computer.buffer_input(1);
        computer.start();
        assert!(computer.is_halted());
        assert_eq!(&Computer::parse_program(input), computer.get_outputs());
    }

    #[test]
    fn part_ex2() {
        let input = "1102,34915192,34915192,7,4,7,99,0";
        let mut computer = Computer::new(Computer::parse_program(input));
        computer.buffer_input(1);
        computer.start();
        assert!(computer.is_halted());
        assert_eq!(1_219_070_632_396_864, computer.result_last_output());
    }

    #[test]
    fn part_ex3() {
        let input = "104,1125899906842624,99";
        let mut computer = Computer::new(Computer::parse_program(input));
        computer.buffer_input(1);
        computer.start();
        assert!(computer.is_halted());
        assert_eq!(1_125_899_906_842_624, computer.result_last_output());
    }

    #[test]
    fn part1_solution() {
        assert_eq!(2377080455, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(74917, part2());
    }
}
