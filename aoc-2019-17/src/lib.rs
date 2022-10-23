#![cfg_attr(not(test), allow(dead_code))]

mod computer;
use computer::Computer;
use computer::Intcode;

type AdventResult = usize;

use std::collections::HashSet;
use std::fs;

use regex::Regex;

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

    fn to_ascii(&self) -> char {
        match self {
            Turn::Left => 'L',
            Turn::Right => 'R',
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Move(Turn, u32);

impl Move {
    fn to_ascii(&self) -> String {
        format!("{},{}", self.0.to_ascii(), self.1)
    }
}

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

    fn program_moves_full(&self) -> String {
        self.robot_moves()
            .into_iter()
            .map(|m| m.to_ascii())
            .collect::<Vec<_>>()
            .join(",")
    }

    // Our full program looks something like
    //   L1,R1,L10,L8,R4,...
    //
    // We want to find three subsequences of moves that are repeated
    // and replace their occurences with A,B,C.
    //
    // This is pretty hard in general, but we do have a constraint and
    // then can make some simplifying assumptions.
    //
    // The constraint we're given is that no part of the program can
    // be longer than 20 characters.
    //
    // Our compressed output "A,B,A,C,..." can only have at most 10
    // ABC symbols in it (need a comma after each). So each ABC needs
    // to encode a little more than 15.2 characters from the full
    // program. Our sequences need to be long, then.
    //
    // Let's try starting at the beginning of the string and finding
    // the longest sequence that has a repeat later in the string,
    // call that A, and then repeat.
    //
    // Not the most general solution, but it's scoped enough to be
    // doable as part2 of this AOC.
    //
    fn program_moves_compressed(&self) -> String {
        // the format! macro only takes a literal string so can't make
        // the RE template a const
        const SUBSEQ_MIN_LEN: usize = 3;
        const SUBSEQ_MAX_LEN: usize = 20;
        const RE_ANCHOR_COUNT: usize = 2;
        const REPLACEMENTS: [&str; 3] = ["A", "B", "C"];

        let full_prog = self.program_moves_full();
        let mut compressed_prog = full_prog.clone();
        let mut expansions = vec![];

        'abc: for abc in REPLACEMENTS {
            let mut maxlen = SUBSEQ_MAX_LEN - RE_ANCHOR_COUNT;
            while maxlen >= SUBSEQ_MIN_LEN {
                let re = Regex::new(&format!(
                    "[LR][LR0-9,]{{{},{}}}[0-9],",
                    SUBSEQ_MIN_LEN, maxlen
                ))
                .unwrap();
                let m = re
                    .find(&compressed_prog)
                    .expect("RE should always match something");
                let subseq_comma = m.as_str();
                let subseq = &subseq_comma[..subseq_comma.len() - 1];
                let trial = compressed_prog.replace(subseq, abc);
                if compressed_prog.len() - trial.len() > subseq.len() {
                    expansions.push(String::from(subseq));
                    compressed_prog = trial;
                    continue 'abc;
                }

                maxlen = subseq.len() - RE_ANCHOR_COUNT - 1;
            }

            panic!("failed to find a subseq for {}", abc);
        }

        compressed_prog += "\n";
        for ex in &expansions {
            compressed_prog += ex;
            compressed_prog += "\n";
        }

        compressed_prog += "n\n"; // no live camera

        compressed_prog
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

fn ascii_to_intcodes(ascii: &str) -> Vec<Intcode> {
    ascii
        .chars()
        .map(|c| Intcode::from(u8::try_from(c).unwrap()))
        .collect()
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

    let ascii_input = s.program_moves_compressed();

    computer = Computer::new(Computer::parse_program(input));

    let inputs = ascii_to_intcodes(&ascii_input);
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

        let full_prog = "R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2";
        assert_eq!(full_prog, s.program_moves_full());

        // My solution produces a different program for the example
        // than what was given, so instead of comparing I will just
        // make sure that it follows the rules and expands to the
        // epxected full program.

        let compressed = s.program_moves_compressed();
        for line in compressed.lines() {
            assert!(line.len() <= 20);
        }

        let mut lines = compressed.lines();
        let mut main = String::from(lines.next().unwrap());
        let a = lines.next().unwrap();
        let b = lines.next().unwrap();
        let c = lines.next().unwrap();
        assert_eq!(Some("n"), lines.next());
        assert_eq!(None, lines.next());

        main = main.replace("A", a);
        main = main.replace("B", b);
        main = main.replace("C", c);
        assert_eq!(main, full_prog);
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
