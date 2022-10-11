// -*- compile-command: "cargo test -- --show-output" -*-

use std::collections::HashMap;
use std::fs;

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> usize {
    let input = input();
    let orbits = Orbits::new(input.lines());
    orbits.total_direct_and_indirect_orbits()
}

pub fn part2() -> usize {
    let input = input();
    let orbits = Orbits::new(input.lines());
    orbits.path_length_between("YOU", "SAN")
}

type OrbitalMap = HashMap<String, String>;

struct Orbits {
    orbital_map: OrbitalMap,
}

const CENTER_OF_MASS: &str = "COM";

impl Orbits {
    fn new<'a, I>(orbits: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut orbital_map: OrbitalMap = OrbitalMap::new();

        let iter = orbits.into_iter().map(|s| {
            let mut split = s.split(')');
            (
                split.next().expect("body"),
                split.next().expect("satellite"),
            )
        });

        for (body, sat) in iter {
            let existing = orbital_map.insert(sat.to_owned(), body.to_owned());
            assert_eq!(None, existing);
        }

        Orbits { orbital_map }
    }

    fn count_sat(&self, sat: &String) -> usize {
        if CENTER_OF_MASS == *sat {
            return 0;
        }

        1 + self.count_sat(self.orbital_map.get(sat).unwrap())
    }

    fn path_length_between(&self, a: &str, b: &str) -> usize {
        let apath = self.path_to_center(a);
        let bpath = self.path_to_center(b);

        for (i, abody) in apath.iter().enumerate() {
            if let Some((j, _)) = bpath.iter().enumerate().find(|(_, &bbody)| *abody == bbody) {
                return i + j;
            }
        }

        panic!("expected to find a common ancestor");
    }

    fn path_to_center<'a>(&'a self, sat: &'a str) -> Vec<&str> {
        let mut path = vec![];

        let mut next = sat;
        while next != CENTER_OF_MASS {
            next = self.orbital_map.get(next).unwrap();
            path.push(next);
        }

        path
    }

    fn total_direct_and_indirect_orbits(&self) -> usize {
        let mut total = 0;

        for sat in self.orbital_map.keys() {
            total += self.count_sat(sat);
        }

        total
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_ex() {
        let input = vec![
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L",
        ];

        let orbits = Orbits::new(input);

        assert_eq!(42, orbits.total_direct_and_indirect_orbits());
    }

    #[test]
    fn part2_ex() {
        let input = vec![
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L", "K)YOU",
            "I)SAN",
        ];

        let orbits = Orbits::new(input);

        assert_eq!(4, orbits.path_length_between("YOU", "SAN"));
    }

    #[test]
    fn run_part1() {
        assert_eq!(241064, part1());
    }

    #[test]
    fn run_part2() {
        assert_eq!(418, part2());
    }
}
