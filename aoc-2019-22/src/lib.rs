#![allow(dead_code, unused_variables)]

use num::Integer;

type AdventResult = i64;

const PART1_DECK_SIZE: i64 = 10_007;
const EX1_DECK_SIZE: i64 = 10;

const PART2_DECK_SIZE: i64 = 119_315_717_514_047;
const PART2_ITERATIONS: i64 = 101_741_582_076_661;

// Extended Euclidean algorithm for computing GCD also yields the
// Bézout coefficients which for our purposes can be used to calculate
// the multiplicative inverse of x where ax = 1 (mod b) if a and b are
// coprime.
//
// https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
fn multiplicative_inverse_mod(n: i64, modulus: i64) -> i64 {
    i64::extended_gcd(&n, &modulus).x.rem_euclid(modulus)
}

#[derive(Clone)]
struct Deck {
    offset: i64,
    size: i64,
    step: i64,
}

impl Deck {
    fn new(size: i64) -> Self {
        assert!(size > 0);
        Deck {
            size,
            offset: 0,
            step: 1,
        }
    }

    fn to_string(&self) -> String {
        format!("{}*x + {}", self.step, self.offset)
    }

    fn cut(&mut self, n: i64) {
        self.offset = (self.offset + n * self.step).rem_euclid(self.size);
    }

    fn deal_increment(&mut self, x: i64) {
        //
        // Before [0 1 2 3 4 5 6 7 8 9]
        //  Inc 7 [0 3 6 9 2 5 8 1 4 7]
        //
        // The offset will be unchanged.
        //
        // The new step size can be determined by finding the value
        // that lands in [1] after the change. 
        //
        // The find the current index of that value, we must determine
        // n where (x * n) % 10 = 1
        //
        // This is the definition of the multiplicative inverse (mod
        // 10) of x.
        //
        let inverse = multiplicative_inverse_mod(x, self.size);
        self.step = (self.nth(inverse) - self.offset).rem_euclid(self.size);
    }

    fn deal_new(&mut self) {
        self.step = (-self.step).rem_euclid(self.size);
        self.cut(1);
    }

    fn nth(&self, n: i64) -> i64 {
        (self.offset + n * self.step).rem_euclid(self.size)
    }

    fn run_program(&mut self, input: &str) {
        for line in input.lines() {
            let mut words = line.split_whitespace();
            match words.next() {
                Some("cut") => self.cut(words.next().unwrap().parse().unwrap()),
                Some("deal") => match words.next() {
                    Some("into") => self.deal_new(),
                    Some("with") => self.deal_increment(words.last().unwrap().parse().unwrap()),
                    x => panic!("don't know how to deal {:?}", x),
                },
                x => panic!("don't recognize command {:?}", x),
            }
        }
    }

    fn to_vec(&self) -> Vec<i64> {
        let mut vec = vec![];
        for n in 0..self.size {
            vec.push(self.nth(n));
        }
        vec
    }
}

use std::fs;

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    let mut deck = Deck::new(PART1_DECK_SIZE);
    deck.run_program(input);
    
    // what is the position of card 2019
    let mut n = 0;
    while deck.nth(n) != 2019 {
        n += 1;
    }
    n
}

fn do_part2(input: &str) -> AdventResult {
    let mut deck = Deck::new(PART2_DECK_SIZE);
    deck.run_program(input);
    println!("{}", deck.to_string());
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
    fn test_initial() {
        let deck = Deck::new(EX1_DECK_SIZE);
        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], deck.to_vec());
    }

    #[test]
    fn test_deal_new() {
        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program("deal into new stack");
        assert_eq!(vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0], deck.to_vec());
    }

    #[test]
    fn test_deal_new_then_increment() {
        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(
            "deal into new stack\n\
                          deal with increment 7",
        );
        assert_eq!(vec![9, 6, 3, 0, 7, 4, 1, 8, 5, 2], deck.to_vec());
    }

    #[test]
    fn test_increment_then_deal_new() {
        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(
            "deal with increment 7\n\
                          deal into new stack",
        );
        assert_eq!(vec![7, 4, 1, 8, 5, 2, 9, 6, 3, 0,], deck.to_vec());
    }

    #[test]
    fn test_increment() {
        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program("deal with increment 7");
        assert_eq!(vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7], deck.to_vec());
    }

    #[test]
    fn test_increment_then_cut() {
        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(
            "deal with increment 3\n\
                          cut 2",
        );
        assert_eq!(vec![4, 1, 8, 5, 2, 9, 6, 3, 0, 7], deck.to_vec());
    }

    #[test]
    fn test_cut_then_increment() {
        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(
            "cut 2\n\
                          deal with increment 3",
        );
        assert_eq!(vec![2, 9, 6, 3, 0, 7, 4, 1, 8, 5], deck.to_vec());
    }

    #[test]
    fn test_cut() {
        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program("cut 3");
        assert_eq!(vec![3, 4, 5, 6, 7, 8, 9, 0, 1, 2], deck.to_vec());
    }

    #[test]
    fn part1_example1() {
        let input = "\
deal with increment 7
deal into new stack
deal into new stack
";

        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(input);
        assert_eq!(vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7], deck.to_vec());
    }

    #[test]
    fn part1_example2() {
        let input = "\
cut 6
deal with increment 7
deal into new stack
";

        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(input);
        assert_eq!(vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6], deck.to_vec());
    }

    #[test]
    fn part1_example3() {
        let input = "\
deal with increment 7
deal with increment 9
cut -2
";

        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(input);
        assert_eq!(vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9], deck.to_vec());
    }

    #[test]
    fn part1_example4() {
        let input = "\
deal into new stack
cut -2
deal with increment 7
cut 8
cut -4
deal with increment 7
cut 3
deal with increment 9
deal with increment 3
cut -1
";

        let mut deck = Deck::new(EX1_DECK_SIZE);
        deck.run_program(input);
        assert_eq!(vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6], deck.to_vec());
    }

    #[test]
    fn part1_solution() {
        assert_eq!(1538, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
