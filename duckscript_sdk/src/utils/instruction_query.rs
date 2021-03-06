use duckscript::types::instruction::{Instruction, InstructionType};
use std::cmp::min;

#[cfg(test)]
#[path = "./instruction_query_test.rs"]
mod instruction_query_test;

#[derive(Debug, Clone)]
pub(crate) struct Positions {
    pub(crate) middle: Vec<usize>,
    pub(crate) end: usize,
}

fn get_start(start: Option<usize>) -> usize {
    match start {
        Some(value) => value,
        None => 0,
    }
}

fn get_end(end: Option<usize>, instructions: &Vec<Instruction>) -> usize {
    match end {
        Some(value) => min(instructions.len(), value),
        None => instructions.len(),
    }
}

pub(crate) fn find_commands(
    instructions: &Vec<Instruction>,
    start_names: &Vec<String>,
    middle_names: &Vec<String>,
    end_names: &Vec<String>,
    start: Option<usize>,
    end: Option<usize>,
    allow_recursive: bool,
    start_blocks: &Vec<String>,
    end_blocks: &Vec<String>,
) -> Result<Option<Positions>, String> {
    if start_names.is_empty() || end_names.is_empty() {
        Err("No command names/aliases provided for search.".to_string())
    } else {
        let start_index = get_start(start);
        let end_index = get_end(end, instructions);

        let mut positions = Positions {
            middle: vec![],
            end: 0,
        };
        let mut skip_to = start_index;
        let mut block_delta = 0;
        for line in start_index..end_index {
            if line >= skip_to {
                let instruction = &instructions[line];

                match instruction.instruction_type {
                    InstructionType::Script(ref script_instruction) => {
                        match script_instruction.command {
                            Some(ref command) => {
                                if start_blocks.contains(command) {
                                    block_delta = block_delta + 1;
                                } else if middle_names.contains(command) {
                                    positions.middle.push(line);
                                } else if end_blocks.contains(command) && block_delta > 0 {
                                    block_delta = block_delta - 1;
                                } else if end_names.contains(command) {
                                    positions.end = line;
                                    return Ok(Some(positions));
                                } else if start_names.contains(command) {
                                    if allow_recursive {
                                        match find_commands(
                                            instructions,
                                            start_names,
                                            middle_names,
                                            end_names,
                                            Some(line + 1),
                                            Some(end_index),
                                            allow_recursive,
                                            start_blocks,
                                            end_blocks,
                                        ) {
                                            Ok(positions_options) => match positions_options {
                                                Some(sub_positions) => {
                                                    skip_to = sub_positions.end + 1;
                                                    ()
                                                }
                                                None => return Err(format!("Unsupported nested structure: {} found but end of structure not found.",command).to_string()),
                                            },
                                            Err(error) => return Err(error),
                                        };
                                    } else {
                                        return Err(format!(
                                            "Unsupported nested structure: {} found.",
                                            command
                                        )
                                        .to_string());
                                    }
                                }

                                ()
                            }
                            None => (),
                        }
                    }
                    _ => (),
                }
            }
        }

        Err(format!(
            "Missing end of structure for start names: {:?} start: {} end: {}",
            &start_names, start_index, end_index
        )
        .to_string())
    }
}
