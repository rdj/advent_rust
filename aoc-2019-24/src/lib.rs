#![allow(dead_code, unused_variables)]

use std::collections::HashSet;
use std::fs;

const RECURSIVE_STEPS: usize = 200;
const DIM: usize = 5;

const ADJACENCY_MASKS: [u32; DIM * DIM] = [
    0b0000000000000000000100010,
    0b0000000000000000001000101,
    0b0000000000000000010001010,
    0b0000000000000000100010100,
    0b0000000000000001000001000,
    0b0000000000000010001000001,
    0b0000000000000100010100010,
    0b0000000000001000101000100,
    0b0000000000010001010001000,
    0b0000000000100000100010000,
    0b0000000001000100000100000,
    0b0000000010001010001000000,
    0b0000000100010100010000000,
    0b0000001000101000100000000,
    0b0000010000010001000000000,
    0b0000100010000010000000000,
    0b0001000101000100000000000,
    0b0010001010001000000000000,
    0b0100010100010000000000000,
    0b1000001000100000000000000,
    0b0001000001000000000000000,
    0b0010100010000000000000000,
    0b0101000100000000000000000,
    0b1010001000000000000000000,
    0b0100010000000000000000000,
];

fn compute_masks() {
    let mut bit = 1;
    println!("[");
    for r in 0..DIM {
        for c in 0..DIM {
            // 9876543210
            let mut mask = 0;
            if r > 0 {
                mask |= bit >> DIM;
            }
            if c > 0 {
                mask |= bit >> 1;
            }
            if c + 1 < DIM {
                mask |= bit << 1;
            }
            if r + 1 < DIM {
                mask |= bit << DIM;
            }

            println!("{:025b},", mask);

            bit <<= 1;
        }
    }
    println!("]");
}

type BugState = u32;
type AdventResult = BugState;

struct RecursiveBugs {
    state: [[BugState; 2 * RECURSIVE_STEPS]; 2],
    current: usize,
}

struct Bugs {
    state: BugState,
}

impl Bugs {
    fn new(state: BugState) -> Self {
        Bugs { state }
    }

    fn advance(&mut self) {
        let mut new_state = 0;
        let mut bit = 1;

        for i in 0..DIM * DIM {
            let mask = ADJACENCY_MASKS[i];

            let masked_state = self.state & mask;
            let bugs_adjacent = masked_state.count_ones();

            let has_bug = 0 != self.state & bit;

            if has_bug && 1 == bugs_adjacent {
                new_state |= bit;
            } else if !has_bug && (1 == bugs_adjacent || 2 == bugs_adjacent) {
                new_state |= bit;
            }

            bit <<= 1;
        }

        self.state = new_state;
    }

    fn to_string(&self) -> String {
        let mut s = String::new();
        let mut bit = 1;

        for _ in 0..DIM {
            if s.len() > 0 {
                s += "\n";
            }
            for _ in 0..DIM {
                s.push(if 0 == self.state & bit { '.' } else { '#' });
                bit <<= 1;
            }
        }

        s
    }
}

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn parse_input_part1(input: &str) -> BugState {
    let input: String = input
        .replace("\n", "")
        .replace(".", "0")
        .replace("#", "1")
        .chars()
        .rev()
        .collect();
    BugState::from_str_radix(&input, 2).unwrap()
}

fn do_part1(input: &str) -> AdventResult {
    let mut seen = HashSet::new();

    let initial = parse_input_part1(input);
    let mut bugs = Bugs::new(initial);

    while !seen.contains(&bugs.state) {
        seen.insert(bugs.state);
        bugs.advance();
    }

    bugs.state
}

fn do_part2(input: &str) -> AdventResult {
    todo!()
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
    fn test_compute_masks() {
        super::compute_masks();
        //todo!()
    }

    #[test]
    fn part1_example() {
        let input = "....#\n\
                    #..#.\n\
                    #..##\n\
                    ..#..\n\
                    #....";

        let mut bugs = Bugs::new(parse_input_part1(input));
        assert_eq!(
            bugs.to_string(),
            "\
            ....#\n\
            #..#.\n\
            #..##\n\
            ..#..\n\
            #...."
        );
        bugs.advance();
        assert_eq!(
            bugs.to_string(),
            "\
            #..#.\n\
            ####.\n\
            ###.#\n\
            ##.##\n\
            .##.."
        );
        bugs.advance();
        assert_eq!(
            bugs.to_string(),
            "\
            #####\n\
            ....#\n\
            ....#\n\
            ...#.\n\
            #.###"
        );
        bugs.advance();
        assert_eq!(
            bugs.to_string(),
            "\
            #....\n\
            ####.\n\
            ...##\n\
            #.##.\n\
            .##.#"
        );
        bugs.advance();
        assert_eq!(
            bugs.to_string(),
            "\
            ####.\n\
            ....#\n\
            ##..#\n\
            .....\n\
            ##..."
        );

        assert_eq!(2129920, do_part1(input));
    }

    #[test]
    fn part2_example() {
        todo!()
    }

    #[test]
    fn part1_solution() {
        assert_eq!(3186366, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
