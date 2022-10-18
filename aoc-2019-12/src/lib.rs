// -*- compile-command: "cargo test -- --show-output" -*-

#![allow(dead_code)]

type AdventResult = i64;

use std::fs;

use num::Integer;
use regex::Regex;

#[derive(Debug, Clone)]
struct Body {
    position: [i64; 3],
    velocity: [i64; 3],
}

impl Body {
    fn new(position: [i64; 3]) -> Self {
        Body {
            position,
            velocity: [0; 3],
        }
    }

    fn apply_gravity(&mut self, other_pos: [i64; 3]) {
        for i in 0..3 {
            if self.position[i] > other_pos[i] {
                self.velocity[i] -= 1;
            } else if self.position[i] < other_pos[i] {
                self.velocity[i] += 1;
            }
        }
    }

    fn apply_velocity(&mut self) {
        for i in 0..3 {
            self.position[i] += self.velocity[i];
        }
    }

    fn kinetic_energy(&self) -> i64 {
        self.velocity.iter().map(|n| n.abs()).sum()
    }

    fn potential_energy(&self) -> i64 {
        self.position.iter().map(|n| n.abs()).sum()
    }

    fn total_energy(&self) -> i64 {
        self.kinetic_energy() * self.potential_energy()
    }
}

struct BodySystem {
    step_number: i64,
    bodies: Vec<Body>,
    initial_state: Vec<Body>,
}

impl BodySystem {
    fn new(bodies: Vec<[i64; 3]>) -> Self {
        let bodies: Vec<_> = bodies.into_iter().map(|a| Body::new(a)).collect();
        let initial_state = bodies.clone();

        BodySystem {
            bodies,
            initial_state,
            step_number: 0,
        }
    }

    fn apply_gravity(&mut self, ai: usize, bi: usize) {
        let bpos = self.bodies.get(bi).unwrap().position;
        self.bodies.get_mut(ai).unwrap().apply_gravity(bpos);
    }

    fn step_until_components_cycle(&mut self) -> [i64; 3] {
        assert_eq!(0, self.step_number);

        let mut cycles = [0; 3];

        while cycles.iter().any(|n| *n == 0) {
            self.step();

            for d in 0..3 {
                if cycles[d] != 0 {
                    continue;
                }

                if self
                    .bodies
                    .iter()
                    .zip(self.initial_state.iter())
                    .all(|(a, b)| a.position[d] == b.position[d] && a.velocity[d] == b.velocity[d])
                {
                    cycles[d] = self.step_number;
                }
            }
        }

        cycles
    }

    fn steps_until_cycle(&mut self) -> i64 {
        let cycles = self.step_until_components_cycle();

        let mut lcm: i64 = 1;
        for i in 0..3 {
            lcm = lcm.lcm(&cycles[i]);
        }
        lcm
    }

    fn step(&mut self) {
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                self.apply_gravity(i, j);
                self.apply_gravity(j, i);
            }
        }

        for b in &mut self.bodies {
            b.apply_velocity();
        }

        self.step_number += 1;
    }

    fn total_energy(&self) -> i64 {
        self.bodies.iter().map(|b| b.total_energy()).sum()
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn parse_input(input: &str) -> Vec<[i64; 3]> {
    let mut v = vec![];

    let re = Regex::new(r"\A<x=(.*?), y=(.*?), z=(.*?)>\z").unwrap();

    for line in input.trim().lines() {
        let cap = re.captures(line).unwrap();
        v.push([
            cap[1].parse().unwrap(),
            cap[2].parse().unwrap(),
            cap[3].parse().unwrap(),
        ]);
    }

    v
}

pub fn part1() -> AdventResult {
    let mut system = BodySystem::new(parse_input(&input()));
    for _ in 0..1000 {
        system.step();
    }
    system.total_energy()
}

pub fn part2() -> AdventResult {
    let mut system = BodySystem::new(parse_input(&input()));
    system.steps_until_cycle()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_input() {
        let input = "\
<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>";
        assert_eq!(
            vec![[-1, 0, 2], [2, -10, -7], [4, -8, 8], [3, 5, -1],],
            parse_input(input)
        );
    }

    #[test]
    fn part1_example1() {
        let input = "\
<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>";
        let mut system = BodySystem::new(parse_input(input));
        system.step();
        let expected = "[\
Body { position: [2, -1, 1], velocity: [3, -1, -1] }, \
Body { position: [3, -7, -4], velocity: [1, 3, 3] }, \
Body { position: [1, -7, 5], velocity: [-3, 1, -3] }, \
Body { position: [2, 2, 0], velocity: [-1, -3, 1] }]";
        assert_eq!(expected, format!("{:?}", system.bodies));

        for _ in 0..9 {
            system.step();
        }
        let expected = "[\
Body { position: [2, 1, -3], velocity: [-3, -2, 1] }, \
Body { position: [1, -8, 0], velocity: [-1, 1, 3] }, \
Body { position: [3, -6, 1], velocity: [3, 2, -3] }, \
Body { position: [2, 0, 4], velocity: [1, -1, -1] }]";
        assert_eq!(expected, format!("{:?}", system.bodies));

        assert_eq!(179, system.total_energy());
    }

    #[test]
    fn part1_example2() {
        let input = "\
<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>";
        let mut system = BodySystem::new(parse_input(input));
        for _ in 0..10 {
            system.step();
        }
        let expected = "[\
Body { position: [-9, -10, 1], velocity: [-2, -2, -1] }, \
Body { position: [4, 10, 9], velocity: [-3, 7, -2] }, \
Body { position: [8, -10, -3], velocity: [5, -1, -2] }, \
Body { position: [5, -10, 3], velocity: [0, -4, 5] }]";
        assert_eq!(expected, format!("{:?}", system.bodies));

        for _ in 0..90 {
            system.step();
        }
        let expected = "[\
Body { position: [8, -12, -9], velocity: [-7, 3, 0] }, \
Body { position: [13, 16, -3], velocity: [3, -11, -5] }, \
Body { position: [-29, -11, -1], velocity: [-3, 7, 4] }, \
Body { position: [16, -13, 23], velocity: [7, 1, 1] }]";
        assert_eq!(expected, format!("{:?}", system.bodies));

        assert_eq!(1940, system.total_energy());
    }

    #[test]
    fn part2_example1() {
        let input = "\
<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>";
        let mut system = BodySystem::new(parse_input(input));
        assert_eq!(2772, system.steps_until_cycle());
    }

    #[test]
    fn part2_example2() {
        let input = "\
<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>";
        let mut system = BodySystem::new(parse_input(input));
        assert_eq!(4686774924, system.steps_until_cycle());
    }

    #[test]
    fn part1_solution() {
        assert_eq!(7202, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(537881600740876, part2());
    }
}
