#![allow(dead_code, unused_variables)]

use std::collections::HashSet;
use std::fs;

const RECURSIVE_STEPS: usize = 200;
const DIM: usize = 5;
const DIM2: usize = DIM * DIM;

const RECURSIVE_INDEX: usize = 12;

const SINGLE_MASK: u32 = 0b1111111111111111111111111;

// Counting from 1 because that's how the pictures in the instructions are labeled.
const ADJACENCY_MASKS: [u128; DIM2] = [
    //            PARENT                           SELF                         CHILD
    //|-----------N - 1-----------|  |-------------N-------------|  |-----------N + 1-----------|
    //2......1...........0.........  2......1...........0.........  2......1...........0.........
    //54321_09876_54321_09876_54321__54321_09876_54321_09876_54321__54321_09876_54321_09876_54321
    0b00000_00000_00010_00100_00000__00000_00000_00000_00001_00010__00000_00000_00000_00000_00000, // 01
    0b00000_00000_00000_00100_00000__00000_00000_00000_00010_00101__00000_00000_00000_00000_00000, // 02
    0b00000_00000_00000_00100_00000__00000_00000_00000_00100_01010__00000_00000_00000_00000_00000, // 03
    0b00000_00000_00000_00100_00000__00000_00000_00000_01000_10100__00000_00000_00000_00000_00000, // 04
    0b00000_00000_01000_00100_00000__00000_00000_00000_10000_01000__00000_00000_00000_00000_00000, // 05
    0b00000_00000_00010_00000_00000__00000_00000_00001_00010_00001__00000_00000_00000_00000_00000, // 06
    0b00000_00000_00000_00000_00000__00000_00000_00010_00101_00010__00000_00000_00000_00000_00000, // 07
    0b00000_00000_00000_00000_00000__00000_00000_00100_01010_00100__00000_00000_00000_00000_11111, // 08
    0b00000_00000_00000_00000_00000__00000_00000_01000_10100_01000__00000_00000_00000_00000_00000, // 09
    0b00000_00000_01000_00000_00000__00000_00000_10000_01000_10000__00000_00000_00000_00000_00000, // 10
    0b00000_00000_00010_00000_00000__00000_00001_00010_00001_00000__00000_00000_00000_00000_00000, // 11
    0b00000_00000_00000_00000_00000__00000_00010_00101_00010_00000__00001_00001_00001_00001_00001, // 12
    0b00000_00000_00000_00000_00000__00000_00100_01010_00100_00000__00000_00000_00000_00000_00000, // 13 -- contains CHILD
    0b00000_00000_00000_00000_00000__00000_01000_10100_01000_00000__10000_10000_10000_10000_10000, // 14
    0b00000_00000_01000_00000_00000__00000_10000_01000_10000_00000__00000_00000_00000_00000_00000, // 15
    0b00000_00000_00010_00000_00000__00001_00010_00001_00000_00000__00000_00000_00000_00000_00000, // 16
    0b00000_00000_00000_00000_00000__00010_00101_00010_00000_00000__00000_00000_00000_00000_00000, // 17
    0b00000_00000_00000_00000_00000__00100_01010_00100_00000_00000__11111_00000_00000_00000_00000, // 18
    0b00000_00000_00000_00000_00000__01000_10100_01000_00000_00000__00000_00000_00000_00000_00000, // 19
    0b00000_00000_01000_00000_00000__10000_01000_10000_00000_00000__00000_00000_00000_00000_00000, // 20
    0b00000_00100_00010_00000_00000__00010_00001_00000_00000_00000__00000_00000_00000_00000_00000, // 21
    0b00000_00100_00000_00000_00000__00101_00010_00000_00000_00000__00000_00000_00000_00000_00000, // 22
    0b00000_00100_00000_00000_00000__01010_00100_00000_00000_00000__00000_00000_00000_00000_00000, // 23
    0b00000_00100_00000_00000_00000__10100_01000_00000_00000_00000__00000_00000_00000_00000_00000, // 24
    0b00000_00100_01000_00000_00000__01000_10000_00000_00000_00000__00000_00000_00000_00000_00000, // 25
];

// Used this to compute the "self" adjacency masks. Then I just
// manually marked the 20 parent and 20 child adjacencies.
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

const STATE_LEN: usize = 2 * RECURSIVE_STEPS + 1;

struct RecursiveBugs {
    state: [[BugState; STATE_LEN]; 2],
    current: usize,
}

impl RecursiveBugs {
    fn new(initial: BugState) -> Self {
        let mut bugs = RecursiveBugs {
            state: [[0; 2 * RECURSIVE_STEPS + 1]; 2],
            current: 0,
        };
        bugs.state[0][0] = initial;
        bugs
    }

    fn advance(&mut self) {
        let mut level = RECURSIVE_STEPS + 1;
        let cur = self.current;
        let new = (self.current + 1) % 2;
        let mut state = 0u128 | self.state[cur][level] as u128;

        for _ in 0..STATE_LEN {
            let next_level = (level + 1) % STATE_LEN;
            state <<= DIM2;
            state |= self.state[cur][next_level] as u128;

            let new_state = &mut self.state[new][level];
            *new_state = 0;

            for i in 0..DIM2 {
                if i == RECURSIVE_INDEX {
                    continue;
                }

                let has_bug = 0 != state & (1 << (DIM2 + i));

                let mask = ADJACENCY_MASKS[i];
                let masked_state = state & mask;
                let bugs_adjacent = masked_state.count_ones();

                let bit = 1 << i;
                if (has_bug && 1 == bugs_adjacent)
                    || (!has_bug && (1 == bugs_adjacent || 2 == bugs_adjacent))
                {
                    *new_state |= bit;
                }
            }

            level = next_level;
        }

        self.current = new;
    }

    fn bug_count(&self) -> u32 {
        let mut n = 0;
        let state = &self.state[self.current];
        for i in 0..STATE_LEN {
            n += state[i].count_ones();
        }
        n
    }

    fn to_string(&self, level: i32) -> String {
        let level = level.rem_euclid(STATE_LEN as i32) as usize;

        let state = &self.state[self.current][level];

        let mut s = String::new();
        let mut bit = 1;

        for r in 0..DIM {
            if s.len() > 0 {
                s += "\n";
            }
            for c in 0..DIM {
                s.push(if r * DIM + c == RECURSIVE_INDEX {
                    '?'
                } else if 0 == *state & bit {
                    '.'
                } else {
                    '#'
                });
                bit <<= 1;
            }
        }

        s
    }
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

        for i in 0..DIM2 {
            let mask = (ADJACENCY_MASKS[i] >> 25) as u32 & SINGLE_MASK;

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

        for r in 0..DIM {
            if s.len() > 0 {
                s += "\n";
            }
            for c in 0..DIM {
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

fn parse_input(input: &str) -> BugState {
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

    let initial = parse_input(input);
    let mut bugs = Bugs::new(initial);

    while !seen.contains(&bugs.state) {
        seen.insert(bugs.state);
        bugs.advance();
    }

    bugs.state
}

fn do_part2(input: &str) -> AdventResult {
    let mut bugs = RecursiveBugs::new(parse_input(input));
    for _ in 0..RECURSIVE_STEPS {
        bugs.advance();
    }
    bugs.bug_count()
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

        let mut bugs = Bugs::new(parse_input(input));
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
        let input = "....#\n\
                    #..#.\n\
                    #..##\n\
                    ..#..\n\
                    #....";

        let mut bugs = RecursiveBugs::new(parse_input(input));
        assert_eq!(
            bugs.to_string(0),
            "\
            ....#\n\
            #..#.\n\
            #.?##\n\
            ..#..\n\
            #...."
        );
        for _ in 0..10 {
            bugs.advance();
        }
        assert_eq!(
            bugs.to_string(0),
            "\
            .#...\n\
            .#.##\n\
            .#?..\n\
            .....\n\
            ....."
        );
        assert_eq!(
            bugs.to_string(5),
            "\
            ####.\n\
            #..#.\n\
            #.?#.\n\
            ####.\n\
            ....."
        );
        assert_eq!(
            bugs.to_string(-5),
            "\
            ..#..\n\
            .#.#.\n\
            ..?.#\n\
            .#.#.\n\
            ..#.."
        );

        assert_eq!(99, bugs.bug_count());
    }

    #[test]
    fn part1_solution() {
        assert_eq!(3186366, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(2031, part2());
    }
}
