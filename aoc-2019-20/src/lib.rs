#![allow(dead_code, unused_variables)]

type AdventResult = usize;

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::fs;

type Coordinate = usize;
type Distance = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position(Coordinate, Coordinate);

impl Position {
    fn manhattan(&self, other: &Position) -> Distance {
        (self.0.max(other.0) - self.0.min(other.0)) as Distance
            + (self.1.max(other.1) - self.1.min(other.1)) as Distance
    }

    fn neighbors(&self) -> Vec<Position> {
        let Position(x, y) = *self;
        let mut vec = vec![];
        if y > 0 {
            vec.push(Position(x, y - 1));
        }
        vec.push(Position(x + 1, y));
        vec.push(Position(x, y + 1));
        if x > 0 {
            vec.push(Position(x - 1, y));
        }
        vec
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.1.cmp(&other.1) {
            Ordering::Equal => self.0.cmp(&other.0),
            diff => diff,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
struct GatewayLabel(u16);

const ORIGIN: GatewayLabel = GatewayLabel(0);
const DESTINATION: GatewayLabel = GatewayLabel(25 << 8 | 25);

const INNER_BIT: u16 = 0x8000;

impl GatewayLabel {
    fn new(l1: u8, l2: u8, is_outer: bool) -> Self {
        let l1 = l1 as u16;
        let l2 = l2 as u16;
        let mut g = l1 << 8 | l2;
        if !is_outer {
            g |= INNER_BIT;
        }
        GatewayLabel(g)
    }

    fn connection_id(&self, other: &Self) -> u32 {
        let mut n = self.0 as u32;
        let mut m = other.0 as u32;
        if n < m {
            n <<= 16;
        } else {
            m <<= 16;
        }

        n | m
    }

    fn is_inner(&self) -> bool {
        0 != self.0 & INNER_BIT
    }

    fn is_outer(&self) -> bool {
        0 == self.0 & INNER_BIT
    }

    fn to_outer(&self) -> GatewayLabel {
        GatewayLabel(self.0 & !INNER_BIT)
    }

    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push(char::from('A' as u8 + (self.0 >> 8) as u8));
        s.push(char::from('A' as u8 + (self.0 & 0xff) as u8));
        s
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tile {
    Empty,
    Wall,
    GatewayLabelPart(u8),
    Gateway(GatewayLabel),
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        match c {
            '#' | ' ' => Tile::Wall,
            '.' => Tile::Empty,
            'A'..='Z' => Tile::GatewayLabelPart(c as u8 - 'A' as u8),
            _ => panic!("unknown tile char {}", c),
        }
    }
}

struct Maze {
    rowlen: usize,
    tiles: Vec<Tile>,
}

impl Maze {
    fn new(input: &str) -> Self {
        let mut tiles = vec![];
        let mut rowlen = 0;

        // leading/trailing whitespace is significant for this puzzle, no trim
        for (row, line) in input.lines().enumerate() {
            if rowlen == 0 {
                rowlen = line.len().try_into().unwrap();
            } else {
                assert_eq!(rowlen, line.len().try_into().unwrap());
            }

            for (col, c) in line.chars().enumerate() {
                let tile = Tile::from(c);
                tiles.push(tile);
            }
        }

        let mut maze = Maze { rowlen, tiles };

        maze.place_gateways();

        maze
    }

    fn build_graph(&self) -> BTreeMap<u32, u32> {
        #[derive(Clone)]
        struct Partial {
            start: GatewayLabel,
            path: Vec<Position>,
        }

        impl Partial {
            fn branch(&self, pos: Position) -> Partial {
                let mut branch = self.clone();
                branch.path.push(pos);
                branch
            }
        }

        let mut costs = BTreeMap::new();

        // Every inner gateway has a direct connection to its outer
        // counterpart at cost 1. For part 2 walking this transition
        // changes levels of the maze.
        for g in self.gateway_labels().into_iter().filter(|g| g.is_inner()) {
            costs.insert(g.connection_id(&g.to_outer()), 1);
        }

        let mut work: Vec<_> = self
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match t {
                Tile::Gateway(label) => Some(Partial {
                    start: *label,
                    path: vec![self.index_to_pos(i)],
                }),
                _ => None,
            })
            .collect();

        while let Some(part) = work.pop() {
            let pos = part.path.last().unwrap();
            for npos in pos.neighbors() {
                if part.path.contains(&npos) {
                    continue;
                }
                match self.tile_at(&npos) {
                    Tile::Wall | Tile::GatewayLabelPart(_) => continue,
                    Tile::Empty => work.push(part.branch(npos)),
                    Tile::Gateway(label) => {
                        if *label == part.start {
                            continue;
                        }
                        let connid = part.start.connection_id(label);
                        let cost = part.path.len() as u32;
                        costs
                            .entry(connid)
                            .and_modify(|existing| {
                                if cost < *existing {
                                    *existing = cost
                                }
                            })
                            .or_insert(cost);
                    }
                }
            }
        }

        costs
    }

    fn place_gateways(&mut self) {
        let parts: Vec<_> = self
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match t {
                Tile::GatewayLabelPart(_) => Some(i),
                _ => None,
            })
            .collect();

        for i in parts {
            let label_pos = self.index_to_pos(i);
            let label = match self.tile_at(&label_pos) {
                Tile::GatewayLabelPart(l) => l,
                _ => continue,
            };

            let mut label2 = None;
            let mut gateway_pos = None;

            for npos in label_pos.neighbors().into_iter() {
                match self.tile_at(&npos) {
                    Tile::Empty => gateway_pos = Some(npos),
                    Tile::GatewayLabelPart(l) => label2 = Some((npos, l)),
                    _ => {}
                }
            }

            if let Some(gateway_pos) = gateway_pos {
                let (label2_pos, label2) = label2.unwrap();
                let is_outer = self.is_outer(&gateway_pos);

                let gateway_label = if label_pos < label2_pos {
                    GatewayLabel::new(*label, *label2, is_outer)
                } else {
                    GatewayLabel::new(*label2, *label, is_outer)
                };

                self.tile_replace(&gateway_pos, Tile::Gateway(gateway_label));
            }
        }
    }

    fn gateway_labels(&self) -> Vec<GatewayLabel> {
        self.tiles
            .iter()
            .filter_map(|t| match t {
                Tile::Gateway(label) => Some(*label),
                _ => None,
            })
            .collect()
    }

    fn index_to_pos(&self, index: usize) -> Position {
        Position(index % self.rowlen, index / self.rowlen)
    }

    fn is_outer(&self, p: &Position) -> bool {
        p.0 < 3 || p.0 + 3 >= self.rowlen || p.1 < 3 || p.1 + 3 >= self.tiles.len() / self.rowlen
    }

    fn pos_to_index(&self, p: &Position) -> usize {
        let Position(x, y) = p;
        y * self.rowlen + x
    }

    fn tile_at(&self, p: &Position) -> &Tile {
        match self.tiles.get(self.pos_to_index(p)) {
            Some(t) => t,
            None => &Tile::Wall,
        }
    }

    fn tile_replace(&mut self, p: &Position, new: Tile) {
        let index = self.pos_to_index(p);
        let old = self.tiles.get_mut(index).unwrap();
        *old = new;
    }
}

struct MinHeapEntry<T>(u32, T);

impl<T> PartialEq for MinHeapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for MinHeapEntry<T> {}

impl<T> PartialOrd for MinHeapEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for MinHeapEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

fn dijkstra(
    labels: &Vec<GatewayLabel>,
    costs: &BTreeMap<u32, u32>,
    start: GatewayLabel,
    end: GatewayLabel,
) -> u32 {
    let mut distances = BTreeMap::new();
    let mut visited = BTreeSet::new();
    let mut to_visit = BinaryHeap::new();

    distances.insert(start, 0);
    to_visit.push(MinHeapEntry(0, start));

    while let Some(MinHeapEntry(distance, label)) = to_visit.pop() {
        if !visited.insert(label) {
            continue;
        }

        if label == end {
            return distances[&end];
        }

        for neighbor in labels.iter().filter(|n| !visited.contains(n)) {
            if let Some(cost) = costs.get(&label.connection_id(neighbor)) {
                let new_distance = distance + cost;
                let is_shorter = distances
                    .get(neighbor)
                    .map_or(true, |existing| new_distance < *existing);

                if is_shorter {
                    distances.insert(*neighbor, new_distance);
                    to_visit.push(MinHeapEntry(new_distance, *neighbor));
                }
            }
        }
    }

    panic!("path not found")
}

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    let maze = Maze::new(input);
    let costs = maze.build_graph();
    // for (conn, cost) in costs {
    //     let a = GatewayLabel((conn >> 16) as u16);
    //     let b = GatewayLabel((conn & 0xFFFF) as u16);
    //     println!("{} <=> {} : {}", a.to_string(), b.to_string(), cost);
    // }
    let nodes = maze.gateway_labels();

    dijkstra(&nodes, &costs, ORIGIN, DESTINATION) as usize
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
        let input = "         A           
         A           
  #######.#########  
  #######.........#  
  #######.#######.#  
  #######.#######.#  
  #######.#######.#  
  #####  B    ###.#  
BC...##  C    ###.#  
  ##.##       ###.#  
  ##...DE  F  ###.#  
  #####    G  ###.#  
  #########.#####.#  
DE..#######...###.#  
  #.#########.###.#  
FG..#########.....#  
  ###########.#####  
             Z       
             Z       ";
        assert_eq!(23, do_part1(input));
    }

    #[test]
    fn part1_example2() {
        let input = "                   A               
                   A               
  #################.#############  
  #.#...#...................#.#.#  
  #.#.#.###.###.###.#########.#.#  
  #.#.#.......#...#.....#.#.#...#  
  #.#########.###.#####.#.#.###.#  
  #.............#.#.....#.......#  
  ###.###########.###.#####.#.#.#  
  #.....#        A   C    #.#.#.#  
  #######        S   P    #####.#  
  #.#...#                 #......VT
  #.#.#.#                 #.#####  
  #...#.#               YN....#.#  
  #.###.#                 #####.#  
DI....#.#                 #.....#  
  #####.#                 #.###.#  
ZZ......#               QG....#..AS
  ###.###                 #######  
JO..#.#.#                 #.....#  
  #.#.#.#                 ###.#.#  
  #...#..DI             BU....#..LF
  #####.#                 #.#####  
YN......#               VT..#....QG
  #.###.#                 #.###.#  
  #.#...#                 #.....#  
  ###.###    J L     J    #.#.###  
  #.....#    O F     P    #.#...#  
  #.###.#####.#.#####.#####.###.#  
  #...#.#.#...#.....#.....#.#...#  
  #.#####.###.###.#.#.#########.#  
  #...#.#.....#...#.#.#.#.....#.#  
  #.###.#####.###.###.#.#.#######  
  #.#.........#...#.............#  
  #########.###.###.#############  
           B   J   C               
           U   P   P               ";

        assert_eq!(58, do_part1(input));
    }

    #[test]
    fn part2_example() {
        todo!()
    }

    #[test]
    fn part1_solution() {
        assert_eq!(618, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
