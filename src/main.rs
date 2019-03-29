use std::env;
use std::io;

#[derive(Debug)]
struct ProgramConfig {
    human_readable: bool,
    instructions: bool,
    opcodes: bool,
    registers: bool,
}

enum InsType {
    // RS, RT, RD, SHAM, FUNC
    RType(u8, u8, u8, u8, u8),
    // OP, RS, RT, IMM
    IType(u8, u8, u8, u16),
    // OP, Addr
    JType(u8, u32),
}

fn main() {
    // setup program run config
    let config = parse_args();

    let instructions: Vec<u32> = parse_instructions()
      .expect("Failed to convert instructions, error reading stdin");

    if config.instructions {
        if config.human_readable {
            // print header
            println!("{0: <10}{1: <10}{2: <10}", "TYPE", "COUNT", "PERCENT")
        }
        handle_instructions(&instructions);
    }

    if config.opcodes {
        if config.human_readable {
            // print header
            println!("{0: <10}{1: <10}{2: <10}", "OPCODE", "COUNT", "PERCENT")
        }
        handle_opcodes(&instructions);
    }

    if config.registers {
        if config.human_readable {
            // print header
            println!(
                "{0: <10}{1: <10}{2: <10}{3: <10}{4: <10}",
                "REG", "USE", "R-TYPE", "I-TYPE", "PERCENT"
            )
        }
        // pass through the readable setting so the
        // human register names can be printed
        handle_registers(&instructions, config.human_readable);
    }
}

fn parse_args() -> ProgramConfig {
    // initialize a new configuration struct with all the defaults
    let mut config = ProgramConfig {
        human_readable: false,
        instructions: false,
        opcodes: false,
        registers: false,
    };

    for arg in env::args() {
        // for each argument check if its one we accept

        if arg == "-u" {
            // turn on the human readable headers and data
            config.human_readable = true;
        }

        if arg == "-i" && !config.opcodes && !config.registers {
            // ensure neither of the other flags have been provided yet
            // do instruction statistics
            config.instructions = true;
        }

        if arg == "-o" && !config.instructions && !config.registers {
            // do opcode statistics
            config.opcodes = true;
        }

        if arg == "-r" && !config.opcodes && !config.instructions {
            // do register statistics
            config.registers = true;
        }
    }

    return config;
}

fn parse_instructions() -> Result<Vec<u32>, io::Error> {
    let mut input = String::new(); // mutable buffer
    let mut instructions = vec![]; // mutable vector
    loop {
        // forever read in a line from stdin
        // the '?' is a way of passing the error up to my Result return type
        let bytes = io::stdin().read_line(&mut input)?;

        if bytes != 11 {
            // if we dont get exactly what we expect, we are done
            // EOF or bad string... we can stop here; 10 chars + nl
            return Ok(instructions);
        }

        // 2..input.len()-1 ; Chop off '0x' and '\n'
        let digit_str = &input.as_str()[2..input.len() - 1];

        // parse the string from hex into an unsigned 32 bit value
        let instruction = u32::from_str_radix(digit_str, 16)
        // if failure to parse, crashes with this message
            .expect("Could not parse digits");

        // add the instruction to the mutable vector
        instructions.push(instruction);

        // clean out buffer for the next line of text
        input.clear();
    }
}

/// prints the statistics related to instruction type usage
fn handle_instructions(instructions: &Vec<u32>) {
    // I-Type	333		69.1%
    // J-Type	28		5.8%
    // R-Type	121		25.1%

    let mut r_type = 0;
    let mut j_type = 0;
    let mut i_type = 0;

    for instruction in instructions {
        match instruction_type(&instruction) {
            InsType::IType(_, _, _, _) => {
                i_type += 1;
            },
            InsType::RType(_, _, _, _,_) => {
                r_type += 1;
            },
            InsType::JType(_, _) => {
                j_type += 1;
            }
        }
    }

    println!(
        "{0: <10}{1: <10}{2: <10}",
        "I-Type",
        i_type,
        format!(
            "{:.2}%",
            (i_type as f32 / instructions.len() as f32) * 100.0
        )
    );
    println!(
        "{0: <10}{1: <10}{2: <10}",
        "J-Type",
        j_type,
        format!(
            "{:.2}%",
            (j_type as f32 / instructions.len() as f32) * 100.0
        )
    );
    println!(
        "{0: <10}{1: <10}{2: <10}",
        "R-Type",
        r_type,
        format!(
            "{:.2}%",
            (r_type as f32 / instructions.len() as f32) * 100.0
        )
    );
}

/// prints the statistics related to opcode usage
fn handle_opcodes(instructions: &Vec<u32>) {
    let mut opcode_counts = [0; 0x3F]; // 0x3F zeroes

    for instruction in instructions {
        // For each instruction get the type
        match instruction_type(&instruction) {
            InsType::JType(op, _) => {
                opcode_counts[op as usize] += 1;
            }
            InsType::IType(op, _, _, _) => {
                opcode_counts[op as usize] += 1;
            }
            _ => {}
        }
    }

    for (opcode, count) in opcode_counts.iter().enumerate() {
        println!(
            "{0: <10}{1: <10}{2: <10}",
            format!("0x{:X?}", opcode),
            count,
            format!(
                "{:.2}%",
                (*count as f32 / instructions.len() as f32) * 100.0
            )
        );
    }
}

/// prints the statistics related to register usage
fn handle_registers(instructions: &Vec<u32>, human_readable: bool) {
    // A static array of the human names of registers, in order
    let reg_map = [
        "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
        "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "fp",
        "sp", "ra",
    ];

    let mut reg_count_r_type = [0; 32]; // 32 zeros
    let mut reg_count_i_type = [0; 32]; // 32 zeros

    for instruction in instructions {
        // For each instruction, obtain the InsType Enum
        match instruction_type(instruction) {
            // Matching on r-type means checking for rs, rt, rd
            InsType::RType(rs, rt, rd, _, _) => {
                reg_count_r_type[rs as usize] += 1;
                reg_count_r_type[rt as usize] += 1;
                reg_count_r_type[rd as usize] += 1;
            }
            // Matching on i-type means only checking rs, rt
            InsType::IType(_, rs, rt, _) => {
                reg_count_i_type[rs as usize] += 1;
                reg_count_i_type[rt as usize] += 1;
            }
            // No registers were harmed in these jump instructions
            InsType::JType(_, _) => {}
        }
    }

    let counts = reg_count_r_type
        .iter() // Get an iterator from the r type counts
        .zip(reg_count_i_type.iter()) // zip together the i-type counts so we have nice tuples
        .enumerate(); // further pair up those pairs with their index appearance in the vector

    for (idx, (r_count, i_count)) in counts {
        // we can spread each item in the tuple out in the for loop declaration
        println!(
            "{:<10}{1: <10}{2: <10}{3: <10}{4: <10}",
            // If statements return values, so they can be inlined like so
            if human_readable {
                // grab the human name, format it with a '$'
                format!("${}", reg_map[idx])
            } else {
                format!("0x{:X?}", idx)
            },
            r_count + i_count, // total count
            r_count,           // all r-type usage
            i_count,           // all i-type usage
            format!(
                "{:.2}%",
                // Must cast to get floats from int division
                ((r_count + i_count) as f32 / instructions.len() as f32) * 100.0
            )
        );
    }
}

/// returns the enum representation of the 32-bit mips instruction
fn instruction_type(instruction: &u32) -> InsType {
    // This long boolean is broken out into its own variable for readability
    // is_j_type checks if the top 6 bits are 000010 or 000011
    let is_j_type = (instruction & 0x08_00_00_00) == 0x08_00_00_00
        || (instruction & 0x0C_00_00_00) == 0x0C_00_00_00;

    if is_j_type {
        return InsType::JType((instruction >> 26) as u8, (instruction & 0xFFFF) as u32);
    } else if (instruction & 0xFC_00_00_00) == 0 {
        // Checks that he top 6 bits are zeroes, this means R-type
        return InsType::RType(
            (instruction >> 21) as u8,
            ((instruction >> 16) & 0x1F as u32) as u8,
            ((instruction >> 11) & 0x1F as u32) as u8,
            ((instruction >> 6) & 0x1F as u32) as u8,
            ((instruction) & 0x3F as u32) as u8,
        );
    } else {
        // All other cases are I-Types
        return InsType::IType(
            (instruction >> 26) as u8,
            ((instruction >> 21) & 0x1F as u32) as u8,
            ((instruction >> 16) & 0x1F as u32) as u8,
            (instruction & 0xFFFF) as u16,
        );
    }
}
