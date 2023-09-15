use std::env;
use std::fs::File;
use std::io::Read;

#[derive(Clone)]
enum Commands {
    IncPointer,
    DecPointer,
    IncByte,
    DecByte,
    WriteByte,
    ReadByte,
    LoopBegin,
    LoopEnd,
}

impl Commands {
    pub fn from_string(source: String) -> Vec<Self> {
        let mut parsed_commands = Vec::new();

        for ident in source.chars() {
            match ident {
                '>' => parsed_commands.push(Commands::IncPointer),
                '<' => parsed_commands.push(Commands::DecPointer),
                '+' => parsed_commands.push(Commands::IncByte),
                '-' => parsed_commands.push(Commands::DecByte),
                '.' => parsed_commands.push(Commands::WriteByte),
                ',' => parsed_commands.push(Commands::ReadByte),
                '[' => parsed_commands.push(Commands::LoopBegin),
                ']' => parsed_commands.push(Commands::LoopEnd),
                _ => {}
            }
        }

        parsed_commands
    }
}

#[derive(Clone)]
enum Instructions {
    Operation(Commands),
    Loop(Vec<Instructions>),
}

impl Instructions {
    pub fn from_commands(commands: Vec<Commands>) -> Vec<Self> {
        let mut instructions = Vec::new();
        let mut loop_nums = 0;
        let mut loop_start = 0;

        for (i, cmd) in commands.iter().enumerate() {
            if loop_nums == 0 {
                match cmd {
                    Commands::IncPointer => {
                        instructions.push(Instructions::Operation(Commands::IncPointer))
                    }
                    Commands::DecPointer => {
                        instructions.push(Instructions::Operation(Commands::DecPointer))
                    }
                    Commands::IncByte => {
                        instructions.push(Instructions::Operation(Commands::IncByte))
                    }
                    Commands::DecByte => {
                        instructions.push(Instructions::Operation(Commands::DecByte))
                    }
                    Commands::WriteByte => {
                        instructions.push(Instructions::Operation(Commands::WriteByte))
                    }
                    Commands::ReadByte => {
                        instructions.push(Instructions::Operation(Commands::ReadByte))
                    }

                    // If no loops are being processed, then we create a new loop 'scope'
                    Commands::LoopBegin => {
                        loop_start = i;
                        loop_nums += 1;
                    }

                    Commands::LoopEnd => panic!("Unmatched ']' at position {}", i),
                }
            } else {
                match cmd {
                    // If we encounter another new loop, add it to the number to process
                    Commands::LoopBegin => loop_nums += 1,

                    // If we encounter a loop end, create a loop instruction
                    Commands::LoopEnd => {
                        // Decrement the number of loops to process
                        loop_nums -= 1;

                        // If this is the outer most loop of this 'scope', then we create a loop instruction
                        if loop_nums == 0 {
                            instructions.push(Instructions::Loop(Instructions::from_commands(
                                // Call from_commands recursively to create the loop's instructions for this 'scope'
                                commands[loop_start + 1..i].to_vec(),
                            )));
                        }
                    }
                    _ => {}
                }
            }
        }

        if loop_nums > 0 {
            panic!("Unmatched '[' at position {}", loop_start);
        }

        instructions
    }
}

fn execute(instr: &Vec<Instructions>) {
    let mut memory = [0u8; 30000];
    let mut pointer: usize = 0;

    execute_inner(instr, &mut pointer, &mut memory);
}

fn execute_inner(instr: &Vec<Instructions>, pointer: &mut usize, memory: &mut [u8; 30000]) {
    let mut infinite_run_guard = 0;

    for instr in instr {
        match instr {
            Instructions::Operation(Commands::IncPointer) => *pointer += 1,
            Instructions::Operation(Commands::DecPointer) => *pointer -= 1,
            Instructions::Operation(Commands::IncByte) => memory[*pointer] += 1,
            Instructions::Operation(Commands::DecByte) => memory[*pointer] -= 1,
            Instructions::Operation(Commands::WriteByte) => print!("{}", memory[*pointer] as char),
            Instructions::Operation(Commands::ReadByte) => {
                let mut input: [u8; 1] = [0; 1];
                std::io::stdin()
                    .read_exact(&mut input)
                    .expect("Failed to read input from stdin");
                memory[*pointer] = input[0];
            }
            Instructions::Loop(instructions) =>
            {
                #[allow(clippy::while_immutable_condition)]
                while memory[*pointer] != 0 {
                    if infinite_run_guard > 1000000 {
                        panic!("Infinite loop detected");
                    }
                    infinite_run_guard += 1;
                    execute_inner(&instructions, pointer, memory);
                }
            }

            Instructions::Operation(_) => unreachable!(),
        }
    }
}

fn main() {
    // read file from command line
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run --release <file.bf>");
        std::process::exit(1);
    }

    let filename = &args[1];

    let mut file = File::open(filename).expect("File not found");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("Something went wrong reading the file");

    let lexed = Commands::from_string(source);
    let instructions = Instructions::from_commands(lexed);

    execute(&instructions);
}
