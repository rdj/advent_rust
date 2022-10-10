// -*- compile-command: "cargo test -- --show-output" -*-

use std::fs;

struct Computer {
    memory: Vec<i32>,
    ip: i32,
    halted: bool,
}

const ADDR_NOUN: i32 = 1;
const ADDR_VERB: i32 = 2;

const OP_ADD: i32 = 1;
const OP_MUL: i32 = 2;
const OP_HALT: i32 = 99;

enum Op {
    Add(i32, i32, i32),
    Mul(i32, i32, i32),
    Halt
}

impl Computer {
    fn new(memory: Vec<i32>) -> Self {
        Computer { memory, ip: 0, halted: false }
    }

    fn binary_op<F>(&mut self, pa: i32, pb: i32, pc: i32, f: F)
    where
        F: FnOnce(i32, i32) -> i32
    {
        let a = self.read(pa);
        let b = self.read(pb);
        let c = f(a, b);
        self.write(pc, c);
    }

    fn compute(&mut self) {
        assert!(!self.halted);
        assert_eq!(0, self.ip);

        while !self.halted {
            let op = self.read_next_instruction();
            self.execute(op);
        }
    }

    fn execute(&mut self, op: Op) {
        match op {
            Op::Add(pa, pb, pc) => self.binary_op(pa, pb, pc, |a, b| a + b),
            Op::Mul(pa, pb, pc) => self.binary_op(pa, pb, pc, |a, b| a * b),
            Op::Halt => self.halted = true,
        }
    }

    fn read(&self, p: i32) -> i32 {
        assert!(p >= 0);
        *self.memory.get(p as usize).unwrap()
    }

    fn read_and_advance(&mut self) -> i32 {
        let n = self.read(self.ip);
        self.ip += 1;
        n
    }

    fn read_next_instruction(&mut self) -> Op {
        let opcode = self.read_and_advance();
        match opcode {
            OP_ADD => Op::Add(self.read_and_advance(), self.read_and_advance(), self.read_and_advance()),
            OP_MUL => Op::Mul(self.read_and_advance(), self.read_and_advance(), self.read_and_advance()),
            OP_HALT => Op::Halt,
            x => panic!("Unknown opcode {x}")
        }
    }

    fn restore_state(&mut self, noun: i32, verb: i32) {
        self.write(ADDR_NOUN, noun);
        self.write(ADDR_VERB, verb);
    }

    fn result(&self) -> i32 {
        assert!(self.halted);
        self.read(0)
    }

    fn write(&mut self, p: i32, n: i32) {
        assert!(p >= 0);
        let p = self.memory.get_mut(p as usize).unwrap();
        *p = n;
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn initial_state() -> Vec<i32> {
    input().trim().split(",").map(|s| s.parse().unwrap()).collect()
}

pub fn part1() -> i32 {
    let mut computer = Computer::new(initial_state());
    computer.restore_state(12, 2);
    computer.compute();
    computer.result()
}

pub fn part2() -> i32 {
    let target_output = 19690720;
    let initial = initial_state();

    for noun in 0..=99 {
        for verb in 0..=99 {
            let mut computer = Computer::new(initial.clone());
            computer.restore_state(noun, verb);
            computer.compute();
            if computer.result() == target_output {
                return 100 * noun + verb;
            }
        }
    }
    panic!("Not found");
}

#[cfg(test)]
mod test {
    use super::*;

    fn compute(memory: Vec<i32>) -> Vec<i32> {
        let mut computer = Computer::new(memory);
        computer.compute();
        computer.memory
    }

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
        assert_eq!(6327510, part1());
    }

    #[test]
    fn run_part2() {
        assert_eq!(4112, part2());
    }
}
