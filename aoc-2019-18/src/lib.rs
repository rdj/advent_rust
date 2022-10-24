#![allow(dead_code, unused_variables)]

type AdventResult = usize;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fs;

type Coordinate = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position(Coordinate, Coordinate);

impl Position {
    fn manhattan(&self, other: &Position) -> usize {
        self.0.max(other.0) - self.0.min(other.0) + self.1.max(other.1) - self.1.min(other.1)
    }

    fn max(&self, other: &Position) -> Position {
        Position(self.0.max(other.0), self.1.max(other.1))
    }

    fn min(&self, other: &Position) -> Position {
        Position(self.0.min(other.0), self.1.min(other.1))
    }

    fn neighbors(&self) -> Vec<Position> {
        let mut locs = vec![];
        for dir in Direction::ALL {
            if let Some(loc) = dir.of(self) {
                locs.push(loc);
            }
        }
        locs
    }
}

#[derive(Clone)]
struct PartialPath<'a> {
    path: Vec<Position>,
    maze: &'a Maze,
    keys_acquired: String,
    next_key: char,
    backtrack_ok: bool,
}

impl<'a> PartialEq for PartialPath<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> Eq for PartialPath<'a> {}

impl<'a> PartialOrd for PartialPath<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for PartialPath<'a> {
    // compares opposite natural ordering because lower cost = higher
    // priority for the queue
    fn cmp(&self, other: &Self) -> Ordering {
        other.min_cost_to_goal().cmp(&self.min_cost_to_goal())
    }
}

impl<'a> PartialPath<'a> {
    fn new(maze: &'a Maze, pos: Position, next_key: char) -> Self {
        let path = vec![pos];
        PartialPath {
            path,
            maze,
            next_key,
            keys_acquired: String::new(),
            backtrack_ok: false,
        }
    }

    fn branch(&self, p: Position) -> Self {
        let mut path = self.path.clone();
        path.push(p);
        PartialPath {
            path,
            maze: self.maze,
            keys_acquired: self.keys_acquired.clone(),
            next_key: self.next_key,
            backtrack_ok: false,
        }
    }

    fn can_open(&self, door: char) -> bool {
        self.has_key(door.to_ascii_lowercase())
    }

    fn cost(&self) -> usize {
        self.len() - 1
    }

    fn got_key(&mut self, key: char, next_key: char) {
        self.keys_acquired += &String::from(key);
        self.next_key = next_key;
        self.backtrack_ok = true;
    }

    fn has_key(&self, key: char) -> bool {
        self.keys_acquired.contains(key)
    }

    fn min_cost_to_goal(&self) -> usize {
        let p = self.last().unwrap();

        let key_costs: Vec<_> = self
            .maze
            .keys
            .iter()
            .filter(|(&k, _)| !self.has_key(k))
            .map(|(_, loc)| p.manhattan(loc))
            .collect();

        self.cost() + key_costs.iter().max().unwrap()
    }
}

use std::ops::Deref;

impl<'a> Deref for PartialPath<'a> {
    type Target = Vec<Position>;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    East,
    South,
    West,
}
use Direction::{East, North, South, West};

impl Direction {
    const ALL: [Self; 4] = [North, East, South, West];

    fn of(&self, p: &Position) -> Option<Position> {
        let Position(x, y) = *p;
        match self {
            North if y > 0 => Some(Position(x, y - 1)),
            East => Some(Position(x + 1, y)),
            South => Some(Position(x, y + 1)),
            West if x > 0 => Some(Position(x - 1, y)),
            _ => None,
        }
    }

    fn reverse(&self) -> Self {
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tile {
    Origin,
    Empty,
    Wall,
    Key(char),
    Door(char),
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        match c {
            '#' => Tile::Wall,
            '.' => Tile::Empty,
            '@' => Tile::Origin,
            'A'..='Z' => Tile::Door(c),
            'a'..='z' => Tile::Key(c),
            _ => panic!("unknown tile char {}", c),
        }
    }
}

struct Maze {
    keys: HashMap<char, Position>,
    origin: Position,
    rowlen: usize,
    tiles: Vec<Tile>,
}

impl Maze {
    fn new(input: &str) -> Self {
        let mut tiles = vec![];
        let mut rowlen = 0;
        let mut keys = HashMap::new();
        let mut origin = Position(0, 0);

        for (row, line) in input.trim().lines().enumerate() {
            let line = line.trim();
            rowlen = line.len().try_into().unwrap();
            for (col, c) in line.chars().enumerate() {
                let tile = Tile::from(c);
                tiles.push(tile);
                match tile {
                    Tile::Origin => {
                        origin = Position(row, col);
                    }
                    Tile::Key(_) => {
                        keys.insert(c, Position(row, col));
                    }
                    _ => {}
                }
            }
        }

        Maze {
            keys,
            origin,
            rowlen,
            tiles,
        }
    }

    fn shortest_path(&self) -> usize {
        let mut paths = BinaryHeap::new();
        for key in self.keys.keys() {
            paths.push(PartialPath::new(self, self.origin, *key));
        }

        while let Some(path) = paths.pop() {
            if paths.len() > 10_000 {
                panic!("too many paths");
            }
            // println!("Examining {:?}, looking for {}, acquired [{}]", path.path, path.next_key, path.keys_acquired);
            let pos = path.last().unwrap();

            let back = if path.len() > 1 {
                path.iter().nth(path.len() - 2).unwrap()
            } else {
                pos
            };

            for next in &pos.neighbors() {
                // println!("Maybe branch to {:?}", next);
                if !path.backtrack_ok && back == next {
                    // println!("pruning backtrack");
                    continue;
                }

                let tile = self.tile_at(next);
                if *tile == Tile::Wall {
                    // println!("pruning wall");
                    continue;
                }
                if let Tile::Door(c) = tile {
                    if !path.can_open(*c) {
                        // println!("pruning door {}", *c);
                        continue;
                    }
                    // println!("entering door {}", *c);
                }

                if let Tile::Key(key) = tile {
                    if *key == path.next_key {
                        // println!("found key {}", *key);
                        let mut keys_needed = 0;
                        for other_key in self.keys.keys() {
                            if other_key != key && !path.has_key(*other_key) {
                                let mut branch = path.branch(*next);
                                branch.got_key(*key, *other_key);
                                paths.push(branch);
                                keys_needed += 1;
                            }
                        }
                        if keys_needed == 0 {
                            return path.len();
                        }
                        continue;
                    } else if !path.has_key(*key) {
                        // println!("pruning wrong key {}", *key);
                        continue;
                    }
                    // already acquired key falls through
                }

                paths.push(path.branch(*next));
            }
        }

        // I think I must be pruning something that I shouldn't prune
        panic!("did not find a path to all goals");
    }

    fn tile_at(&self, p: &Position) -> &Tile {
        let Position(r, c) = *p;
        if let Some(t) = self.tiles.get(r * self.rowlen + c) {
            t
        } else {
            &Tile::Wall
        }
    }
}

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    let maze = Maze::new(input);
    maze.shortest_path()
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
    fn part1_example1() {
        let input = "\
#########
#b.A.@.a#
#########";

        assert_eq!(8, do_part1(input));
    }

    #[test]
    fn part1_example2() {
        let input = "\
########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################";

        assert_eq!(86, do_part1(input));
    }

    #[test]
    fn part1_example3() {
        let input = "\
########################
#...............b.C.D.f#
#.######################
#.....@.a.B.c.d.A.e.F.g#
########################";

        assert_eq!(132, do_part1(input));
    }

    #[test]
    fn part1_example4() {
        let input = "\
#################
#i.G..c...e..H.p#
########.########
#j.A..b...f..D.o#
########@########
#k.E..a...g..B.n#
########.########
#l.F..d...h..C.m#
#################";

        assert_eq!(136, do_part1(input));
    }
    

    #[test]
    fn part2_example() {
        todo!()
    }

    #[test]
    fn part1_solution() {
        // assert_eq!(AdventResult::MAX, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
