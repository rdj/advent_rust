// -*- compile-command: "cargo test -- --show-output" -*-

mod computer;

type AdventResult = usize;

use std::collections::HashMap;
use std::fs;

use computer::Computer;
use computer::Intcode;

#[derive(PartialEq, Eq)]
enum Tile {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball,
}

impl Tile {
    fn from(c: Intcode) -> Self {
        match c {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Block,
            3 => Tile::HorizontalPaddle,
            4 => Tile::Ball,
            _ => panic!("expected valid tile id"),
        }
    }
}

struct Board {
    score: Intcode,
    tiles: HashMap<(Intcode, Intcode), Tile>,
    paddle: (Intcode, Intcode),
    ball: (Intcode, Intcode),
}

impl Board {
    fn new() -> Self {
        Board {
            score: 0,
            tiles: HashMap::new(),
            paddle: (0, 0),
            ball: (0, 0),
        }
    }

    #[allow(dead_code)]
    fn display(&self) {
        let mut sb = String::new();
        
        let (xmax, _) = *self.tiles.keys().max_by_key(|(x, _)| x).unwrap();
        let (_, ymax) = *self.tiles.keys().max_by_key(|(_, y)| y).unwrap();

        for y in 0..=ymax {
            if sb.len() > 0 {
                sb += "\n";
            }
            
            for x in 0..=xmax {
                let t = match self.tiles.get(&(x, y)).unwrap_or(&Tile::Empty) {
                    Tile::Empty => " ",
                    Tile::Wall => "#",
                    Tile::Block => "▮",
                    Tile::HorizontalPaddle => "=",
                    Tile::Ball => "●",
                };
                sb += t;
            }
        }

        println!("{}", sb);
        println!("Score: {}", self.score);
    }

    fn draw_tile(&mut self, pos: (Intcode, Intcode), tile: Tile) {
        match tile {
            Tile::HorizontalPaddle => self.paddle = pos,
            Tile::Ball => self.ball = pos,
            _ => ()
        }

        self.tiles.insert(pos, tile);
    }

    fn get_paddle_move(&self) -> Intcode {
        self.ball.0 - self.paddle.0
    }

    fn import(&mut self, data: impl IntoIterator<Item = Intcode>) {
        let data: Vec<_> = data.into_iter().collect();
        for chunk in data.chunks(3) {
            match chunk {
                &[-1, 0, s] => self.set_score(s),
                &[x, y, c] => self.draw_tile((x, y), Tile::from(c)),
                _ => panic!("expected outputs to be a multiple of 3"),
            }
        }        
    }

    fn set_score(&mut self, score: Intcode) {
        self.score = score;
    }

    fn tile_count(&self, tile: Tile) -> usize {
        self.tiles.values().filter(|&t| *t == tile).count()
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() -> AdventResult {
    let mut computer = Computer::new(Computer::parse_program(&input()));
    computer.start();
    assert!(computer.is_halted());
    let mut board = Board::new();
    board.import(computer.consume_output_buffer());
    board.tile_count(Tile::Block)
}

pub fn part2() -> Intcode {
    let mut computer = Computer::new(Computer::parse_program(&input()));
    computer.write(0, 2);

    let mut board = Board::new();

    while !computer.is_halted() {
        computer.start_or_resume();
        board.import(computer.consume_output_buffer());
        computer.buffer_input(board.get_paddle_move());
    }

    board.score
}

#[cfg(test)]
mod test {
    use super::*;

    // no examples

    #[test]
    fn part1_solution() {
        assert_eq!(273, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(13_140, part2());
    }
}
