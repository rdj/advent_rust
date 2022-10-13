// -*- compile-command: "cargo test -- --show-output" -*-

#![allow(dead_code)]

type AdventResult = usize;

const WIDTH: usize = 25;
const HEIGHT: usize = 6;

use std::fs;

struct Image {
    pixels: Vec<usize>,
    width: usize,
    height: usize,
    layer_count: usize,
}

impl Image {
    fn new(pixels: Vec<usize>, width: usize, height: usize) -> Self {
        assert_eq!(0, pixels.len() % (width * height));
        let layer_count = pixels.len() / (width * height);
        Image { pixels, width, height, layer_count }
    }

    fn count_layer_occurences(&self, nlayer: usize, digit: usize) -> usize {
        let layer = self.get_layer_as_slice(nlayer);
        layer.iter().filter(|&n| *n == digit).count()
    }

    fn get_layer_as_slice(&self, nlayer: usize) -> &[usize] {
        assert!(nlayer < self.layer_count);
        let layer_size = self.width * self.height;
        &self.pixels[
            (layer_size * nlayer)..(layer_size * (nlayer + 1))
        ]
    }

    fn part1_calc(&self) -> usize {
        let mut zero_counts: Vec<(usize, usize)> =
            (0..self.layer_count).map(|i| (i, self.count_layer_occurences(i, 0))).collect();
        zero_counts.sort_by_key(|(_, n)| *n);

        let (layer, _) = *zero_counts.first().unwrap();

        let ones_count = self.count_layer_occurences(layer, 1);
        let twos_count = self.count_layer_occurences(layer, 2);

        ones_count * twos_count
    }

    fn render(&self) -> String {
        let mut output = Vec::from(self.get_layer_as_slice(0));

        for layer in 1..self.layer_count {
            let layer = self.get_layer_as_slice(layer);
            for (i, n) in layer.iter().enumerate() {
                let o = output.get_mut(i).unwrap();
                if *o == 2 {
                    *o = *n;
                }
            }
        }

        let mut iter = output.iter();
        let mut os = String::new();
        for _ in 0..self.height {
            if os.len() > 0 {
                os += "\n";
            }
            for _ in 0..self.width {
                os += &iter.next().unwrap().to_string();
            }
        }

        os
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn parse(input: &str) -> Vec<usize> {
    input.trim().chars().map(|c| c.to_digit(10).unwrap() as usize).collect()
}

pub fn part1() -> AdventResult {
    let image = Image::new(parse(&input()), WIDTH, HEIGHT);
    image.part1_calc()
}

pub fn part2() -> String {
    let image = Image::new(parse(&input()), WIDTH, HEIGHT);
    image.render()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_example() {
        // weirdly this puzzle had no example
        let input = "123456789012";
        let image = Image::new(parse(input), 3, 2);
        assert_eq!(1, image.part1_calc());
    }

    #[test]
    fn part2_example() {
        let input = "0222112222120000";
        let image = Image::new(parse(input), 2, 2);
        assert_eq!("01\n10", image.render());
    }
    
    #[test]
    fn part1_solution() {
        assert_eq!(1463, part1());
    }

    #[test]
    fn part2_solution() {
        // This is terrible, it graphically spells out
        //  ▮▮  ▮  ▮  ▮▮  ▮  ▮ ▮  ▮ 
        // ▮  ▮ ▮ ▮  ▮  ▮ ▮ ▮  ▮  ▮ 
        // ▮    ▮▮   ▮    ▮▮   ▮▮▮▮ 
        // ▮ ▮▮ ▮ ▮  ▮    ▮ ▮  ▮  ▮ 
        // ▮  ▮ ▮ ▮  ▮  ▮ ▮ ▮  ▮  ▮ 
        //  ▮▮▮ ▮  ▮  ▮▮  ▮  ▮ ▮  ▮
         
        assert_eq!("0110010010011001001010010\n1001010100100101010010010\n1000011000100001100011110\n1011010100100001010010010\n1001010100100101010010010\n0111010010011001001010010", part2());
    }
}
