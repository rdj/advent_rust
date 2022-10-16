// -*- compile-command: "cargo test -- --show-output" -*-

#![allow(dead_code)]

mod computer;
use computer::Computer;
use computer::Intcode;

type AdventResult = usize;

use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PaintColor {
    White,
    Black,
}

impl PaintColor {
    fn from_code(code: Intcode) -> Self {
        match code {
            0 => PaintColor::Black,
            1 => PaintColor::White,
            x => panic!("Unknown paint code `{}`", x),
        }
    }

    fn to_code(&self) -> Intcode {
        match self {
            PaintColor::Black => 0,
            PaintColor::White => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position(i32, i32);

impl Position {
    fn advance(&mut self, facing: Orientation) {
        let Position(x, y) = self;
        match facing {
            North => *y = *y + 1,
            East => *x = *x + 1,
            South => *y = *y - 1,
            West => *x = *x - 1,
        }
    }
}

enum InputState {
    Paint,
    Turn,
}
use InputState::*;

impl InputState {
    fn next(&self) -> Self {
        match self {
            Paint => Turn,
            Turn => Paint,
        }
    }
}

#[derive(Debug)]
enum TurnDirection {
    Left,
    Right,
}
use TurnDirection::*;

impl TurnDirection {
    fn from_code(code: Intcode) -> Self {
        match code {
            0 => Left,
            1 => Right,
            x => panic!("Unknown turn direction `{x}`"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Orientation {
    North,
    East,
    South,
    West,
}
use Orientation::*;

impl Orientation {
    fn turn(&self, dir: &TurnDirection) -> Self {
        match (self, dir) {
            (North, Left) => West,
            (North, Right) => East,
            (East, Left) => North,
            (East, Right) => South,
            (South, Left) => East,
            (South, Right) => West,
            (West, Left) => South,
            (West, Right) => North,
        }
    }
}



struct Robot {
    panels: HashMap<Position, PaintColor>,
    computer: Computer,
    position: Position,
    input_state: InputState,
    facing: Orientation,
}

impl Robot {
    fn new(program: Vec<Intcode>) -> Self {
        Robot {
            panels: HashMap::new(),
            computer: Computer::new(program),
            position: Position(0, 0),
            input_state: InputState::Paint,
            facing: North,
        }
    }

    fn advance(&mut self) {
        self.position.advance(self.facing);
    }

    fn get_panel_color(&self, pos: Position) -> PaintColor {
        *self.panels.get(&pos).unwrap_or(&PaintColor::Black)
    }

    fn paint_panel(&mut self, color: PaintColor) {
        self.panels.insert(self.position, color);
    }

    fn panel_string(&self) -> String {
        let mut s = String::new();

        let x_coords: Vec<_> = self.panels.keys().map(|Position(x, _)| *x).collect();
        let x_min = *x_coords.iter().min().unwrap();
        let x_max = *x_coords.iter().max().unwrap();

        let y_coords: Vec<_> = self.panels.keys().map(|Position(_, y)| *y).collect();
        let y_min = *y_coords.iter().min().unwrap();
        let y_max = *y_coords.iter().max().unwrap();

        for y in (y_min..=y_max).rev() {
            for x in x_min..=x_max {
                s += 
                    match self.get_panel_color(Position(x, y)) {
                        PaintColor::White => "▮",
                        PaintColor::Black => " ",
                    };
            }
            s += "\n";
        }

        println!("{}", s);

        s
    }

    fn run(&mut self) {
        while !self.computer.is_halted() {
            self.computer.start_or_resume();
            while let Some(output) = self.computer.consume_output() {
                match self.input_state {
                    Paint => self.paint_panel(PaintColor::from_code(output)),
                    Turn => self.turn(TurnDirection::from_code(output)),
                }
                self.input_state = self.input_state.next();
            }
            if self.computer.is_awaiting_input() {
                self.computer.buffer_input(self.get_panel_color(self.position).to_code());
            }
        }
    }

    fn turn(&mut self, dir: TurnDirection) {
        self.facing = self.facing.turn(&dir);
        self.advance();
    }

    fn unique_panels_painted(&self) -> usize {
        self.panels.len()
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> AdventResult {
    let mut robot = Robot::new(Computer::parse_program(&input()));
    robot.run();
    robot.unique_panels_painted()
}

pub fn part2() -> String {
    let mut robot = Robot::new(Computer::parse_program(&input()));
    robot.paint_panel(PaintColor::White);
    robot.run();
    robot.panel_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_example() {
        // no example input given
    }

    #[test]
    fn part2_example() {
        // no example input given
    }
    
    #[test]
    fn part1_solution() {
        assert_eq!(1907, part1());
    }

    #[test]
    fn part2_solution() {
        //  ▮▮  ▮▮▮  ▮▮▮▮ ▮  ▮ ▮▮▮▮  ▮▮  ▮▮▮▮  ▮▮    
        // ▮  ▮ ▮  ▮ ▮    ▮ ▮     ▮ ▮  ▮ ▮    ▮  ▮   
        // ▮  ▮ ▮▮▮  ▮▮▮  ▮▮     ▮  ▮    ▮▮▮  ▮      
        // ▮▮▮▮ ▮  ▮ ▮    ▮ ▮   ▮   ▮ ▮▮ ▮    ▮ ▮▮   
        // ▮  ▮ ▮  ▮ ▮    ▮ ▮  ▮    ▮  ▮ ▮    ▮  ▮   
        // ▮  ▮ ▮▮▮  ▮▮▮▮ ▮  ▮ ▮▮▮▮  ▮▮▮ ▮     ▮▮▮
        let solution = "  ▮▮  ▮▮▮  ▮▮▮▮ ▮  ▮ ▮▮▮▮  ▮▮  ▮▮▮▮  ▮▮    \n ▮  ▮ ▮  ▮ ▮    ▮ ▮     ▮ ▮  ▮ ▮    ▮  ▮   \n ▮  ▮ ▮▮▮  ▮▮▮  ▮▮     ▮  ▮    ▮▮▮  ▮      \n ▮▮▮▮ ▮  ▮ ▮    ▮ ▮   ▮   ▮ ▮▮ ▮    ▮ ▮▮   \n ▮  ▮ ▮  ▮ ▮    ▮ ▮  ▮    ▮  ▮ ▮    ▮  ▮   \n ▮  ▮ ▮▮▮  ▮▮▮▮ ▮  ▮ ▮▮▮▮  ▮▮▮ ▮     ▮▮▮   \n";
        assert_eq!(solution, part2());
    }
}
