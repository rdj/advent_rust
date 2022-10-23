mod computer;
use computer::Computer;
use computer::Intcode;

type AdventResult = usize;

use std::collections::HashSet;
use std::fs;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Position(i32, i32);

impl Position {
    fn alignment_parameter(&self) -> usize {
        usize::try_from(self.0 * self.1).unwrap()
    }

    fn manhattan(&self, other: &Position) -> u32 {
        (self.0 - other.0).abs() as u32 + (self.1 - other.1).abs() as u32
    }

    fn neighbors(&self) -> Vec<(Direction, Position)> {
        Direction::ALL.iter().map(|&d| (d, d.of(self))).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Direction {
    North,
    East,
    South,
    West,
}
use Direction::{East, North, South, West};

impl Direction {
    const ALL: [Self; 4] = [North, East, South, West];

    fn from_ascii(c: char) -> Self {
        match c {
            '^' => North,
            '>' => East,
            'v' => South,
            '<' => West,
            _ => panic!(),
        }
    }

    fn left90(&self) -> Direction {
        match self {
            North => West,
            East => North,
            South => East,
            West => South,
        }
    }

    fn of(&self, p: &Position) -> Position {
        let Position(x, y) = *p;
        match self {
            North => Position(x, y - 1),
            East => Position(x + 1, y),
            South => Position(x, y + 1),
            West => Position(x - 1, y),
        }
    }

    fn right90(&self) -> Direction {
        match self {
            North => East,
            East => South,
            South => West,
            West => North,
        }
    }

    fn turn(&self, turn: Turn) -> Direction {
        match turn {
            Turn::Left => self.left90(),
            Turn::Right => self.right90(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Turn {
    Left,
    Right,
}

impl Turn {
    const ALL: [Turn; 2] = [Turn::Left, Turn::Right];
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Move(Turn, u32);

struct Scaffolding {
    locations: HashSet<Position>,
    origin: Position,
    orientation: Direction,
}

impl Scaffolding {
    fn new(ascii: &str) -> Self {
        let mut locations = HashSet::new();
        let mut origin = None;
        let mut orientation = None;

        for (y, line) in ascii.trim().lines().enumerate() {
            let y = i32::try_from(y).unwrap();
            for (x, c) in line.trim().chars().enumerate() {
                if c == '.' {
                    continue;
                }
                let x = i32::try_from(x).unwrap();
                let pos = Position(x, y);
                locations.insert(pos);
                if c != '#' {
                    origin = Some(pos);
                    orientation = Some(Direction::from_ascii(c));
                }
            }
        }

        let origin = origin.unwrap();
        let orientation = orientation.unwrap();

        Scaffolding {
            locations,
            origin,
            orientation,
        }
    }

    fn calibration_parameter(&self) -> usize {
        let mut param = 0;

        for pos in &self.locations {
            if pos
                .neighbors()
                .iter()
                .all(|(_, p)| self.locations.contains(p))
            {
                param += pos.alignment_parameter();
            }
        }

        param
    }

    fn robot_moves(&self) -> Vec<Move> {
        let mut moves = vec![];

        let mut last_pos = self.origin;
        let mut last_heading = self.orientation;

        loop {
            let mut next_pos = last_pos;
            let mut next_heading = last_heading;
            let mut next_turn = Turn::Left;

            for t in Turn::ALL {
                next_turn = t;
                next_heading = last_heading.turn(t);
                let mut lookahead = next_pos;
                while self.locations.contains(&lookahead) {
                    next_pos = lookahead;
                    lookahead = next_heading.of(&lookahead);
                }

                if next_pos != last_pos {
                    break;
                }
            }

            if next_pos == last_pos {
                break;
            }

            moves.push(Move(next_turn, next_pos.manhattan(&last_pos)));

            last_pos = next_pos;
            last_heading = next_heading;
        }

        moves
    }
}

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn intcodes_to_ascii<I>(intcodes: I) -> String
where
    I: Iterator<Item = Intcode>,
{
    intcodes
        .map(|n| char::from(u8::try_from(n).unwrap()))
        .collect()
}

fn ascii_to_intcodes(ascii: &str) -> Vec<Intcode>
{
    ascii.chars().map(|c| Intcode::from(u8::try_from(c).unwrap())).collect()
}


fn do_part1(input: &str) -> AdventResult {
    let mut computer = Computer::new(Computer::parse_program(input));
    computer.start();
    assert!(computer.is_halted());
    let ascii = intcodes_to_ascii(computer.consume_output_buffer());
    let s = Scaffolding::new(&ascii);
    s.calibration_parameter()
}

fn do_part2(input: &str) -> AdventResult {
    let mut computer = Computer::new(Computer::parse_program(input));
    computer.start();
    assert!(computer.is_halted());
    let s = Scaffolding::new(&intcodes_to_ascii(computer.consume_output_buffer()));
    println!("{:?}", s.robot_moves());

    // The resulting move list for my input:
    //
    // L,10,R,8,R,6,R,10,L,12,R,8,L,12,L,10,R,8,R,6,R,10,L,12,R,8,L,12,L,10,R,8,R,8,L,10,R,8,R,8,L,12,R,8,L,12,L,10,R,8,R,6,R,10,L,10,R,8,R,8,L,10,R,8,R,6,R,10
    //
    // I actually think that greedy subsequences bounded by the 20
    // character limit will produce the ABC encoding, but since I was
    // inspecting it anyway to figure out a reasonable strategy I
    // realized it was trivial to just produce the input program by
    // hand.

    computer = Computer::new(Computer::parse_program(input));

    let ascii_input="\
A,B,A,B,C,C,B,A,C,A
L,10,R,8,R,6,R,10
L,12,R,8,L,12
L,10,R,8,R,8
n
";

    let inputs = ascii_to_intcodes(ascii_input);
    for input in inputs {
        computer.buffer_input(input);
    }

    computer.write(0, 2);

    computer.start();
    assert!(computer.is_halted());

    let outputs: Vec<_> = computer.consume_output_buffer().collect();
    *(outputs.last().unwrap()) as usize
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
        let ascii = "\
..#..........
..#..........
#######...###
#.#...#...#.#
#############
..#...#...#..
..#####...^..";
        let s = Scaffolding::new(ascii);
        assert_eq!(76, s.calibration_parameter());
    }

    #[test]
    fn part2_example() {
        let ascii = "\
#######...#####
#.....#...#...#
#.....#...#...#
......#...#...#
......#...###.#
......#.....#.#
^########...#.#
......#.#...#.#
......#########
........#...#..
....#########..
....#...#......
....#...#......
....#...#......
....#####......";
        let s = Scaffolding::new(ascii);

        let moves = vec![
            Move(Turn::Right, 8),
            Move(Turn::Right, 8),
            Move(Turn::Right, 4),
            Move(Turn::Right, 4),
            Move(Turn::Right, 8),
            Move(Turn::Left, 6),
            Move(Turn::Left, 2),
            Move(Turn::Right, 4),
            Move(Turn::Right, 4),
            Move(Turn::Right, 8),
            Move(Turn::Right, 8),
            Move(Turn::Right, 8),
            Move(Turn::Left, 6),
            Move(Turn::Left, 2),
        ];

        assert_eq!(moves, s.robot_moves());
    }

    #[test]
    fn part1_solution() {
        assert_eq!(7328, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(1289413, part2());
    }
}
