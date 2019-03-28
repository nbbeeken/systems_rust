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
    let config = parse_args();

    // eprintln!("DEBUG: {:?}", config);

    let instructions: Vec<u32> = parse_instructions().expect("Failed to convert instructions");

    // eprintln!("DEBUG: {:?}", instructions);

    if config.instructions {
        if config.human_readable {
            println!(
                "{0: <10}{1: <10}{2: <10}",
                "TYPE", "COUNT", "PERCENT"
            )
        }
        handle_instructions(&instructions);
    }

    if config.opcodes {
        if config.human_readable {
            println!(
                "{0: <10}{1: <10}{2: <10}",
                "OPCODE", "COUNT", "PERCENT"
            )
        }
        handle_opcodes(&instructions);
    }

    if config.registers {
        if config.human_readable {
            println!(
                "{0: <10}{1: <10}{2: <10}{3: <10}{4: <10}",
                "REG", "USE", "R-TYPE", "I-TYPE", "PERCENT"
            )
        }
        handle_registers(&instructions, config.human_readable);
    }
}

fn parse_args() -> ProgramConfig {
    let mut config = ProgramConfig {
        human_readable: false,
        instructions: false,
        opcodes: false,
        registers: false,
    };

    for arg in env::args() {
        if arg == "-u" {
            config.human_readable = true;
        }

        if arg == "-i" && !config.opcodes && !config.registers {
            config.instructions = true;
        }

        if arg == "-o" && !config.instructions && !config.registers {
            config.opcodes = true;
        }

        if arg == "-r" && !config.opcodes && !config.instructions {
            config.registers = true;
        }
    }

    return config;
}

fn parse_instructions() -> Result<Vec<u32>, io::Error> {
    let mut input = String::new();
    let mut instructions = vec![];
    loop {
        let bytes = io::stdin().read_line(&mut input)?;

        if bytes != 11 {
            // EOF or bad string... we can stop here; 10 chars + nl
            return Ok(instructions);
        }

        // 2..input.len()-1 ; Chop off '0x' and '\n'
        let digit_str = &input.as_str()[2..input.len() - 1];

        instructions.push(u32::from_str_radix(digit_str, 16).expect("Could not parse digits"));

        input.clear();
    }
}

fn handle_instructions(instructions: &Vec<u32>) {
    // I-Type	333		69.1%
    // J-Type	28		5.8%
    // R-Type	121		25.1%

    let mut r_type = 0;
    let mut j_type = 0;
    let mut i_type = 0;

    for instruction in instructions {
        if (instruction & 0xFC_00_00_00) == 0 {
            r_type += 1;
        } else if (instruction & 0x08_00_00_00) == 0x08_00_00_00
            || (instruction & 0x0C_00_00_00) == 0x0C_00_00_00
        {
            j_type += 1;
        } else {
            i_type += 1;
        }
    }

    println!(
        "{0: <10}{1: <10}{2: <10}",
        "I-Type",
        i_type,
        format!("{:.2}%", (i_type as f32 / instructions.len() as f32) * 100.0)
    );
    println!(
        "{0: <10}{1: <10}{2: <10}",
        "J-Type",
        j_type,
        format!("{:.2}%", (j_type as f32 / instructions.len() as f32) * 100.0)
    );
    println!(
        "{0: <10}{1: <10}{2: <10}",
        "R-Type",
        r_type,
        format!("{:.2}%", (r_type as f32 / instructions.len() as f32) * 100.0)
    );
}

fn handle_opcodes(instructions: &Vec<u32>) {

    let mut opcode_counts = vec![0; 0x3F];

    for instruction in instructions {
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

    for (idx, count) in opcode_counts.iter().enumerate() {
        println!(
            "{0: <10}{1: <10}{2: <10}",
            format!("0x{:X?}", idx),
            count,
            format!("{:.2}%", (*count as f32 / instructions.len() as f32) * 100.0)
        );
    }
}

fn handle_registers(instructions: &Vec<u32>, human_readable: bool) {
    let reg_map = vec![
        "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
        "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "fp",
        "sp", "ra",
    ];

    let mut reg_count_r_type: Vec<usize> = vec![0; 32]; // 32 zeros
    let mut reg_count_i_type: Vec<usize> = vec![0; 32]; // 32 zeros

    for instruction in instructions {
        match instruction_type(instruction) {
            InsType::RType(rs, rt, rd, _, _) => {
                reg_count_r_type[rs as usize] += 1;
                reg_count_r_type[rt as usize] += 1;
                reg_count_r_type[rd as usize] += 1;
            }
            InsType::IType(_, rs, rt, _) => {
                reg_count_i_type[rs as usize] += 1;
                reg_count_i_type[rt as usize] += 1;
            }
            InsType::JType(_, _) => {}
        }
    }

    for (idx, (r_count, i_count)) in reg_count_r_type
        .iter()
        .zip(reg_count_i_type.iter())
        .enumerate()
    {
        println!(
            "{:<10}{1: <10}{2: <10}{3: <10}{4: <10}",
            if human_readable {
                format!("${}", reg_map[idx])
            } else {
                format!("0x{:X?}", idx)
            },
            r_count + i_count,
            r_count,
            i_count,
            format!(
                "{:.2}%",
                ((r_count + i_count) as f32 / instructions.len() as f32) * 100.0
            )
        );
    }
}

fn instruction_type(instruction: &u32) -> InsType {
    let is_i_type = (instruction & 0x08_00_00_00) == 0x08_00_00_00
        || (instruction & 0x0C_00_00_00) == 0x0C_00_00_00;

    if (instruction & 0xFC_00_00_00) == 0 {
        return InsType::RType(
            (instruction >> 21) as u8,
            ((instruction >> 16) & 0x1F as u32) as u8,
            ((instruction >> 11) & 0x1F as u32) as u8,
            ((instruction >> 6) & 0x1F as u32) as u8,
            ((instruction) & 0x3F as u32) as u8,
        );
    } else if is_i_type {
        return InsType::IType(
            (instruction >> 26) as u8,
            ((instruction >> 21) & 0x1F as u32) as u8,
            ((instruction >> 16) & 0x1F as u32) as u8,
            (instruction & 0xFFFF) as u16,
        );
    } else {
        return InsType::JType((instruction >> 26) as u8, (instruction & 0xFFFF) as u32);
    }
}
