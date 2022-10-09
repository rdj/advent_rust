use std::fs;

type Mass = usize;
type Fuel = usize;

fn input() -> String {
    fs::read_to_string("input.txt").expect("Can't find input.txt")
}

pub fn part1() {
    println!("# Part 1");
    let mut total: Mass = 0;
    
    for line in input().lines() {
        let mass: Mass = line.parse().unwrap();
        let fuel = fuel_for_mass(mass);
        println!("mass = {mass}; fuel = {fuel}");
        total += fuel;
    }

    println!("total = {total}");
}

pub fn part2() {
    println!("# Part 2");
    let mut total: Mass = 0;
    
    for line in input().lines() {
        let mass: Mass = line.parse().unwrap();
        let fuel = fuel_for_mass_and_fuel(mass);
        total += fuel;
    }

    println!("total = {total}");
}

fn fuel_for_mass(mass: Mass) -> Fuel {
    let div3 = mass / 3;
    if div3 <= 2 {
        0
    } else {
        div3 - 2
    }
}

fn fuel_for_mass_and_fuel(mass: Mass) -> Fuel {
    let mut total_fuel = 0;
    let mut mass = mass;
    while mass > 0 {
        let fuel = fuel_for_mass(mass);
        total_fuel += fuel;
        mass = fuel;
    }
    total_fuel
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_fuel_for_mass() {
        assert_eq!(fuel_for_mass(12), 2);
        assert_eq!(fuel_for_mass(14), 2);
        assert_eq!(fuel_for_mass(1969), 654);
        assert_eq!(fuel_for_mass(100756), 33583);
    }

    #[test]
    fn test_fuel_for_mass_and_fuel() {
        assert_eq!(fuel_for_mass_and_fuel(14), 2);
        assert_eq!(fuel_for_mass_and_fuel(1969), 966);
        assert_eq!(fuel_for_mass_and_fuel(100756), 50346);
    }

    #[test]
    fn run_part1() {
        part1();
    }

    #[test]
    fn run_part2() {
        part2();
    }
}
