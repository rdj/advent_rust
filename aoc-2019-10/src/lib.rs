// -*- compile-command: "cargo test -- --show-output" -*-

type AdventResult = usize;

use std::collections::HashSet;
use std::fs;

use num_rational::Ratio;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Slope(isize, isize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    y: usize,
    x: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Position { x, y }
    }

    fn slope_to(&self, other: &Position) -> Slope {
        let dx = other.x as isize - self.x as isize;
        let dy = other.y as isize - self.y as isize;

        let dxsign = if dx < 0 { -1 } else { 1 };
        let dysign = if dy < 0 { -1 } else { 1 };

        assert!(dx != 0 || dy != 0);

        if dx == 0 {
            Slope(0, dysign)
        } else if dy == 0 {
            Slope(dxsign, 0)
        } else {
            let ratio = Ratio::new(dx.abs(), dy.abs());
            Slope(dxsign * *ratio.numer(), dysign * *ratio.denom())
        }
    }
}

struct AsteroidMap {
    asteroids: Vec<Position>,
}

impl AsteroidMap {
    fn new<'a, I>(lines: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut asteroids = vec![];

        for (y, line) in lines.into_iter().enumerate() {
            for (x, c) in line.trim().chars().enumerate() {
                if c == '#' {
                    asteroids.push(Position::new(x, y));
                }
            }
        }

        AsteroidMap { asteroids }
    }

    fn find_max_detection(&self) -> usize {
        let mut max = 0;

        for p in &self.asteroids {
            let mut slopes = HashSet::new();
            for a in &self.asteroids {
                if p == a {
                    continue;
                }

                let slope = p.slope_to(a);
                // println!(
                //     "({}, {}) -> ({}, {}) = [{}, {}]",
                //     p.x, p.y, a.x, a.y, slope.0, slope.1
                // );
                slopes.insert(slope);
            }

            let visible = slopes.len();
            if visible > max {
                max = visible;
            }

            // println!("({}, {}) = {}", p.x, p.y, visible);
        }

        max
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> AdventResult {
    let asteroids = AsteroidMap::new(input().lines());
    asteroids.find_max_detection()
}

pub fn part2() -> AdventResult {
    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_example1() {
        let input = "
            .#..#
            .....
            #####
            ....#
            ...##";
        let asteroids = AsteroidMap::new(input.lines());
        assert_eq!(8, asteroids.find_max_detection());
    }

    #[test]
    fn part1_example2() {
        let input = "
            ......#.#.
            #..#.#....
            ..#######.
            .#.#.###..
            .#..#.....
            ..#....#.#
            #..#....#.
            .##.#..###
            ##...#..#.
            .#....####";
        let asteroids = AsteroidMap::new(input.lines());
        assert_eq!(33, asteroids.find_max_detection());
    }

    #[test]
    fn part1_example3() {
        let input = "
            #.#...#.#.
            .###....#.
            .#....#...
            ##.#.#.#.#
            ....#.#.#.
            .##..###.#
            ..#...##..
            ..##....##
            ......#...
            .####.###.";
        let asteroids = AsteroidMap::new(input.lines());
        assert_eq!(35, asteroids.find_max_detection());
    }

    #[test]
    fn part1_example4() {
        let input = "
            .#..#..###
            ####.###.#
            ....###.#.
            ..###.##.#
            ##.##.#.#.
            ....###..#
            ..#.#..#.#
            #..#.#.###
            .##...##.#
            .....#.#..";
        let asteroids = AsteroidMap::new(input.lines());
        assert_eq!(41, asteroids.find_max_detection());
    }
    
    #[test]
    fn part1_example5() {
        let input = "
            .#..##.###...#######
            ##.############..##.
            .#.######.########.#
            .###.#######.####.#.
            #####.##.#.##.###.##
            ..#####..#.#########
            ####################
            #.####....###.#.#.##
            ##.#################
            #####.##.###..####..
            ..######..##.#######
            ####.##.####...##..#
            .#####..#.######.###
            ##...#.##########...
            #.##########.#######
            .####.#.###.###.#.##
            ....##.##.###..#####
            .#.#.###########.###
            #.#.#.#####.####.###
            ###.##.####.##.#..##";
        let asteroids = AsteroidMap::new(input.lines());
        assert_eq!(210, asteroids.find_max_detection());
    }
    
    #[test]
    fn part2_example() {
        panic!("part 2 example");
    }

    #[test]
    fn part1_solution() {
        assert_eq!(260, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
