// -*- compile-command: "cargo test -- --show-output" -*-

#![allow(dead_code)]

use std::collections::VecDeque;
use std::fs;

struct Computer {
    memory: Vec<i32>,
    ip: i32,
    halted: bool,
    inputs: VecDeque<i32>,
    outputs: Vec<i32>,
}

const ADDR_NOUN: i32 = 1;
const ADDR_VERB: i32 = 2;

const OP_ADD: i32 = 1;
const OP_MUL: i32 = 2;
const OP_STORE_INPUT: i32 = 3;
const OP_WRITE_OUTPUT: i32 = 4;
const OP_JUMP_IF_TRUE: i32 = 5;
const OP_JUMP_IF_FALSE: i32 = 6;
const OP_LESS_THAN: i32 = 7;
const OP_EQUALS: i32 = 8;
const OP_HALT: i32 = 99;

enum Op {
    Add(Parameter, Parameter, Parameter),
    Mul(Parameter, Parameter, Parameter),
    StoreInput(Parameter),
    WriteOutput(Parameter),
    JumpIfTrue(Parameter, Parameter),
    JumpIfFalse(Parameter, Parameter),
    LessThan(Parameter, Parameter, Parameter),
    Equals(Parameter, Parameter, Parameter),
    Halt,
}

enum Parameter {
    Position(i32),
    Immediate(i32),
}
use Parameter::*;

const PARAMETER_BASE: i32 = 10;
const PARAMETER_BASE_POS: u32 = 3;
const PARAM_POSITION: i32 = 0;
const PARAM_IMMEDIATE: i32 = 1;

impl Parameter {
    fn deref(&self, computer: &Computer) -> i32 {
        match self {
            Position(p) => computer.read(*p),
            Immediate(n) => *n,
        }
    }

    fn read_and_advance(computer: &mut Computer, opcode: i32, argno: u32) -> Parameter {
        let value = computer.read_and_advance();
        match opcode % (PARAMETER_BASE.pow(argno + PARAMETER_BASE_POS))
            / PARAMETER_BASE.pow(argno + PARAMETER_BASE_POS - 1)
        {
            PARAM_POSITION => Position(value),
            PARAM_IMMEDIATE => Immediate(value),
            x => panic!("Unknown parameter type {x}"),
        }
    }
}

impl Computer {
    fn new(memory: Vec<i32>, inputs: Vec<i32>) -> Self {
        Computer {
            memory,
            inputs: VecDeque::from(inputs),
            ip: 0,
            halted: false,
            outputs: vec![],
        }
    }

    fn binary_op<F>(&mut self, pa: Parameter, pb: Parameter, pc: Parameter, f: F)
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let a = pa.deref(&self);
        let b = pb.deref(&self);
        let c = f(a, b);

        if let Position(p) = pc {
            self.write(p, c);
        } else {
            panic!("Binary op arg c must be Position type");
        }
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
            Op::StoreInput(pa) => self.store_input(pa),
            Op::WriteOutput(pa) => self.write_output(pa),
            Op::JumpIfTrue(pa, pb) => self.jump_if_true(pa, pb),
            Op::JumpIfFalse(pa, pb) => self.jump_if_false(pa, pb),
            Op::LessThan(pa, pb, pc) => {
                self.binary_op(pa, pb, pc, |a, b| if a < b { 1 } else { 0 })
            }
            Op::Equals(pa, pb, pc) => self.binary_op(pa, pb, pc, |a, b| if a == b { 1 } else { 0 }),
            Op::Halt => self.halted = true,
        }
    }

    fn jump_if_false(&mut self, pa: Parameter, pb: Parameter) {
        let cond = pa.deref(self);
        if cond == 0 {
            let addr = pb.deref(self);
            self.ip = addr;
        }
    }

    fn jump_if_true(&mut self, pa: Parameter, pb: Parameter) {
        let cond = pa.deref(self);
        if cond != 0 {
            let addr = pb.deref(self);
            self.ip = addr;
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

    fn read_input(&mut self) -> i32 {
        self.inputs.pop_front().expect("Ran out of inputs")
    }

    fn read_next_instruction(&mut self) -> Op {
        let opcode = self.read_and_advance();
        match opcode % 100 {
            OP_ADD => Op::Add(
                Parameter::read_and_advance(self, opcode, 0),
                Parameter::read_and_advance(self, opcode, 1),
                Parameter::read_and_advance(self, opcode, 2),
            ),
            OP_MUL => Op::Mul(
                Parameter::read_and_advance(self, opcode, 0),
                Parameter::read_and_advance(self, opcode, 1),
                Parameter::read_and_advance(self, opcode, 2),
            ),
            OP_STORE_INPUT => Op::StoreInput(Parameter::read_and_advance(self, opcode, 0)),
            OP_WRITE_OUTPUT => Op::WriteOutput(Parameter::read_and_advance(self, opcode, 0)),
            OP_JUMP_IF_TRUE => Op::JumpIfTrue(
                Parameter::read_and_advance(self, opcode, 0),
                Parameter::read_and_advance(self, opcode, 1),
            ),
            OP_JUMP_IF_FALSE => Op::JumpIfFalse(
                Parameter::read_and_advance(self, opcode, 0),
                Parameter::read_and_advance(self, opcode, 1),
            ),
            OP_LESS_THAN => Op::LessThan(
                Parameter::read_and_advance(self, opcode, 0),
                Parameter::read_and_advance(self, opcode, 1),
                Parameter::read_and_advance(self, opcode, 2),
            ),
            OP_EQUALS => Op::Equals(
                Parameter::read_and_advance(self, opcode, 0),
                Parameter::read_and_advance(self, opcode, 1),
                Parameter::read_and_advance(self, opcode, 2),
            ),
            OP_HALT => Op::Halt,
            x => panic!("Unknown opcode {x}"),
        }
    }

    fn restore_state(&mut self, noun: i32, verb: i32) {
        self.write(ADDR_NOUN, noun);
        self.write(ADDR_VERB, verb);
    }

    fn result_addr0(&self) -> i32 {
        assert!(self.halted);
        self.read(0)
    }

    fn result_last_output(&self) -> i32 {
        assert!(self.halted);
        *self.outputs.iter().last().unwrap()
    }

    fn store_input(&mut self, pa: Parameter) {
        let input = self.read_input();
        if let Position(p) = pa {
            self.write(p, input);
        } else {
            panic!("StoreInput arg0 must be Position type");
        }
    }

    fn write(&mut self, p: i32, n: i32) {
        assert!(p >= 0);
        let p = self.memory.get_mut(p as usize).unwrap();
        *p = n;
    }

    fn write_output(&mut self, pa: Parameter) {
        let value = pa.deref(self);
        self.outputs.push(value);
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn initial_state() -> Vec<i32> {
    input()
        .trim()
        .split(",")
        .map(|s| s.parse().unwrap())
        .collect()
}

pub fn part1() -> i32 {
    let inputs: Vec<i32> = vec![1];
    let mut computer = Computer::new(initial_state(), inputs);
    computer.compute();
    computer.result_last_output()
}

pub fn part2() -> i32 {
    let inputs: Vec<i32> = vec![5];
    let mut computer = Computer::new(initial_state(), inputs);
    computer.compute();
    computer.result_last_output()
}

#[cfg(test)]
mod test {
    use super::*;

    fn compute(program: Vec<i32>, inputs: Vec<i32>) -> i32 {
        let mut computer = Computer::new(program, inputs);
        computer.compute();
        computer.result_last_output()
    }

    #[test]
    fn test_ne0_imm() {
        let prog: Vec<i32> = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        assert_eq!(0, compute(prog.clone(), vec![0]));
        assert_eq!(1, compute(prog.clone(), vec![-1]));
        assert_eq!(1, compute(prog.clone(), vec![1]));
    }

    #[test]
    fn test_ne0_pos() {
        let prog: Vec<i32> = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        assert_eq!(0, compute(prog.clone(), vec![0]));
        assert_eq!(1, compute(prog.clone(), vec![-1]));
        assert_eq!(1, compute(prog.clone(), vec![1]));
    }

    #[test]
    fn test_eq8_imm() {
        let prog: Vec<i32> = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];
        assert_eq!(1, compute(prog.clone(), vec![8]));
        assert_eq!(0, compute(prog.clone(), vec![7]));
        assert_eq!(0, compute(prog.clone(), vec![9]));
    }

    #[test]
    fn test_eq8_pos() {
        let prog: Vec<i32> = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_eq!(1, compute(prog.clone(), vec![8]));
        assert_eq!(0, compute(prog.clone(), vec![7]));
        assert_eq!(0, compute(prog.clone(), vec![9]));
    }

    #[test]
    fn test_lt8_imm() {
        let prog: Vec<i32> = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];
        assert_eq!(1, compute(prog.clone(), vec![0]));
        assert_eq!(1, compute(prog.clone(), vec![-1]));
        assert_eq!(0, compute(prog.clone(), vec![8]));
        assert_eq!(0, compute(prog.clone(), vec![9]));
    }

    #[test]
    fn test_lt8_pos() {
        let prog: Vec<i32> = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_eq!(1, compute(prog.clone(), vec![0]));
        assert_eq!(1, compute(prog.clone(), vec![-1]));
        assert_eq!(0, compute(prog.clone(), vec![8]));
        assert_eq!(0, compute(prog.clone(), vec![9]));
    }

    #[test]
    fn test_complicated() {
        let prog: Vec<i32> = vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        assert_eq!(999, compute(prog.clone(), vec![-1]));
        assert_eq!(1000, compute(prog.clone(), vec![8]));
        assert_eq!(1001, compute(prog.clone(), vec![9]));
    }

    #[test]
    fn run_part1() {
        assert_eq!(9961446, part1());
    }

    #[test]
    fn run_part2() {
        assert_eq!(742621, part2());
    }
}
