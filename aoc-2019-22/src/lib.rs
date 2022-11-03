#![allow(dead_code, unused_variables)]

use std::fs;
use num::Integer; // extended_gcd

type AdventResult = i128;

const PART1_DECK_SIZE: i128 = 10_007;
const EX1_DECK_SIZE: i128 = 10;

const PART2_DECK_SIZE: i128 = 119_315_717_514_047;
const PART2_ITERATIONS: i128 = 101_741_582_076_661;

// Using rem_euclid throughout in order to get mathematically proper
// modulus results for negative numbers, unlike rust's % operator.

// Extended Euclidean algorithm for computing GCD also yields the
// Bézout coefficients which for our purposes can be used to calculate
// the multiplicative inverse of x where ax = 1 (mod b) if a and b are
// coprime.
//
// https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
fn multiplicative_inverse_mod(n: i128, modulus: i128) -> i128 {
    i128::extended_gcd(&n, &modulus).x.rem_euclid(modulus)
}

// Clever algorithm exploits the fact the number is stored in base-2
// already, so you can distribute the exponentiation and mod each step
// to keep things from overflowing.
//
// https://en.wikipedia.org/wiki/Modular_exponentiation#Right-to-left_binary_method
fn pow_mod(base: i128, mut exponent: i128, modulus: i128) -> i128 {
    assert!(base >= 0);
    assert!(exponent >= 0);
    assert!(modulus > 1);
    
    let mut result  = 1;
    let mut base = base.rem_euclid(modulus);
    while exponent > 0 {
        if exponent.rem_euclid(2) == 1 {
            result = (result * base).rem_euclid(modulus);
        }
        exponent >>= 1;
        base = (base * base).rem_euclid(modulus);
    }

    result
}

struct Deck {
    offset: i128,
    size: i128,
    step: i128,
}

impl Deck {
    fn new(size: i128) -> Self {
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

    fn cut(&mut self, n: i128) {
        //                       v
        // Before [0 1 2 3 4 5 6 7 8 9]
        //  Cut 7 [8 9 0 1 2 3 4 5 6 7]
        //                           ^
        // This same operation could be called Cut -3.
        //
        // Advances the series by n steps, so:
        //   offset += n * step (mod size)

        self.offset = (self.offset + n * self.step).rem_euclid(self.size);
    }

    fn deal_increment(&mut self, x: i128) {
        //           v
        // Before [0 1 2 3 4 5 6 7 8 9]
        //  Inc 7 [0 3 6 9 2 5 8 1 4 7]
        //                       ^
        // Leaves [0] as a fixed point and moves the value currently
        // at [1] to [x], at [2] to [2*x % size], etc. 
        //
        // The new step size can be determined by finding the value
        // that lands in [1] after the change. 
        //
        // To figure out the current index, find n where
        //   n * x = 1 (mod size)
        //
        // This is the definition of the multiplicative inverse in the
        // group.
        //
        // Note in the example the value "3" is found at index [3],
        // which is the multiplicative inverse of 7, i.e.
        //   7 * 3 = 1 (mod 10).
        //
        // Once we know [0] and [1], we can compute the new step:
        //   step = nth(1/x) - nth(0)
        //   step = step * 1/x + offset - (step * 0 + offset) (mod size)
        //   step = step * 1/x (mod size)
        //
        // All that said, maybe it's more intuitive just to think of
        // it as division.
        
        let x_inv = multiplicative_inverse_mod(x, self.size);
        self.step = (self.step * x_inv).rem_euclid(self.size);
    }

    fn deal_new(&mut self) {
        //
        // Before [0 1 2 3 4 5 6 7 8 9]
        //    New [9 8 7 6 5 4 3 2 1 0]
        //
        // Reverses the list, including shifting the old [0] to the
        // end, so:
        //    step *= -1 (mod size)
        //  offset += step
        //
        self.step = (-self.step).rem_euclid(self.size);
        self.cut(1);
    }

    fn iterate(&mut self, n: i128) {
        //
        // To iterate, compose the a*x + b transform repeatedly:
        //
        // y_1 = a*x + b
        // y_2 = a*y_1 + b
        //     = a*(a*x + b) + b
        //     = a^2*x + ab + b
        //     = a^2*x + (a + 1)*b
        // y_3 = a*y_2 + b
        //     = a*(a^2*x + ab + b) + b
        //     = a^3*x + a^2*b + ab + b
        //     = a^3*x + (a^2 + a + 1)*b
        // y_n = a^n*x + (b + ba + ba^2 + ba^3 + ... + ba^(n-1))
        //
        // step size `a` can be found by exponentiation
        //   a_n = a^n
        //
        // offset `b` looks like the partial sum of a geometric series
        //   b_n = b + ba + ba^2 + ba^3 + ... + ba^(n-1)
        //       = b * (1 - a^n) / (1 - a)
        // 
        // Of course, division isn't defined in our group so we need
        // to use the multiplicative inverse which we helpfully used
        // in part 1 as well.
        //   b_n = b * (1 - a^n) * 1 / (1 - a)
        //
        // Carefully applying mod after each step to hopefully avoid
        // overflow (and importing a bigint module).
        
        let new_step = pow_mod(self.step, n, self.size);
        
        let num = (1 - new_step).rem_euclid(self.size);
        let den = (1 - self.step).rem_euclid(self.size);
        let den_inv = multiplicative_inverse_mod(den, self.size);
        let ratio = (num * den_inv).rem_euclid(self.size);
        
        let new_offset = (self.offset * ratio).rem_euclid(self.size);

        self.step = new_step;
        self.offset = new_offset;
    }

    fn nth(&self, n: i128) -> i128 {
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

    fn to_vec(&self) -> Vec<i128> {
        assert!(self.size < 100);
        let mut vec = vec![];
        for n in 0..self.size {
            vec.push(self.nth(n));
        }
        vec
    }
}

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
    deck.iterate(PART2_ITERATIONS);

    // what card is in position 2020
    deck.nth(2020)
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
        assert_eq!(96_196_710_942_473, part2());
    }

    #[test]
    fn test_pow_mod() {
        assert_eq!(445, pow_mod(4, 13, 497));
        assert_eq!(4, pow_mod(2, 50, 13));
        assert_eq!(12, pow_mod(2, 90, 13));
    }
}
