#![allow(dead_code, unused_variables)]

type AdventResult = String;

use std::fs;

struct Pattern {
    repeats: usize,
    pos: usize,
}

impl Pattern {
    fn new(repeats: usize) -> Self {
        assert!(repeats > 0);
        Pattern { repeats, pos: 0 }
    }
}

impl Iterator for Pattern {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;

        let n = match self.pos / self.repeats % 4 {
            0 => 0,
            1 => 1,
            2 => 0,
            3 => -1,
            _ => panic!("unpossible"),
        };

        Some(n)
    }
}

const RESULT_LENGTH: usize = 8;

struct Transform {
    data: Vec<i32>,
}

impl Transform {
    fn new(seq: Vec<i32>) -> Self {
        let data = seq.into_iter().collect();

        Self { data }
    }

    fn run(&mut self, phase_count: usize) {
        for _ in 0..phase_count {
            self.phase();
        }
    }

    // We can do this in-place, because terms never depend on earlier
    // terms. See the explanation for part 2 for more detail.
    fn phase(&mut self) {
        for i in 0..self.data.len() {
            let p = Pattern::new(i + 1);

            let o = self
                .data
                .iter()
                .zip(p)
                .map(|(n, p_i)| n * p_i)
                .sum::<i32>()
                .abs()
                % 10;

            let n = self.data.get_mut(i).unwrap();
            *n = o;
        }
    }

    fn result(&self) -> String {
        self.data
            .iter()
            .take(RESULT_LENGTH)
            .map(|&n| char::from_digit(n as u32, 10).expect(&format!("should be a digit: {n}")))
            .collect()
    }
}

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn parse_input(input: &str) -> Vec<i32> {
    input
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as i32)
        .collect()
}

fn do_part1(input: &str) -> AdventResult {
    let mut xform = Transform::new(parse_input(input));
    xform.run(100);
    xform.result()
}

fn do_part2(input: &str) -> AdventResult {
    // Part 2 does not directly build on part 1.
    //
    // In Part 2, the input is repeated 10_000 times, and the result
    // is read from the index given by the first seven digits of the
    // input.
    //
    // The sequence is obviously supposed to be too long to calculate
    // over using the method from part 1, which was already slow
    // noticeably sluggish even with the small input.
    //
    // So we must find a shortcut. And there's not a generalizable
    // shortcut. Spoiler: there is a shortcut for the specific input
    // input we're given.
    //
    // Here are the patterns of coefficients for the members of a
    // sequence with length 20.
    //
    //     00 +0-0+0-0+0-0+0-0+0-0
    //     01 0++00--00++00--00++0
    //     02 00+++000---000+++000
    //     03 000++++0000----0000+
    //     04 0000+++++00000-----0
    //     05 00000++++++000000---
    //     06 000000+++++++0000000
    //     07 0000000++++++++00000
    //     08 00000000+++++++++000
    //     09 000000000++++++++++0
    //     10 0000000000++++++++++
    //     11 00000000000+++++++++
    //     12 000000000000++++++++
    //     13 0000000000000+++++++
    //     14 00000000000000++++++
    //     15 000000000000000+++++
    //     16 0000000000000000++++
    //     17 00000000000000000+++
    //     18 000000000000000000++
    //     19 0000000000000000000+
    //
    // Early terms have much more complicated patterns than later
    // terms. Some observations:
    //
    //   1. The final term is just itself.
    //
    //   2. In fact, no calculation ever depends on earlier terms.
    //
    //   3. Terms 6-9 can be calculated with a single subsequence sum.
    //
    //          output[n] = sum(input[n..2*n])
    //
    //   4. At the halfway point, each output term is just the sum of
    //      all the subsequent input terms.
    //
    //          output[n] = sum(input[n..])
    //                    = input[n] + sum(input[n+1..])
    //                    = input[n] + output[n+1]
    //
    // It feels kind of cheap, but let's look at the input and see if
    // it is past the halfway point, then we have an easy solution.
    //
    //     Input length = 650
    //         x 10_000 = 6_500_000
    //        Leading 7 = 5_976_463
    //
    // Yeah, that's well past halfway. We only have to look at about
    // half a million terms or so.

    let offset: usize = input[0..7].parse().unwrap();

    let input_seq = parse_input(input);
    let input_len = input_seq.len();
    assert!(offset >= input_len / 2);

    let looped_len = 10_000 * input_len;
    let mut looped_seq: Vec<i32> = Vec::with_capacity(looped_len - offset);

    for i in offset..looped_len {
        looped_seq.push(*input_seq.get(i % input_len).unwrap());
    }

    for _ in 0..100 {
        let mut prev = 0;
        for i in (0..looped_seq.len()).rev() {
            let m = looped_seq.get_mut(i).unwrap();
            *m += prev;
            *m %= 10;
            prev = *m;
        }
    }

    looped_seq
        .iter()
        .take(RESULT_LENGTH)
        .map(|&n| char::from_digit(n as u32, 10).unwrap())
        .collect()
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
    fn test_pattern() {
        let p = Pattern::new(1);
        assert_eq!(
            p.take(10).collect::<Vec<_>>(),
            vec![1, 0, -1, 0, 1, 0, -1, 0, 1, 0]
        );

        let p = Pattern::new(2);
        assert_eq!(
            p.take(20).collect::<Vec<_>>(),
            vec![0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0]
        );

        let p = Pattern::new(3);
        assert_eq!(
            p.take(30).collect::<Vec<_>>(),
            vec![
                0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1, 0, 0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1, 0, 0, 0,
                1, 1, 1, 0
            ]
        );
    }

    #[test]
    fn test_phase() {
        let phases = ["12345678", "48226158", "34040438", "03415518", "01029498"];

        let mut xform = Transform::new(parse_input(phases[0]));

        for p in &phases[1..] {
            xform.run(1);
            assert_eq!(p, &xform.result());
        }
    }

    #[test]
    fn part1_example() {
        assert_eq!("24176176", &do_part1("80871224585914546619083218645595"));
        assert_eq!("73745418", &do_part1("19617804207202209144916044189917"));
        assert_eq!("52432133", &do_part1("69317163492948606335995924319873"));
    }

    #[test]
    fn part2_example() {
        assert_eq!(&do_part2("03036732577212944063491565474664"), "84462026");
        assert_eq!(&do_part2("02935109699940807407585447034323"), "78725270");
        assert_eq!(&do_part2("03081770884921959731165446850517"), "53553731");
    }

    #[test]
    fn part1_solution() {
        assert_eq!("63483758", &part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!("96099551", &part2());
    }
}
