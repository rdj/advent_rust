// -*- compile-command: "cargo test -- --show-output" -*-

use std::collections::HashMap;
use std::fs;

const PATH_COUNT: usize = 2;

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> usize {
    let input = input();
    let paths: Vec<&str> = input.trim().lines().take(PATH_COUNT).collect();
    let result = find_closest_intersection_distance(&paths);
    println!("part 1 = {result}");
    result
}

pub fn part2() -> usize {
    let input = input();
    let paths: Vec<&str> = input.trim().lines().take(PATH_COUNT).collect();
    let result = find_lowest_intersection_cost(&paths);
    println!("part 2 = {result}");
    result
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Point {
    x: isize,
    y: isize,
}

impl Point {
    fn distance_from_origin(&self) -> usize {
        (self.x.abs() + self.y.abs()) as usize
    }
}

#[derive(Debug)]
enum Move {
    Up(isize),
    Down(isize),
    Left(isize),
    Right(isize),
}

impl Move {
    fn destination(&self, from: &Point) -> Point {
        match self {
            Move::Up(y) => Point {
                y: from.y + y,
                ..*from
            },
            Move::Down(y) => Point {
                y: from.y - y,
                ..*from
            },
            Move::Left(x) => Point {
                x: from.x - x,
                ..*from
            },
            Move::Right(x) => Point {
                x: from.x + x,
                ..*from
            },
        }
    }

    fn step(&self, point: &mut Point) {
        match self {
            Move::Up(_) => point.y += 1,
            Move::Down(_) => point.y -= 1,
            Move::Left(_) => point.x -= 1,
            Move::Right(_) => point.x += 1,
        }
    }
}

fn parse_path(path: &str) -> Vec<Move> {
    path.split(',')
        .map(|s| {
            let mag = s[1..].parse().unwrap();
            match s.chars().next().unwrap() {
                'U' => Move::Up(mag),
                'D' => Move::Down(mag),
                'L' => Move::Left(mag),
                'R' => Move::Right(mag),
                x => panic!("Unknown move direction {x}"),
            }
        })
        .collect()
}

struct Location {
    visited: [bool; PATH_COUNT],
    costs: [usize; PATH_COUNT],
}

impl Location {
    fn new() -> Self {
        Location {
            visited: [false; PATH_COUNT],
            costs: [0; PATH_COUNT],
        }
    }

    fn is_intersection(&self) -> bool {
        self.visited.iter().all(|v| *v)
    }

    fn total_cost(&self) -> usize {
        self.costs.iter().sum()
    }

    fn visit(&mut self, path_index: usize, cost: usize) {
        self.visited[path_index] = true;
        if self.costs[path_index] == 0 {
            self.costs[path_index] = cost;
        }
    }
}

struct Layout(HashMap<Point, Location>);

impl Layout {
    fn new(paths: &Vec<&str>) -> Self {
        let mut points: HashMap<Point, Location> = HashMap::new();

        for (path_index, path) in paths.iter().enumerate() {
            let mut current = Point { x: 0, y: 0 };
            let mut cost = 0;
            // println!("Path {}", i);
            for m in parse_path(path) {
                let dest = m.destination(&current);
                while current != dest {
                    m.step(&mut current);
                    cost += 1;

                    let location = points
                        .entry(current.clone())
                        .or_insert_with(|| Location::new());
                    location.visit(path_index, cost);
                    // println!("{:?} => {:?} ({})", m, current, *point);
                }
            }
        }

        Layout(points)
    }

    fn find_intersections(&self) -> Vec<(&Point, &Location)> {
        self.0
            .iter()
            .filter(|(_, location)| location.is_intersection())
            .collect()
    }
}

fn find_closest_intersection_distance(paths: &Vec<&str>) -> usize {
    let layout = Layout::new(paths);

    let intersections = layout.find_intersections();

    let distances = intersections
        .into_iter()
        .map(|(p, _)| p.distance_from_origin());

    distances.min().expect("No intersection found")
}

fn find_lowest_intersection_cost(paths: &Vec<&str>) -> usize {
    let layout = Layout::new(&paths);

    let intersections = layout.find_intersections();

    let costs = intersections.into_iter().map(|(_, loc)| loc.total_cost());

    costs.min().expect("No intersections found")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_closest_intersection_distance() {
        assert_eq!(
            6,
            find_closest_intersection_distance(&vec!["R8,U5,L5,D3", "U7,R6,D4,L4"])
        );
        assert_eq!(
            159,
            find_closest_intersection_distance(&vec![
                "R75,D30,R83,U83,L12,D49,R71,U7,L72",
                "U62,R66,U55,R34,D71,R55,D58,R83",
            ])
        );
        assert_eq!(
            135,
            find_closest_intersection_distance(&vec![
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            ])
        );
    }

    #[test]
    fn test_find_lowest_intersection_cost() {
        assert_eq!(
            30,
            find_lowest_intersection_cost(&vec!["R8,U5,L5,D3", "U7,R6,D4,L4"])
        );
        assert_eq!(
            610,
            find_lowest_intersection_cost(&vec![
                "R75,D30,R83,U83,L12,D49,R71,U7,L72",
                "U62,R66,U55,R34,D71,R55,D58,R83",
            ])
        );
        assert_eq!(
            410,
            find_lowest_intersection_cost(&vec![
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            ])
        );
    }

    #[test]
    fn run_part1() {
        assert_eq!(1431, part1());
    }

    #[test]
    fn run_part2() {
        assert_eq!(48012, part2());
    }
}
