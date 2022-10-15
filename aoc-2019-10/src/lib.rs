// -*- compile-command: "cargo test -- --show-output" -*-

type AdventResult = usize;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;

use num_rational::Ratio;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Slope(isize, isize);

const UPPER_RIGHT: u8 = 1;
const LOWER_RIGHT: u8 = 2;
const LOWER_LEFT: u8 = 3;
const UPPER_LEFT: u8 = 4;

impl Slope {
    fn on_axis(&self) -> bool {
        let Slope(dx, dy) = self;
        *dx == 0 || *dy == 0
    }

    fn quadrant(&self) -> u8 {
        // Laser start pointing up and then rotates clockwise. We want
        // to order by quadrant and then largest-first dx/dy ratios
        // within each quadrant to model the laser's sweep.
        //
        // Up     = (0, -1) 0
        // Quad 1 = (+, -)  dx/dy ratio increasing(ly negative = decreasing)
        // Right  = (1, 0)  inf
        // Quad 2 = (+, +)  dx/dy ratio decreasing(ly positive = decreasing)
        // Down   = (0, 1)  0
        // Quad 3 = (-, +)  dx/dy ratio increasing(ly negative = decreasing)
        // Left   = (-1, 0) inf
        // Quad 4 = (-, -)  dx/dy ratio decreasing(ly positive = decreasing)

        match *self {
            Slope(0, -1) => UPPER_RIGHT,
            Slope(pos, neg) if pos > 0 && neg < 0 => UPPER_RIGHT,
            Slope(1, 0) => LOWER_RIGHT,
            Slope(pos1, pos2) if pos1 > 0 && pos2 > 0 => LOWER_RIGHT,
            Slope(0, 1) => LOWER_LEFT,
            Slope(neg, pos) if neg < 0 && pos > 0 => LOWER_LEFT,
            Slope(-1, 0) => UPPER_LEFT,
            Slope(neg1, neg2) if neg1 < 0 && neg2 < 0 => UPPER_LEFT,
            _ => panic!("unexpected slope value {:?}", self),
        }
    }
}

impl PartialOrd for Slope {
    fn partial_cmp(&self, other: &Slope) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Slope {
    fn cmp(&self, other: &Slope) -> Ordering {
        let quadcmp = self.quadrant().cmp(&other.quadrant());
        if quadcmp != Ordering::Equal {
            return quadcmp;
        }

        let self_axis = self.on_axis();
        let other_axis = other.on_axis();
        if self_axis && other_axis {
            return Ordering::Equal;
        }
        if self_axis {
            return Ordering::Less;
        }
        if other_axis {
            return Ordering::Greater;
        }

        let self_ratio = Ratio::new(self.0, self.1);
        let other_ratio = Ratio::new(other.0, other.1);
        // reverse the comparison; larger ratios should order earlier
        other_ratio.cmp(&self_ratio)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Position {
    y: usize,
    x: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Position { x, y }
    }

    fn distance_to(&self, other: &Position) -> f64 {
        let x0 = self.x as f64;
        let y0 = self.y as f64;

        let x1 = other.x as f64;
        let y1 = other.y as f64;

        f64::sqrt((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0))
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
    laser_position: Option<Position>,
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

        AsteroidMap {
            asteroids,
            laser_position: None,
        }
    }

    fn build_slope_groups(&self, p: &Position) -> HashMap<Slope, VecDeque<Position>> {
        let mut slopes = HashMap::new();
        for a in &self.asteroids {
            if p == a {
                continue;
            }

            let slope = p.slope_to(a);
            let list = slopes.entry(slope).or_insert_with(|| VecDeque::new());
            list.push_back(*a);
            if list.len() > 1 {
                list.make_contiguous().sort_by(|a, b| {
                    p.distance_to(a)
                        .partial_cmp(&p.distance_to(b))
                        .expect("should have real distance")
                });
            }
        }
        slopes
    }

    /// Returns the number of asteroids directly visible from the
    /// laser position, which is the metric maximized for choosing
    /// where to place the laser.
    fn place_laser(&mut self) -> usize {
        let mut max = 0;
        let mut max_pos = None;

        for p in &self.asteroids {
            let slopes = self.build_slope_groups(p);

            let visible = slopes.len();
            if visible > max {
                max = visible;
                max_pos = Some(p);
            }
        }

        self.laser_position = Some(*max_pos.expect("should have found a laser position"));
        max
    }

    fn find_nth_destroyed(&self, n: usize) -> Position {
        let mut destroyed = 0;
        let mut slopes = self.build_slope_groups(
            &self
                .laser_position
                .expect("must place laser before simulating destruction"),
        );

        while slopes.len() > 0 {
            let keys = {
                let mut keys: Vec<_> = slopes.keys().map(|k| *k).collect();
                keys.sort();
                keys
            };

            for slope in keys {
                let targets = slopes.get_mut(&slope).unwrap();

                let target = targets.pop_front().unwrap();
                destroyed += 1;
                if n == destroyed {
                    return target;
                }

                if 0 == targets.len() {
                    slopes.remove(&slope);
                }
            }
        }

        panic!("expected to destroy {} asteroids", n);
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> AdventResult {
    let mut asteroids = AsteroidMap::new(input().lines());
    asteroids.place_laser()
}

pub fn part2() -> AdventResult {
    let mut asteroids = AsteroidMap::new(input().lines());
    asteroids.place_laser();
    let Position { x, y } = asteroids.find_nth_destroyed(200);
    x * 100 + y
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_example1() {
        let input = "\
            .#..#
            .....
            #####
            ....#
            ...##";
        let mut asteroids = AsteroidMap::new(input.lines());
        assert_eq!(8, asteroids.place_laser());
    }

    #[test]
    fn part1_example2() {
        let input = "\
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
        let mut asteroids = AsteroidMap::new(input.lines());
        assert_eq!(33, asteroids.place_laser());
    }

    #[test]
    fn part1_example3() {
        let input = "\
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
        let mut asteroids = AsteroidMap::new(input.lines());
        assert_eq!(35, asteroids.place_laser());
    }

    #[test]
    fn part1_example4() {
        let input = "\
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
        let mut asteroids = AsteroidMap::new(input.lines());
        assert_eq!(41, asteroids.place_laser());
    }

    #[test]
    fn part1_example5() {
        let input = "\
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
        let mut asteroids = AsteroidMap::new(input.lines());
        assert_eq!(210, asteroids.place_laser());
    }

    #[test]
    fn part2_example_small() {
        let input = "\
            .#....#####...#..
            ##...##.#####..##
            ##...#...#.#####.
            ..#.....X...###..
            ..#.#.....#....##";

        let mut asteroids = AsteroidMap::new(input.lines());
        asteroids.laser_position = Some(Position { x: 8, y: 3 });

        let p = asteroids.find_nth_destroyed(1);
        assert_eq!(p, Position { x: 8, y: 1 });

        let p = asteroids.find_nth_destroyed(2);
        assert_eq!(p, Position { x: 9, y: 0 });

        let p = asteroids.find_nth_destroyed(3);
        assert_eq!(p, Position { x: 9, y: 1 });
    }

    #[test]
    fn part2_example_large() {
        let input = "\
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
        let mut asteroids = AsteroidMap::new(input.lines());
        asteroids.place_laser();
        let p = asteroids.find_nth_destroyed(200);
        assert_eq!(Position { x: 8, y: 2 }, p);
    }

    #[test]
    fn part1_solution() {
        assert_eq!(260, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(608, part2());
    }
}
