#![allow(dead_code, unused_variables)]

mod computer;
use computer::{Computer, Intcode};

type AdventResult = Intcode;

use std::collections::VecDeque;
use std::fs;

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

type ComputerBuffer = VecDeque<(Intcode, Intcode)>;

fn do_part1(input: &str) -> AdventResult {
    const NCOMPUTERS: usize = 50;
    const EMPTY_BUFFER: Intcode = -1;

    let program = Computer::parse_program(input);

    let mut computers: Vec<Computer> = Vec::with_capacity(NCOMPUTERS);
    let mut buffers: Vec<ComputerBuffer> = Vec::with_capacity(NCOMPUTERS);

    for i in 0..NCOMPUTERS {
        let mut computer = Computer::new(program.clone());
        computer.buffer_input(i as Intcode);
        computer.start();
        computers.push(computer);
        buffers.push(VecDeque::new());
    }

    loop {
        for i in 0..NCOMPUTERS {
            let computer = &mut computers[i];

            let output: Vec<_> = computer.consume_output_buffer().collect();
            for output in output.chunks(3) {
                assert_eq!(3, output.len());
                let dest = output[0];
                if dest == 255 {
                    return output[2];
                }
                let packet = (output[1], output[2]);
                buffers[dest as usize].push_back(packet);
            }

            let buffer = &mut buffers[i];
            if buffer.len() > 0 {
                let packet = buffer.pop_front().unwrap();
                computer.buffer_input(packet.0);
                computer.buffer_input(packet.1);
            } else {
                computer.buffer_input(EMPTY_BUFFER);
            }
            computer.resume();
        }
    }
}

fn do_part2(input: &str) -> AdventResult {
    const NCOMPUTERS: usize = 50;
    const EMPTY_BUFFER: Intcode = -1;

    let program = Computer::parse_program(input);

    let mut computers: Vec<Computer> = Vec::with_capacity(NCOMPUTERS);
    let mut buffers: Vec<ComputerBuffer> = Vec::with_capacity(NCOMPUTERS);

    let mut nat = None;
    let mut nat_last_y = None;

    for i in 0..NCOMPUTERS {
        let mut computer = Computer::new(program.clone());
        computer.buffer_input(i as Intcode);
        computer.start();
        computers.push(computer);
        buffers.push(VecDeque::new());
    }

    loop {
        let mut sent = false;

        for i in 0..NCOMPUTERS {
            let computer = &mut computers[i];

            let output: Vec<_> = computer.consume_output_buffer().collect();
            for output in output.chunks(3) {
                assert_eq!(3, output.len());

                sent = true;

                let dest = output[0];
                let packet = (output[1], output[2]);

                if dest == 255 {
                    nat = Some(packet);
                } else {
                    buffers[dest as usize].push_back(packet);
                }
            }

            let buffer = &mut buffers[i];
            if buffer.len() > 0 {
                let packet = buffer.pop_front().unwrap();
                computer.buffer_input(packet.0);
                computer.buffer_input(packet.1);
            } else {
                computer.buffer_input(EMPTY_BUFFER);
            }
            computer.resume();
        }

        if !sent && buffers.iter().all(|b| b.is_empty()) {
            if let Some((x, y)) = nat {
                nat = None;

                if let Some(last_y) = nat_last_y {
                    if y == last_y {
                        return y;
                    }
                }

                nat_last_y = Some(y);
                buffers[0].push_back((x, y));
            }
        }
    }
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
    fn part1_solution() {
        assert_eq!(24268, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(19316, part2());
    }
}
