#![allow(dead_code, unused_variables)]

type AdventResult = usize;

mod computer;

use computer::Computer;
use computer::Intcode;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::fs;
use std::ops::RangeInclusive;

type Coordinate = i32;
type CoordinateDistance = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position(Coordinate, Coordinate);

impl Position {
    fn manhattan(&self, other: &Position) -> u32 {
        (self.0 - other.0).abs() as u32 + 
            (self.1 - other.1).abs() as u32
    }
    
    fn max(&self, other: &Position) -> Position {
        Position(self.0.max(other.0), self.1.max(other.1))
    }

    fn min(&self, other: &Position) -> Position {
        Position(self.0.min(other.0), self.1.min(other.1))
    }
}

#[derive(Clone)]
struct PartialPath<'a> {
    path: Vec<Position>,
    maze: &'a Maze,
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
    fn cmp(&self, other: &Self) -> Ordering {
        self.min_cost_to_goal().cmp(&other.min_cost_to_goal())
    }
}

impl<'a> PartialPath<'a> {
    fn new(maze: &'a Maze) -> Self {
        let path = vec![Position(0, 0)];
        PartialPath {
            path,
            maze
        }
    }
    
    fn branch(&self, p: Position) -> Self {
        let mut path = self.path.clone();
        path.push(p);
        PartialPath {
            path,
            maze: self.maze,
        }
    }

    fn min_cost_to_goal(&self) -> u32 {
        let p = self.path.last().unwrap();
        p.manhattan(&self.maze.goal)
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

    fn code(&self) -> Intcode {
        match self {
            North => 1,
            East => 4,
            South => 2,
            West => 3,
        }
    }

    fn of(&self, p: Position) -> Position {
        let Position(x, y) = p;
        match self {
            North => Position(x, y - 1),
            East => Position(x + 1, y),
            South => Position(x, y + 1),
            West => Position(x - 1, y),
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
    Oxygen,
}
use Tile::{Empty, Origin, Oxygen, Wall};

impl Tile {
    fn from_result(n: Intcode) -> Self {
        match n {
            0 => Wall,
            1 => Empty,
            2 => Oxygen,
            _ => panic!("unexpected result {}", n),
        }
    }
}

struct Maze {
    goal: Position,
    map: HashMap<Position, Tile>,
    pos_max: Position,
    pos_min: Position,
}

impl Maze {
    fn new(map: HashMap<Position, Tile>) -> Self {
        assert!(map.len() > 0);

        let mut pos_min = Position(Coordinate::MAX, Coordinate::MAX);
        let mut pos_max = Position(Coordinate::MIN, Coordinate::MIN);
        let mut goal = None;

        for (pos, tile) in &map {
            pos_min = pos_min.min(pos);
            pos_max = pos_max.max(pos);
            if *tile == Oxygen {
                assert_eq!(None, goal);
                goal = Some(*pos);
            }
        }

        let goal = goal.expect("maze should have an Oxygen tile");

        Maze {
            goal,
            map,
            pos_max,
            pos_min,
        }
    }

    fn display(&self) {
        let mut sb = String::new();

        for y in self.yrange() {
            if sb.len() > 0 {
                sb += "\n";
            }
            for x in self.xrange() {
                sb += match self.map.get(&Position(x, y)) {
                    None => "•",
                    Some(Origin) => "*",
                    Some(Empty) => " ",
                    Some(Wall) => "▓",
                    Some(Oxygen) => "X",
                };
            }
        }

        println!("{}", sb);
    }

    fn shortest_path(&self) -> usize {
        let mut paths = BinaryHeap::new();
        paths.push(PartialPath::new(self));

        while let Some(path) = paths.pop() {
            let pos = *path.last().unwrap();
            for d in Direction::ALL {
                let next = d.of(pos);
                if path.contains(&next) {
                    continue;
                }
                match self.map.get(&next).unwrap() {
                    Oxygen => return path.len(),
                    Wall => continue,
                    Empty => paths.push(path.branch(next)),
                    Origin => panic!("should not return to origin"),
                }
            }
        }
        panic!("expected to find a path to goal");
    }

    fn xrange(&self) -> RangeInclusive<Coordinate> {
        self.pos_min.0..=self.pos_max.0
    }

    fn yrange(&self) -> RangeInclusive<Coordinate> {
        self.pos_min.1..=self.pos_max.1
    }
}

struct MazeMapper {
    computer: Computer,
    map: HashMap<Position, Tile>,
    position: Position,
}

impl MazeMapper {
    fn build_maze(input: &str) -> Maze {
        let program = Computer::parse_program(input);
        let computer = Computer::new(program);
        let mut map = HashMap::new();
        map.insert(Position(0, 0), Origin);

        let mut mapper = MazeMapper {
            computer,
            map,
            position: Position(0, 0),
        };
        mapper.explore();
        Maze::new(mapper.map)
    }

    fn explore(&mut self) {
        for d in Direction::ALL {
            if !self.map.contains_key(&d.of(self.position)) {
                self.venture(d);
            }
        }
    }

    fn venture(&mut self, d: Direction) {
        let origin = self.position;
        let dest = d.of(origin);

        // Try moving to the destination
        self.computer.buffer_input(d.code());
        self.computer.start_or_resume();

        // Record the contents of the destination
        let result = self.computer.consume_output().expect("should get output");
        let tile = Tile::from_result(result);
        self.map.insert(dest, tile);

        // In the case of a Wall, we did not actually move. We're done.
        if tile == Wall {
            return;
        }

        // Otherwise, we did successfully move. Update our position
        // and recursively explore
        self.position = dest;
        self.explore();

        // Complete the venture by reversing the movement
        self.computer.buffer_input(d.reverse().code());
        self.computer.start_or_resume();
        assert_ne!(
            Wall,
            Tile::from_result(self.computer.consume_output().unwrap())
        );
        self.position = origin;
    }
}

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn do_part1(input: &str) -> AdventResult {
    let maze = MazeMapper::build_maze(input);
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
    fn part2_example() {
        todo!()
    }

    #[test]
    fn part1_solution() {
        assert_eq!(300, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(AdventResult::MAX, part2());
    }
}
