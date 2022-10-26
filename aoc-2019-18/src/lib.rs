#![cfg_attr(not(test), allow(dead_code, unused_variables))]

type AdventResult = usize;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fs;

type Coordinate = i32;
type Distance = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position(Coordinate, Coordinate);

impl Position {
    fn manhattan(&self, other: &Position) -> Distance {
        (self.0.max(other.0) - self.0.min(other.0)) as Distance
            + (self.1.max(other.1) - self.1.min(other.1)) as Distance
    }

    fn neighbors(&self) -> [Position; 4] {
        let Position(x, y) = *self;
        [
            Position(x, y - 1),
            Position(x + 1, y),
            Position(x, y + 1),
            Position(x - 1, y),
        ]
    }
}

const PART2_ORIGIN_COUNT: usize = 4;

type Positions = [Position; PART2_ORIGIN_COUNT];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tile {
    Origin,
    Empty,
    Wall,
    Key(u8),
    Door(u8),
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        match c {
            '#' => Tile::Wall,
            '.' => Tile::Empty,
            '@' => Tile::Origin,
            'A'..='Z' => Tile::Door(Maze::doorno(c)),
            'a'..='z' => Tile::Key(Maze::keyno(c)),
            _ => panic!("unknown tile char {}", c),
        }
    }
}

const MAX_KEYS: usize = 26;

struct Maze {
    all_keys: [Position; MAX_KEYS],
    key_count: u8,
    origin: Positions,
    rowlen: usize,
    tiles: Vec<Tile>,
    part2: bool,
}

impl Maze {
    fn new(input: &str) -> Self {
        let mut tiles = vec![];
        let mut rowlen = 0;
        let mut origin = [Position(0, 0); 4];
        let mut all_keys = [Position(0, 0); MAX_KEYS];
        let mut last_key = 0;

        for (row, line) in input.trim().lines().enumerate() {
            let line = line.trim();
            rowlen = line.len().try_into().unwrap();
            for (col, c) in line.chars().enumerate() {
                let tile = Tile::from(c);
                tiles.push(tile);
                match tile {
                    Tile::Origin => {
                        origin[0] = Position(row as Coordinate, col as Coordinate);
                    }
                    Tile::Key(k) => {
                        all_keys[k as usize] = Position(row as Coordinate, col as Coordinate);
                        last_key = last_key.max(k);
                    }
                    _ => {}
                }
            }
        }

        Maze {
            all_keys,
            key_count: last_key + 1,
            origin,
            rowlen,
            tiles,
            part2: false,
        }
    }

    fn enable_part2(&mut self) {
        self.part2 = true;

        let Position(r, c) = self.origin[0];
        self.tile_replace(&Position(r - 1, c - 1), Tile::Origin);
        self.tile_replace(&Position(r - 1, c), Tile::Wall);
        self.tile_replace(&Position(r - 1, c + 1), Tile::Origin);
        self.tile_replace(&Position(r, c - 1), Tile::Wall);
        self.tile_replace(&Position(r, c), Tile::Wall);
        self.tile_replace(&Position(r, c + 1), Tile::Wall);
        self.tile_replace(&Position(r + 1, c - 1), Tile::Origin);
        self.tile_replace(&Position(r + 1, c), Tile::Wall);
        self.tile_replace(&Position(r + 1, c + 1), Tile::Origin);

        self.origin[0] = Position(r - 1, c - 1);
        self.origin[1] = Position(r - 1, c + 1);
        self.origin[2] = Position(r + 1, c - 1);
        self.origin[3] = Position(r + 1, c + 1);
    }

    fn quadrants(&self) -> impl Iterator<Item = usize> {
        if self.part2 {
            (0..PART2_ORIGIN_COUNT).into_iter()
        } else {
            (0..1).into_iter()
        }
    }

    fn quadrant(&self, pos: &Position) -> usize {
        if !self.part2 {
            return 0;
        }

        if pos.0 <= self.origin[0].0 {
            if pos.1 <= self.origin[0].1 {
                0
            } else {
                1
            }
        } else if pos.1 <= self.origin[2].1 {
            2
        } else {
            3
        }
    }

    fn doorno(door: char) -> u8 {
        door as u8 - 'A' as u8
    }

    fn keys(&self) -> &[Position] {
        &self.all_keys[0..self.key_count as usize]
    }

    fn keyno(key: char) -> u8 {
        key as u8 - 'a' as u8
    }

    fn shortest_path(&self) -> usize {
        Pathfinder::new(&self).shortest_path()
    }

    fn tile_at(&self, p: &Position) -> &Tile {
        let Position(r, c) = *p;
        if r < 0 || c < 0 {
            &Tile::Wall
        } else if let Some(t) = self.tiles.get(r as usize * self.rowlen + c as usize) {
            t
        } else {
            &Tile::Wall
        }
    }

    fn tile_replace(&mut self, p: &Position, new: Tile) {
        let Position(r, c) = *p;
        let old = self
            .tiles
            .get_mut(r as usize * self.rowlen + c as usize)
            .unwrap();
        *old = new;
    }
}

struct PartialConnection {
    path: Vec<Position>,
    min_cost_remaining: u32,
}

impl PartialConnection {
    fn new(origin: Position, goal: &Position) -> Self {
        PartialConnection {
            min_cost_remaining: origin.manhattan(goal),
            path: vec![origin],
        }
    }

    fn branch(&self, next: Position, goal: &Position) -> Self {
        let mut path = self.path.clone();
        path.push(next);
        PartialConnection {
            path,
            min_cost_remaining: next.manhattan(goal),
        }
    }

    fn min_cost_to_goal(&self) -> usize {
        self.path.len() + self.min_cost_remaining as usize
    }
}

impl PartialEq for PartialConnection {
    fn eq(&self, other: &Self) -> bool {
        Ordering::Equal == self.cmp(other)
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
        match other.min_cost_to_goal().cmp(&self.min_cost_to_goal()) {
            Ordering::Equal => self.path.len().cmp(&other.path.len()),
            x => x,
        }
    }
}

struct Connection {
    cost: u32,
    keys_required: u32,
}

struct PartialPath {
    position: Positions,
    keys: u32,
    cost: u32,
    min_cost_remaining: u32,
}

impl PartialPath {
    fn new(origin: &Positions, maze: &Maze) -> PartialPath {
        PartialPath {
            position: origin.clone(),
            keys: 0,
            cost: 0,
            min_cost_remaining: Self::mincost(origin, 0, &maze),
        }
    }

    fn branch(
        &self,
        position: Positions,
        connection_cost: u32,
        keyno_acquired: u8,
        maze: &Maze,
    ) -> Self {
        let keys = self.keys | 1 << keyno_acquired;

        PartialPath {
            position,
            keys,
            cost: self.cost + connection_cost,
            min_cost_remaining: Self::mincost(&position, keys, &maze),
        }
    }

    fn mincost(position: &Positions, keys: u32, maze: &Maze) -> u32 {
        maze.quadrants()
            .map(|q| {
                maze.keys()
                    .iter()
                    .enumerate()
                    .filter(|(keyno, _)| 0 == keys & 1 << *keyno)
                    .filter(|(_, pos)| maze.quadrant(pos) == q)
                    .map(|(_, pos)| position[q].manhattan(pos))
                    .max()
                    .unwrap_or(0)
            })
            .sum()
    }

    fn has_keyno(&self, keyno: u8) -> bool {
        0 != self.keys & 1 << keyno
    }

    fn has_keys(&self, keys_required: u32) -> bool {
        0 == (self.keys & keys_required) ^ keys_required
    }

    fn keycount(&self) -> u32 {
        self.keys.count_ones()
    }

    fn min_cost_to_goal(&self) -> u32 {
        self.cost + self.min_cost_remaining
    }
}

impl PartialEq for PartialPath {
    fn eq(&self, other: &Self) -> bool {
        Ordering::Equal == self.cmp(other)
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
        match other.min_cost_to_goal().cmp(&self.min_cost_to_goal()) {
            Ordering::Equal => self.keycount().cmp(&other.keycount()),
            x => x,
        }
    }
}

struct Pathfinder<'a> {
    maze: &'a Maze,
    connections: HashMap<(Position, Position), Connection>,
}

impl<'a> Pathfinder<'a> {
    fn new(maze: &'a Maze) -> Self {
        Pathfinder {
            maze,
            connections: HashMap::new(),
        }
    }

    fn get_connection(&mut self, a: &Position, b: &Position) -> &Connection {
        let cmp = match a.0.cmp(&b.0) {
            Ordering::Equal => a.1.cmp(&b.1),
            diff => diff,
        };
        let (a, b) = match cmp {
            Ordering::Greater => (*b, *a),
            _ => (*a, *b),
        };

        self.connections.entry((a, b)).or_insert_with(|| {
            let mut heap = BinaryHeap::new();
            heap.push(PartialConnection::new(a, &b));

            while let Some(part) = heap.pop() {
                if part.min_cost_remaining == 0 {
                    let mut keys_required = 0;
                    for pos in &part.path {
                        if let Tile::Door(d) = self.maze.tile_at(pos) {
                            keys_required |= 1 << d;
                        }
                    }
                    return Connection {
                        cost: (part.path.len() - 1) as u32,
                        keys_required,
                    };
                }

                for next in part.path.last().unwrap().neighbors() {
                    if part.path.contains(&next) {
                        continue;
                    }
                    match self.maze.tile_at(&next) {
                        Tile::Wall => continue,
                        _ => heap.push(part.branch(next, &b)),
                    }
                }
            }

            panic!("failed to find connection {:?} <=> {:?}", a, b);
        })
    }

    fn shortest_path(&mut self) -> usize {
        let mut heap = BinaryHeap::new();
        let mut best_seen_map = HashMap::new();

        heap.push(PartialPath::new(&self.maze.origin, &self.maze));

        while let Some(part) = heap.pop() {
            if part.min_cost_remaining == 0 {
                return part.cost as usize;
            }

            for (next_keyno, next_keypos) in self.maze.keys().iter().enumerate() {
                let next_keyno = next_keyno as u8;
                if part.has_keyno(next_keyno) {
                    continue;
                }

                let quad = self.maze.quadrant(next_keypos);

                let next_conn = self.get_connection(&part.position[quad], next_keypos);
                if !part.has_keys(next_conn.keys_required) {
                    continue;
                }

                let mut newpos = part.position.clone();
                newpos[quad] = *next_keypos;

                let branch = part.branch(newpos, next_conn.cost, next_keyno, &self.maze);

                // Prune the branch if it is no better than an
                // already-seen branch at this position with the same
                // keys
                if let Some(best_seen) = best_seen_map.get_mut(&(branch.position, branch.keys)) {
                    if *best_seen <= branch.cost {
                        continue;
                    }
                    *best_seen = branch.cost;
                } else {
                    best_seen_map.insert((branch.position, branch.keys), branch.cost);
                }

                heap.push(branch);
            }
        }

        panic!("no complete path found");
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
    let mut maze = Maze::new(input);
    maze.enable_part2();
    let result = maze.shortest_path();
    result
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

    #[test]
    fn part2_example1() {
        let input = "\
#######
#a.#Cd#
##...##
##.@.##
##...##
#cB#Ab#
#######";

        assert_eq!(8, do_part2(input));
    }

    #[test]
    fn part2_example2() {
        let input = "\
###############
#d.ABC.#.....a#
###############
#######@#######
###############
#b.....#.....c#
###############";

        assert_eq!(24, do_part2(input));
    }

    #[test]
    fn part2_example3() {
        let input = "\
#############
#DcBa.#.GhKl#
#.#######I###
#e#d##@##j#k#
###C#######J#
#fEbA.#.FgHi#
#############";

        // This one does not follow the same rule of quadrants that
        // all the other examples (and the actual input) do. I'm not
        // going to bother fixing it, since it's a red herring.

        assert_eq!(32, do_part2(input));
    }

    #[test]
    fn part2_example4() {
        let input = "\
#############
#g#f.D#..h#l#
#F###e#E###.#
#dCba###BcIJ#
######@######
#nK.L###G...#
#M###N#H###.#
#o#m..#i#jk.#
#############";

        assert_eq!(72, do_part2(input));
    }

    #[test]
    fn part1_solution() {
        assert_eq!(4192, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(1790, part2());
    }
}
