#![allow(dead_code, unused_variables)]

use std::collections::HashSet;
use std::fs;

type BugState = u32;
type AdventResult = BugState;

struct Bugs {
    state: BugState,
    dim: usize,
}

impl Bugs {
    fn new(state: BugState, dim: usize) -> Self {
        Bugs { state, dim }
    }

    fn advance(&mut self) {
        let mut new_state = 0;
        let mut bit = 1;
        for r in 0..self.dim {
            for c in 0..self.dim {
                // 9876543210
                let mut mask = 0;
                if r > 0 {
                    mask |= bit >> self.dim;
                }
                if c > 0 {
                    mask |= bit >> 1;
                }
                if c + 1 < self.dim {
                    mask |= bit << 1;
                }
                if r + 1 < self.dim {
                    mask |= bit << self.dim;
                }

                let masked_state = self.state & mask;
                let bugs_adjacent = masked_state.count_ones();

                let has_bug = 0 != self.state & bit;

            // println!("bit    = {:025b}\n\
            //           mask   = {:025b}\n\
            //           masked = {:025b}\n\
            //           bugs = {}; has_bug = {}\n\
            //           ====", bit, mask, masked_state, bugs_adjacent, has_bug);

                if has_bug && 1 == bugs_adjacent {
                    new_state |= bit;
                } else if !has_bug && (1 == bugs_adjacent || 2 == bugs_adjacent) {
                    new_state |= bit;
                }

                bit <<= 1;
            }
        }

        self.state = new_state;
    }

    fn to_string(&self) -> String {
        let mut s = String::new();
        let mut bit = 1;

        for _ in 0..self.dim {
            if s.len() > 0 {
                s += "\n";
            }
            for _ in 0..self.dim {
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
    const DIM: usize = 5;

    let mut seen = HashSet::new();

    let initial = parse_input_part1(input);
    let mut bugs = Bugs::new(initial, DIM);

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
    fn part1_example() {
        let input = "....#\n\
                    #..#.\n\
                    #..##\n\
                    ..#..\n\
                    #....";

        let mut bugs = Bugs::new(parse_input_part1(input), 5);
        assert_eq!(bugs.to_string(), "\
            ....#\n\
            #..#.\n\
            #..##\n\
            ..#..\n\
            #....");
        bugs.advance();
        assert_eq!(bugs.to_string(), "\
            #..#.\n\
            ####.\n\
            ###.#\n\
            ##.##\n\
            .##..");
        bugs.advance();
        assert_eq!(bugs.to_string(), "\
            #####\n\
            ....#\n\
            ....#\n\
            ...#.\n\
            #.###");
        bugs.advance();
        assert_eq!(bugs.to_string(), "\
            #....\n\
            ####.\n\
            ...##\n\
            #.##.\n\
            .##.#");
        bugs.advance();
        assert_eq!(bugs.to_string(), "\
            ####.\n\
            ....#\n\
            ##..#\n\
            .....\n\
            ##...");
        
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
