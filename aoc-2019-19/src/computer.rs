#![allow(dead_code)]

use std::collections::VecDeque;
use std::mem::swap;

pub type Intcode = i64;

const OP_ADD: Intcode = 1;
const OP_MUL: Intcode = 2;
const OP_STORE_INPUT: Intcode = 3;
const OP_WRITE_OUTPUT: Intcode = 4;
const OP_JUMP_IF_TRUE: Intcode = 5;
const OP_JUMP_IF_FALSE: Intcode = 6;
const OP_LESS_THAN: Intcode = 7;
const OP_EQUALS: Intcode = 8;
const OP_ADJUST_RELATIVE_BASE: Intcode = 9;
const OP_HALT: Intcode = 99;

const OP_PARAMETER_BASE: Intcode = 10;
const OP_PARAMETER_BASE_POS: u32 = 3;

const PARAM_TYPE_POSITION: Intcode = 0;
const PARAM_TYPE_IMMEDIATE: Intcode = 1;
const PARAM_TYPE_RELATIVE: Intcode = 2;

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
    Position(Intcode),
    Immediate(Intcode),
    Relative(Intcode),
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
struct OpDecoder(Intcode);

impl OpDecoder {
    fn opcode(&self) -> Intcode {
        self.0 % OP_PARAMETER_BASE.pow(OP_PARAMETER_BASE_POS - 1)
    }

    fn param_type(&self, argno: u32) -> Intcode {
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

pub struct Computer {
    memory: Vec<Intcode>,
    ip: Intcode,
    state: ComputerState,
    inputs: VecDeque<Intcode>,
    outputs: VecDeque<Intcode>,
    op: Option<Op>,
    relative_base: Intcode,
}

impl Computer {
    pub fn parse_program(prog: &str) -> Vec<Intcode> {
        prog.trim().split(",").map(|s| s.parse().unwrap()).collect()
    }

    pub fn new(memory: Vec<Intcode>) -> Self {
        Computer {
            memory,
            inputs: VecDeque::new(),
            ip: 0,
            state: ComputerState::Initial,
            outputs: VecDeque::new(),
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
        F: FnOnce(Intcode, Intcode) -> Intcode,
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

    pub fn buffer_input(&mut self, input: Intcode) {
        self.inputs.push_back(input);
    }

    fn compute(&mut self) {
        while self.state == ComputerState::Running {
            self.read_next_instruction();
            self.execute();
        }
    }

    fn deref(&self, param: &Parameter) -> Intcode {
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

    pub fn consume_output(&mut self) -> Option<Intcode> {
        self.outputs.pop_front()
    }

    pub fn consume_output_buffer(&mut self) -> impl Iterator<Item = Intcode> {
        let mut outputs = VecDeque::new();
        swap(&mut outputs, &mut self.outputs);
        outputs.into_iter()
    }

    pub fn is_awaiting_input(&self) -> bool {
        ComputerState::AwaitingInput == self.state
    }

    pub fn is_halted(&self) -> bool {
        ComputerState::Halted == self.state
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

    fn read(&self, p: Intcode) -> Intcode {
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

    fn read_param_and_advance(&mut self, param_type: Intcode) -> Parameter {
        let value = self.read_word_and_advance();
        match param_type {
            PARAM_TYPE_POSITION => Position(value),
            PARAM_TYPE_IMMEDIATE => Immediate(value),
            PARAM_TYPE_RELATIVE => Relative(value),
            x => panic!("Unknown parameter type {x}"),
        }
    }

    fn read_word_and_advance(&mut self) -> Intcode {
        let n = self.read(self.ip);
        self.ip += 1;
        n
    }

    fn read_input(&mut self) -> Option<Intcode> {
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

    pub fn result_addr0(&self) -> Intcode {
        assert_eq!(ComputerState::Halted, self.state);
        self.read(0)
    }

    pub fn result_last_output(&self) -> Intcode {
        assert_eq!(ComputerState::Halted, self.state);
        *self.outputs.iter().last().unwrap()
    }

    pub fn resume(&mut self) {
        assert_eq!(ComputerState::AwaitingInput, self.state);
        assert_ne!(0, self.inputs.len());

        self.state = ComputerState::Running;
        self.execute();

        self.compute();
    }

    pub fn start(&mut self) {
        assert_eq!(ComputerState::Initial, self.state);
        assert_eq!(0, self.ip);

        self.state = ComputerState::Running;
        self.compute();
    }

    pub fn start_or_resume(&mut self) {
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

    pub fn write(&mut self, p: Intcode, n: Intcode) {
        assert!(p >= 0);
        while self.memory.len() - 1 < p as usize {
            self.memory.push(0);
        }

        let p = self.memory.get_mut(p as usize).unwrap();
        *p = n;
    }

    fn write_output(&mut self, pa: Parameter) {
        let value = self.deref(&pa);
        self.outputs.push_back(value);
    }
}
