// -*- compile-command: "cargo test -- --show-output" -*-

#![allow(dead_code)]

type AdventResult = i64;

use std::collections::VecDeque;
use std::fs;

const ADDR_NOUN: i64 = 1;
const ADDR_VERB: i64 = 2;

const OP_ADD: i64 = 1;
const OP_MUL: i64 = 2;
const OP_STORE_INPUT: i64 = 3;
const OP_WRITE_OUTPUT: i64 = 4;
const OP_JUMP_IF_TRUE: i64 = 5;
const OP_JUMP_IF_FALSE: i64 = 6;
const OP_LESS_THAN: i64 = 7;
const OP_EQUALS: i64 = 8;
const OP_ADJUST_RELATIVE_BASE: i64 = 9;
const OP_HALT: i64 = 99;

const OP_PARAMETER_BASE: i64 = 10;
const OP_PARAMETER_BASE_POS: u32 = 3;

const PARAM_TYPE_POSITION: i64 = 0;
const PARAM_TYPE_IMMEDIATE: i64 = 1;
const PARAM_TYPE_RELATIVE: i64 = 2;

enum Op {
    Add(Parameter, Parameter, Parameter),
    Mul(Parameter, Parameter, Parameter),
    StoreInput(Parameter),
    WriteOutput(Parameter),
    JumpIfTrue(Parameter, Parameter),
    JumpIfFalse(Parameter, Parameter),
    LessThan(Parameter, Parameter, Parameter),
    Equals(Parameter, Parameter, Parameter),
    AdjustRelativeBase(Parameter),
    Halt,
}

#[derive(Clone, Copy)]
enum Parameter {
    Position(i64),
    Immediate(i64),
    Relative(i64),
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
struct OpDecoder(i64);

impl OpDecoder {
    fn opcode(&self) -> i64 {
        self.0 % OP_PARAMETER_BASE.pow(OP_PARAMETER_BASE_POS - 1)
    }

    fn param_type(&self, argno: u32) -> i64 {
        self.0 % (OP_PARAMETER_BASE.pow(argno + OP_PARAMETER_BASE_POS))
            / OP_PARAMETER_BASE.pow(argno + OP_PARAMETER_BASE_POS - 1)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ComputerState {
    Initial,
    Running,
    Halted,
    AwaitingInput,
}

struct Computer {
    memory: Vec<i64>,
    ip: i64,
    state: ComputerState,
    inputs: VecDeque<i64>,
    outputs: Vec<i64>,
    op: Option<Op>,
    relative_base: i64,
}

impl Computer {
    fn new(memory: Vec<i64>) -> Self {
        Computer {
            memory,
            inputs: VecDeque::new(),
            ip: 0,
            state: ComputerState::Initial,
            outputs: vec![],
            op: None,
            relative_base: 0,
        }
    }

    fn adjust_relative_base(&mut self, pa: Parameter) {
        let a = self.deref(&pa);
        self.relative_base += a;
    }

    fn binary_op<F>(&mut self, pa: Parameter, pb: Parameter, pc: Parameter, f: F)
    where
        F: FnOnce(i64, i64) -> i64,
    {
        let a = self.deref(&pa);
        let b = self.deref(&pb);
        let c = f(a, b);

        match pc {
            Position(p) => self.write(p, c),
            Relative(o) => self.write(o + self.relative_base, c),
            _ => panic!("Binary op arg c must be Position or Relative type"),
        }
    }

    fn buffer_input(&mut self, input: i64) {
        self.inputs.push_back(input);
    }

    fn compute(&mut self) {
        while self.state == ComputerState::Running {
            self.read_next_instruction();
            self.execute();
        }
    }

    fn deref(&self, param: &Parameter) -> i64 {
        match param {
            Position(p) => self.read(*p),
            Immediate(n) => *n,
            Relative(offset) => self.read(self.relative_base + offset),
        }
    }

    fn execute(&mut self) {
        // We deref the parameter values because we need to preserve
        // the op, unmoved, in case we need to pause execution and
        // resume later.
        match self.op.as_ref().expect("expect op to be loaded") {
            Op::Add(pa, pb, pc) => self.binary_op(*pa, *pb, *pc, |a, b| a + b),
            Op::Mul(pa, pb, pc) => self.binary_op(*pa, *pb, *pc, |a, b| a * b),
            Op::StoreInput(pa) => self.store_input(*pa),
            Op::WriteOutput(pa) => self.write_output(*pa),
            Op::JumpIfTrue(pa, pb) => self.jump_if_true(*pa, *pb),
            Op::JumpIfFalse(pa, pb) => self.jump_if_false(*pa, *pb),
            Op::LessThan(pa, pb, pc) => {
                self.binary_op(*pa, *pb, *pc, |a, b| if a < b { 1 } else { 0 })
            }
            Op::Equals(pa, pb, pc) => {
                self.binary_op(*pa, *pb, *pc, |a, b| if a == b { 1 } else { 0 })
            }
            Op::AdjustRelativeBase(pa) => {
                self.adjust_relative_base(*pa);
            }
            Op::Halt => self.state = ComputerState::Halted,
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

    fn read(&self, p: i64) -> i64 {
        assert!(p >= 0);

        if let Some(value) = self.memory.get(p as usize) {
            *value
        } else {
            0
        }            
    }

    fn read_op_and_advance(&mut self) -> OpDecoder {
        OpDecoder(self.read_word_and_advance())
    }

    fn read_param_and_advance(&mut self, param_type: i64) -> Parameter {
        let value = self.read_word_and_advance();
        match param_type {
            PARAM_TYPE_POSITION => Position(value),
            PARAM_TYPE_IMMEDIATE => Immediate(value),
            PARAM_TYPE_RELATIVE => Relative(value),
            x => panic!("Unknown parameter type {x}"),
        }
    }

    fn read_word_and_advance(&mut self) -> i64 {
        let n = self.read(self.ip);
        self.ip += 1;
        n
    }

    fn read_input(&mut self) -> Option<i64> {
        self.inputs.pop_front()
    }

    fn read_next_instruction(&mut self) {
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

        self.op = Some(match op.opcode() {
            OP_ADD => op_read_params!(Add, 3),
            OP_MUL => op_read_params!(Mul, 3),
            OP_STORE_INPUT => op_read_params!(StoreInput, 1),
            OP_WRITE_OUTPUT => op_read_params!(WriteOutput, 1),
            OP_JUMP_IF_TRUE => op_read_params!(JumpIfTrue, 2),
            OP_JUMP_IF_FALSE => op_read_params!(JumpIfFalse, 2),
            OP_LESS_THAN => op_read_params!(LessThan, 3),
            OP_EQUALS => op_read_params!(Equals, 3),
            OP_ADJUST_RELATIVE_BASE => op_read_params!(AdjustRelativeBase, 1),
            OP_HALT => Op::Halt,
            x => panic!("Unknown opcode {x}"),
        });
    }

    fn restore_state(&mut self, noun: i64, verb: i64) {
        self.write(ADDR_NOUN, noun);
        self.write(ADDR_VERB, verb);
    }

    fn result_addr0(&self) -> i64 {
        assert_eq!(ComputerState::Halted, self.state);
        self.read(0)
    }

    fn result_last_output(&self) -> i64 {
        assert_eq!(ComputerState::Halted, self.state);
        *self.outputs.iter().last().unwrap()
    }

    fn resume(&mut self) {
        assert_eq!(ComputerState::AwaitingInput, self.state);
        assert_ne!(0, self.inputs.len());

        self.state = ComputerState::Running;
        self.execute();

        self.compute();
    }

    fn start(&mut self) {
        assert_eq!(ComputerState::Initial, self.state);
        assert_eq!(0, self.ip);

        self.state = ComputerState::Running;
        self.compute();
    }

    fn start_or_resume(&mut self) {
        match &self.state {
            ComputerState::Initial => self.start(),
            ComputerState::AwaitingInput => self.resume(),
            s => panic!("unexpected state {:?}", s),
        }
    }

    fn store_input(&mut self, pa: Parameter) {
        if let Some(input) = self.read_input() {
            match pa {
                Position(p) => self.write(p, input),
                Relative(o) => self.write(o + self.relative_base, input),
                _ => panic!("StoreInput arg0 must be Position or Relative"),
            }
        } else {
            self.state = ComputerState::AwaitingInput;
        }
    }

    fn write(&mut self, p: i64, n: i64) {
        assert!(p >= 0);
        while self.memory.len() - 1 < p as usize {
            self.memory.push(0);
        }

        let p = self.memory.get_mut(p as usize).unwrap();
        *p = n;
    }

    fn write_output(&mut self, pa: Parameter) {
        let value = self.deref(&pa);
        self.outputs.push(value);
    }
}

fn parse_program(prog: &str) -> Vec<i64> {
    prog.trim().split(",").map(|s| s.parse().unwrap()).collect()
}

fn initial_state() -> Vec<i64> {
    parse_program(&input())
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> AdventResult {
    let mut computer = Computer::new(parse_program(&input()));
    computer.buffer_input(1);
    computer.start();
    assert_eq!(ComputerState::Halted, computer.state);
    computer.result_last_output()
}

pub fn part2() -> AdventResult {
    let mut computer = Computer::new(parse_program(&input()));
    computer.buffer_input(2);
    computer.start();
    assert_eq!(ComputerState::Halted, computer.state);
    computer.result_last_output()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_ex1() {
        let input = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
        let mut computer = Computer::new(parse_program(input));
        computer.buffer_input(1);
        computer.start();
        assert_eq!(ComputerState::Halted, computer.state);
        assert_eq!(parse_program(input), computer.outputs);
    }

    #[test]
    fn part_ex2() {
        let input = "1102,34915192,34915192,7,4,7,99,0";
        let mut computer = Computer::new(parse_program(input));
        computer.buffer_input(1);
        computer.start();
        assert_eq!(ComputerState::Halted, computer.state);
        assert_eq!(1_219_070_632_396_864, computer.result_last_output());
    }

    #[test]
    fn part_ex3() {
        let input = "104,1125899906842624,99";
        let mut computer = Computer::new(parse_program(input));
        computer.buffer_input(1);
        computer.start();
        assert_eq!(ComputerState::Halted, computer.state);
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
