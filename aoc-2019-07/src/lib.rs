// -*- compile-command: "cargo test -- --show-output" -*-

#![allow(dead_code)]

use std::collections::VecDeque;
use std::fs;

use itertools::Itertools;

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

const OP_PARAMETER_BASE: i32 = 10;
const OP_PARAMETER_BASE_POS: u32 = 3;

const PARAM_TYPE_POSITION: i32 = 0;
const PARAM_TYPE_IMMEDIATE: i32 = 1;

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

/// Decodes opcode words.
///
/// To decode, regard the word as a base-10 number. The 2 least
/// significant digits encode the operator type. The remaining digits
/// encode the types of the parameters: the 3rd least sigificant digit
/// for first parameter, the 4th for the second, etc.
///
/// Note that leading zeroes are implied if the decimal representation
/// has fewer digits than required.
///
/// # Example
///
///   1002
///  |||||
///  |||||
///  |||++- Op type = 02 (OP_MUL)
///  ||+--- Param 0 type = 0 (PARAM_TYPE_POSTIION)
///  |+---- Param 1 type = 1 (PARAM_TYPE_IMMEDIATE)
///  +----- Param 2 type = 0 (PARAM_TYPE_POSITION)
struct OpDecoder(i32);

impl OpDecoder {
    fn opcode(&self) -> i32 {
        self.0 % OP_PARAMETER_BASE.pow(OP_PARAMETER_BASE_POS - 1)
    }

    fn param_type(&self, argno: u32) -> i32 {
        self.0 % (OP_PARAMETER_BASE.pow(argno + OP_PARAMETER_BASE_POS))
            / OP_PARAMETER_BASE.pow(argno + OP_PARAMETER_BASE_POS - 1)
    }
}

struct Computer {
    memory: Vec<i32>,
    ip: i32,
    halted: bool,
    inputs: VecDeque<i32>,
    outputs: Vec<i32>,
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
        let a = self.deref(&pa);
        let b = self.deref(&pb);
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

    fn deref(&self, param: &Parameter) -> i32 {
        match param {
            Position(p) => self.read(*p),
            Immediate(n) => *n,
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
        let cond = self.deref(&pa);
        if cond == 0 {
            let addr = self.deref(&pb);
            self.ip = addr;
        }
    }

    fn jump_if_true(&mut self, pa: Parameter, pb: Parameter) {
        let cond = self.deref(&pa);
        if cond != 0 {
            let addr = self.deref(&pb);
            self.ip = addr;
        }
    }

    fn read(&self, p: i32) -> i32 {
        assert!(p >= 0);
        *self.memory.get(p as usize).unwrap()
    }

    fn read_op_and_advance(&mut self) -> OpDecoder {
        OpDecoder(self.read_word_and_advance())
    }

    fn read_param_and_advance(&mut self, param_type: i32) -> Parameter {
        let value = self.read_word_and_advance();
        match param_type {
            PARAM_TYPE_POSITION => Position(value),
            PARAM_TYPE_IMMEDIATE => Immediate(value),
            x => panic!("Unknown parameter type {x}"),
        }
    }

    fn read_word_and_advance(&mut self) -> i32 {
        let n = self.read(self.ip);
        self.ip += 1;
        n
    }

    fn read_input(&mut self) -> i32 {
        self.inputs.pop_front().expect("Ran out of inputs")
    }

    fn read_next_instruction(&mut self) -> Op {
        let op = self.read_op_and_advance();

        macro_rules! op_read_params_inner {
            ($enum:ident, $($argno:expr),*) => {
                Op::$enum(
                    $(self.read_param_and_advance(op.param_type($argno))),*
                )
            }
        }

        macro_rules! op_read_params {
            ($enum:ident, 1) => {
                op_read_params_inner!($enum, 0)
            };
            ($enum:ident, 2) => {
                op_read_params_inner!($enum, 0, 1)
            };
            ($enum:ident, 3) => {
                op_read_params_inner!($enum, 0, 1, 2)
            };
        }

        match op.opcode() {
            OP_ADD => op_read_params!(Add, 3),
            OP_MUL => op_read_params!(Mul, 3),
            OP_STORE_INPUT => op_read_params!(StoreInput, 1),
            OP_WRITE_OUTPUT => op_read_params!(WriteOutput, 1),
            OP_JUMP_IF_TRUE => op_read_params!(JumpIfTrue, 2),
            OP_JUMP_IF_FALSE => op_read_params!(JumpIfFalse, 2),
            OP_LESS_THAN => op_read_params!(LessThan, 3),
            OP_EQUALS => op_read_params!(Equals, 3),
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
        let value = self.deref(&pa);
        self.outputs.push(value);
    }
}

fn max_thruster(initial: Vec<i32>, amp_count: usize) -> i32 {
    let mut phase_sequences = (0..amp_count as i32).permutations(amp_count);

    let mut max = None;

    while let Some(phase_sequence) = phase_sequences.next() {
        let mut input_signal = 0;
        let mut seq = phase_sequence.iter();

        while let Some(phase) = seq.next() {
            let mut computer = Computer::new(initial.clone(), vec![*phase, input_signal]);
            computer.compute();

            assert_eq!(1, computer.outputs.len());
            let result = *computer.outputs.first().unwrap();

            max = match max {
                None => Some(result),
                Some(loser) if result > loser => Some(result),
                _ => max,
            };

            input_signal = result;
        }
    }

    max.unwrap()
}

fn parse_program(prog: &str) -> Vec<i32> {
    prog.trim().split(",").map(|s| s.parse().unwrap()).collect()
}

fn initial_state() -> Vec<i32> {
    parse_program(&input())
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> i32 {
    max_thruster(initial_state(), 5)
}

pub fn part2() -> i32 {
    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ex_part1() {
        let input = "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0";
        assert_eq!(43210, max_thruster(parse_program(&input), 5));
        let input = "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0";
        assert_eq!(54321, max_thruster(parse_program(&input), 5));
        let input = "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0";
        assert_eq!(65210, max_thruster(parse_program(&input), 5));
    }

    #[test]
    fn ex_part2() {
        panic!("put example here");
    }

    #[test]
    fn run_part1() {
        assert_eq!(116680, part1());
    }

    #[test]
    fn run_part2() {
        assert_eq!(i32::MAX, part2());
    }
}
