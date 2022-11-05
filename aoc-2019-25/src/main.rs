use std::fs;

mod computer;
use computer::Computer;

fn main() {
    let program = fs::read_to_string("input.txt").expect("Can't find input.txt");
    let mut computer = Computer::new(Computer::parse_program(&program));

    // I started out just playing it like a zork game, but it didn't
    // seem like there was logic or cleverness to figure out which
    // items are required to pass through the exit, so I brute forced
    // it.
    let input = "\
    south\n\
    east\n\
    take whirled peas\n\
    west\n\
    north\n\
    north\n\
    east\n\
    take ornament\n\
    north\n\
    north\n\
    take dark matter\n\
    south\n\
    south\n\
    west\n\
    west\n\
    west\n\
    take candy cane\n\
    west\n\
    west\n\
    take tambourine\n\
    east\n\
    east\n\
    east\n\
    north\n\
    take astrolabe\n\
    east\n\
    take hologram\n\
    east\n\
    take klein bottle\n\
    west\n\
    south\n\
    west\n\
    ";

    let items = [
        "astrolabe",
        "candy cane",
        "dark matter",
        "hologram",
        "klein bottle",
        "ornament",
        "tambourine",
        "whirled peas",
    ];

    let drop_all: String = items.iter().map(|s| format!("drop {}\n", s)).collect();
    let drop_all = Computer::ascii_to_intcodes(&drop_all);

    computer.buffer_inputs(Computer::ascii_to_intcodes(input));

    computer.start();
    _ = computer.consume_output_buffer();

    for n in 1..(2_u32.pow(items.len() as u32)) {
        computer.buffer_inputs(drop_all.clone());
        computer.resume();
        _ = computer.consume_output_buffer();

        let mut take = String::new();
        for i in 0..items.len() {
            if 0 != n & (1 << i) {
                take += &format!("take {}\n", items[i]);
            }
        }
        take += "inv\n";
        take += "north\n";
        computer.buffer_inputs(Computer::ascii_to_intcodes(&take));

        computer.resume();
        let output = Computer::intcodes_to_ascii(computer.consume_output_buffer());
        if 0 == output.matches("Alert! Droids on this ship are").count() {
            println!("{}", output);
            break;
        }
    }

    // loop { 
    //     computer.start_or_resume();
    //     let output = computer.consume_output_buffer();
    //     let output = Computer::intcodes_to_ascii(output);
    //     println!("{}", output);

    //     let mut command = String::new();
    //     std::io::stdin()
    //         .read_line(&mut command)
    //         .expect("Failed to read line");

    //     computer.buffer_inputs(Computer::ascii_to_intcodes(&command));
    // }
}
