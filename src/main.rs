use serde_json::{Result, Value};
use clap::Parser;

fn deserialize_json(filename: String) -> Result<Value> {

    let contents = std::fs::read_to_string(filename)
        .expect("Not able to read ion.json");

    let data: Value = serde_json::from_str(&contents);
        .expect("Not able to parse ion.json");

    Ok(data)
}

fn parse_instructions(instructions: &Vec<Value>) -> String {

    let mut debugout = String::new();

    // First iteration as a dumb way to find the length of the longest opcode
    // and operand. This will be used to align the output properly. Note that if
    // a graph is large, then this will slow it down, so in that case, just give
    // `opcode_len` and `operand_len` a large value and comment this out.
    let mut opcode_len  = 0;
    let mut operand_len = 0;
    for instr in instructions.into_iter() {
        let instruction = instr["opcode"].as_str().unwrap();
        let (opcode, operand) = match instruction.split_once(" ") {
            Some((opcode, operand)) =>  (opcode, operand),
            None => (instruction, "")
        };

        if opcode.len() > opcode_len {
            opcode_len = opcode.len();
        }

        if operand.len() > operand_len {
            operand_len = operand.len();
        }
    }

    // Now go through each instruction in this block and parse that.
    for instr in instructions.into_iter() {
        let id     = instr["id"].as_u64().unwrap();
        let instruction = instr["opcode"].as_str().unwrap();
        let (opcode, operand) = match instruction.split_once(" ") {
            Some((opcode, operand)) =>  (opcode, operand),
            None => (instruction, "")
        };

        debugout += &format!("          {:>3}: {:<opw$} {:<orw$} {}\n",
                             id, opcode, operand, instr["type"],
                             opw = opcode_len + 5, orw = operand_len + 5);
    }

    debugout
}

fn parse_blocks(blocks: &Vec<Value>) -> String {

    let mut debugout = String::new();

    for block in blocks.into_iter() {
        debugout += &format!("\n      Block#{}\n", block["number"]);

        let instructions = block["instructions"].as_array().unwrap();
        debugout += &parse_instructions(instructions);

        let successors = block["successors"].as_array().unwrap();

        if successors.len() == 1 {
            debugout += &format!("          Successor: Block#{}\n", successors[0]);
        } else if successors.len() == 2 {
            debugout += &format!("          Successors: T:Block#{} F:Block#{}\n",
                                 successors[0], successors[1]);

        } else if successors.len() > 2 {

            let successors = successors.into_iter()
                                       .map(|v| format!("Block#{}", v))
                                       .collect::<Vec<_>>();
            debugout += &format!("Successors: {}\n", successors.join(" "));
        }
    }

    debugout
}

fn parse_passes(passes: &Vec<Value>) -> String {

    let mut debugout = String::new();

    for pass in passes.into_iter() {
        debugout += &format!("\n\n  After Ion Phase {}\n\n", pass["name"]);

        // Fetch the basic blocks in this pass and parse them. We are only
        // looking at MIR code now.
        // TODO: Add support for LIR as well
        let mirblocks = pass["mir"]["blocks"].as_array().unwrap();
        debugout += &parse_blocks(mirblocks);
    }

    debugout
}

/// Simple script to convert the ion.json file into a text based IR form
#[derive(Parser, Debug)]
#[clap(author, about, long_about=None)]
struct Args {

    /// Path of the ion.json file
    #[clap(short, long, value_parser, default_value = "/tmp/ion.json")]
    ionfile: String,

    /// Path of the file where to save the output
    #[clap(short, long, value_parser, default_value = "/tmp/iongraph")]
    outfile: String,

}

fn main() {

    let args = Args::parse();

    // Parse the ion.json file into the program
    let iondata = deserialize_json(args.ionfile);

    // This will hold the output disassembly
    let mut debugout = String::new();

    // Go through all the functions that were ion compiled
    for func in iondata["functions"].as_array().unwrap().into_iter() {
        debugout += &format!("\n\nGraph for Function: {}", func["name"]);

        // Fetch the optimization passes that ran on this function and parse
        // them
        let passes = func["passes"].as_array().unwrap();
        debugout += &parse_passes(passes);
    }

    std::fs::write(args.outfile, debugout)
        .expect("unable to write output");

}
