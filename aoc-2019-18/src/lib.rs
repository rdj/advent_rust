#![allow(dead_code, unused_variables)]

type AdventResult = usize;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;

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
struct PartialPath {
    cost: usize,
    min_remaining: usize,
    keys_acquired: String,
}

impl PartialEq for PartialPath {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for PartialPath {}

impl PartialOrd for PartialPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PartialPath {
    // compares opposite natural ordering because lower cost = higher
    // priority for the queue
    fn cmp(&self, other: &Self) -> Ordering {
        let c = other.min_cost_to_goal().cmp(&self.min_cost_to_goal());
        if c == Ordering::Equal {
            let c = other.cost.cmp(&self.cost);
            if c == Ordering::Equal {
                return self.keys_acquired.cmp(&other.keys_acquired);
            }
            return c;
        }
        return c;
    }
}

impl PartialPath {
    fn new(maze: &Maze) -> Self {
        let min_remaining = maze
            .keys
            .iter()
            .map(|(_, pos)| maze.origin.manhattan(pos))
            .max()
            .unwrap();
        PartialPath {
            min_remaining,
            keys_acquired: String::new(),
            cost: 0,
        }
    }

    fn branch(&self, key: char, marginal_cost: usize, maze: &Maze) -> Self {
        let mut keys_acquired = self.keys_acquired.clone();
        keys_acquired.push(key);

        let keypos = maze.keys[&key];

        let cost = self.cost + marginal_cost;

        let min_remaining = maze
            .keys
            .iter()
            .filter(|(&key, _)| !keys_acquired.contains(key))
            .map(|(_, pos)| keypos.manhattan(pos))
            .max()
            .unwrap_or(0);

        PartialPath {
            min_remaining,
            keys_acquired,
            cost,
        }
    }

    fn has_key(&self, key: char) -> bool {
        self.keys_acquired.contains(key)
    }

    fn meets_requirements(&self, key_requirements: &str) -> bool {
        key_requirements.chars().all(|k| self.has_key(k))
    }

    fn min_cost_to_goal(&self) -> usize {
        self.cost + self.min_remaining
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
        Pathfinder::new(&self).shortest_path()
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

#[derive(Clone)]
struct PartialConnection {
    goal: Position,
    path: Vec<Position>,
}

impl PartialEq for PartialConnection {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.goal == other.goal
    }
}

impl Eq for PartialConnection {}

impl PartialOrd for PartialConnection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PartialConnection {
    // compares opposite natural ordering because lower cost = higher
    // priority for the queue
    fn cmp(&self, other: &Self) -> Ordering {
        other.min_cost_to_goal().cmp(&self.min_cost_to_goal())
    }
}

impl PartialConnection {
    fn new(from: Position, to: Position) -> Self {
        let path = vec![from];
        PartialConnection { goal: to, path }
    }

    fn branch(&self, p: Position) -> Self {
        let mut path = self.path.clone();
        path.push(p);
        PartialConnection {
            goal: self.goal,
            path,
        }
    }

    fn cost(&self) -> usize {
        self.len() - 1
    }

    fn min_cost_to_goal(&self) -> usize {
        self.cost() + self.last().unwrap().manhattan(&self.goal)
    }
}

impl Deref for PartialConnection {
    type Target = Vec<Position>;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

struct Connection {
    from: char,
    to: char,
    cost: usize,
    requires: String,
}

struct Pathfinder<'a> {
    maze: &'a Maze,
    connections: HashMap<(char, char), Connection>,
}

impl<'a> Pathfinder<'a> {
    fn new(maze: &'a Maze) -> Self {
        Pathfinder {
            maze,
            connections: HashMap::new(),
        }
    }

    fn new_connection(&self, a: char, b: char, part: &PartialConnection) -> Connection {
        let cost = part.cost();
        let mut requires = String::new();
        for pos in &part.path {
            match self.maze.tile_at(pos) {
                Tile::Door(d) => requires.push(d.to_ascii_lowercase()),
                _ => (),
            }
        }
        let from = a.min(b);
        let to = b.max(a);
        Connection {
            from,
            to,
            cost,
            requires,
        }
    }

    fn get_cached_connection(&self, a: char, b: char) -> Option<&Connection> {
        let from = a.min(b);
        let to = b.max(a);
        self.connections.get(&(from, to))
    }

    fn get_new_connection(&mut self, from: char, to: char) -> &Connection {
        let mut partials = BinaryHeap::new();
        partials.push(PartialConnection::new(
            *self.maze.keys.get(&from).unwrap_or(&self.maze.origin),
            *self.maze.keys.get(&to).unwrap(),
        ));

        while let Some(part) = partials.pop() {
            let pos = *part.last().unwrap();
            for next in &pos.neighbors() {
                if part.contains(next) {
                    continue;
                }

                if *next == part.goal {
                    self.connections.insert(
                        (from, to),
                        self.new_connection(from, to, &part.branch(*next)),
                    );
                    return self.connections.get(&(from, to)).unwrap();
                }

                match self.maze.tile_at(next) {
                    Tile::Wall => continue,
                    _ => partials.push(part.branch(*next)),
                }
            }
        }

        panic!("found no path {} -> {}", from, to);
    }

    fn shortest_path(&mut self) -> usize {
        let mut best_cost_for_keys: HashMap<String, usize> = HashMap::new();
        let mut partials = BinaryHeap::new();
        partials.push(PartialPath::new(self.maze));

        while let Some(part) = partials.pop() {
            if partials.len() > 100_000 {
                panic!("too many paths");
            }

            if part.min_remaining == 0 {
                return part.cost;
            }
            let last_key = part.keys_acquired.chars().last().unwrap_or('@');
            for (&next_key, _) in &self.maze.keys {
                if part.has_key(next_key) {
                    continue;
                }

                let keys_sorted: String = {
                    // It only matters which key we picked up last
                    let mut chars: Vec<_> = part.keys_acquired.chars().collect();
                    chars.sort();
                    chars.push(next_key);
                    chars.into_iter().collect()
                };

                let prev_best = best_cost_for_keys.get_mut(&keys_sorted);
                if let Some(&mut cost) = prev_best {
                    let min_cost_to_next =
                        self.maze.keys[&last_key].manhattan(&self.maze.keys[&next_key]);
                    if min_cost_to_next > cost {
                        continue;
                    }
                }

                let mut conn = self.get_cached_connection(last_key, next_key);
                if conn.is_none() {
                    conn = Some(self.get_new_connection(last_key, next_key));
                }
                let conn = conn.unwrap();

                if !part.meets_requirements(&conn.requires) {
                    continue;
                }

                let branch = part.branch(next_key, conn.cost, &self.maze);

                let mut pursue: bool = false;
                if let Some(cost) = prev_best {
                    if branch.cost < *cost {
                        pursue = true;
                        *cost = branch.cost;
                    }
                } else {
                    pursue = true;
                    best_cost_for_keys.insert(keys_sorted, branch.cost);
                }

                if pursue {
                    partials.push(branch);
                }
            }
        }

        panic!("found no path")
    }
}

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    let maze = Maze::new(input);
    let result = maze.shortest_path();
    result
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
    fn part1_example5() {
        let input = "\
########################
#@..............ac.GI.b#
###d#e#f################
###A#B#C################
###g#h#i################
########################";

        assert_eq!(81, do_part1(input));
    }

    //    #[test]
    fn part2_example() {
        todo!()
    }

    // #[test]
    fn part1_solution() {
        assert_eq!(AdventResult::MAX, part1());
    }

    //    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
