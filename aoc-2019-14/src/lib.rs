// -*- compile-command: "cargo test -- --show-output" -*-

type AdventResult = usize;

use std::collections::HashMap;
use std::fmt;
use std::fs;

use regex::Regex;

struct Ingredient {
    name: String,
    quantity: usize,
}

impl Ingredient {
    fn new(name: &str, quantity: usize) -> Self {
        let name = name.to_owned();
        Ingredient { name, quantity }
    }
}

struct Recipe {
    inputs: Vec<Ingredient>,
    output: Ingredient,
}

impl Recipe {
    fn new(inputs: Vec<Ingredient>, output: Ingredient) -> Self {
        Recipe { inputs, output }
    }
}

impl fmt::Display for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sb = String::new();
        for input in &self.inputs {
            if sb.len() > 0 {
                sb += ", ";
            }
            sb += &format!("{} {}", input.quantity, input.name);
        }

        sb += &format!(" => {} {}", self.output.quantity, self.output.name);

        write!(f, "{}", sb)
    }
}

struct RecipeBook {
    recipes: HashMap<String, Recipe>,
}

impl RecipeBook {
    fn new(text: &str) -> Self {
        let mut recipes = HashMap::new();

        let re = Regex::new(r"(?P<qty>\d+) (?P<name>[A-Z]+)").unwrap();

        for line in text.lines() {
            let mut ingredients = Vec::new();

            for caps in re.captures_iter(line) {
                ingredients.push(Ingredient::new(&caps["name"], caps["qty"].parse().unwrap()));
            }

            let output = ingredients.pop().unwrap();
            let key = output.name.to_owned();
            let recipe = Recipe::new(ingredients, output);
            recipes.insert(key, recipe);
        }

        RecipeBook { recipes }
    }

    fn calculate_input_quantity(&self, input_name: &str, wanted: &Ingredient) -> usize {
        let mut inventory: HashMap<String, usize> = HashMap::new();
        let cost = self.calculate_input_quantity_inner(input_name, wanted, &mut inventory);
        cost
    }

    fn calculate_input_quantity_inner(
        &self,
        input_name: &str,
        wanted: &Ingredient,
        mut inventory: &mut HashMap<String, usize>,
    ) -> usize {
        if input_name == wanted.name {
            return wanted.quantity;
        }

        let cached = inventory.entry(wanted.name.clone()).or_insert(0);
        let mut needed = wanted.quantity;
        if needed <= *cached {
            *cached -= needed;
            return 0;
        }

        needed -= *cached;
        *cached = 0;

        let recipe = self
            .recipes
            .get(&wanted.name)
            .expect("should have a recipe for every ingredient");

        let single_yield = recipe.output.quantity;

        // intmath trick: ceil(i1/i2) = (i1 + i2 - 1) / i2
        let multiplier = (needed + single_yield - 1) / single_yield;

        let leftover = multiplier * single_yield - needed;
        *cached += leftover;

        recipe
            .inputs
            .iter()
            .map(|i| {
                self.calculate_input_quantity_inner(
                    input_name,
                    &Ingredient::new(&i.name, multiplier * i.quantity),
                    &mut inventory,
                )
            })
            .sum::<usize>()
    }

    fn calculate_yield(&self, output_name: &str, input: &Ingredient) -> usize {
        let unit_cost =
            self.calculate_input_quantity(&input.name, &Ingredient::new(output_name, 1));

        let mut inventory: HashMap<String, usize> = HashMap::new();

        let mut remaining = input.quantity;
        let mut cumulative_output = 0;
        while remaining > unit_cost {
            let batch_output = remaining / unit_cost;
            let batch_cost = self.calculate_input_quantity_inner(
                &input.name,
                &Ingredient::new(output_name, batch_output),
                &mut inventory,
            );

            remaining -= batch_cost;
            cumulative_output += batch_output;
        }

        cumulative_output
    }
}

pub fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

fn part1_do(input: &str) -> AdventResult {
    let book = RecipeBook::new(input);
    book.calculate_input_quantity("ORE", &Ingredient::new("FUEL", 1))
}

pub fn part1() -> AdventResult {
    part1_do(&input())
}

fn part2_do(input: &str) -> AdventResult {
    let book = RecipeBook::new(input);
    book.calculate_yield("FUEL", &Ingredient::new("ORE", 1_000_000_000_000))
}

pub fn part2() -> AdventResult {
    part2_do(&input())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_example1() {
        let input = "\
10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL";
        assert_eq!(31, part1_do(input));
    }

    #[test]
    fn part1_example2() {
        let input = "\
9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL";
        assert_eq!(165, part1_do(input));
    }

    #[test]
    fn part1_example3() {
        let input = "\
157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
";
        assert_eq!(13312, part1_do(input));
    }

    #[test]
    fn part1_example4() {
        let input = "\
2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF
";
        assert_eq!(180697, part1_do(input));
    }

    #[test]
    fn part1_example5() {
        let input = "\
171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX
";
        assert_eq!(2210736, part1_do(input));
    }

    #[test]
    fn part2_example1() {
        let input = "\
157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
";
        assert_eq!(82_892_753, part2_do(input));
    }

    #[test]
    fn part2_example2() {
        let input = "\
2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF
";
        assert_eq!(5586022, part2_do(input));
    }

    #[test]
    fn part2_example3() {
        let input = "\
171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX
";
        assert_eq!(460664, part2_do(input));
    }

    #[test]
    fn part1_solution() {
        assert_eq!(365768, part1());
    }

    #[test]
    fn part2_solution() {
        assert_eq!(3756877, part2());
    }
}
