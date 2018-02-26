use common::*;

use std::io;
use std::io::{BufWriter, Write};
use std::rc::Rc;

pub struct PipeUnpackActionInput {
    input_file_name: String,
    nested_file_names: Vec<String>,
    unpack_target_file: String,
}

impl PipeUnpackActionInput {
    pub fn new(input: Vec<&str>) -> MsgResult<Box<Action>> {
        match input.as_slice().split_first() {
            None => return Err("Not a valid input files argument. Should supply at least one value"),
            Some(first_split_result) => {
                let (input_file, nested_files) = first_split_result;
                match nested_files.split_last() {
                    None => return Err("Cannot get unpack target file"),
                    Some(second_split_result) => {
                        let (target_file, middle_files) = second_split_result;

                        return Ok(Box::new(PipeUnpackActionInput {
                            input_file_name: input_file.to_string(),
                            nested_file_names: middle_files.iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>(),
                            unpack_target_file: target_file.to_string(),
                        }));
                    }
                }
            }
        };
    }
}

#[test]
fn test_empty_input_for_pipe_unpack_action() {
    let err = PipeUnpackActionInput::new(Vec::new()).err().unwrap();
    assert_eq!("Not a valid input files argument. Should supply at least one value", err);
}

#[test]
fn test_single_input_for_pipe_unpack_action() {
    let err = PipeUnpackActionInput::new(["test_input_file"].to_vec()).err().unwrap();
    assert_eq!("Cannot get unpack target file", err);
}

#[test]
fn test_nested_input_for_pipe_unpack_action() {
    PipeUnpackActionInput::new(["test_input_file", "inner_nested_file"].to_vec()).unwrap();
}

impl Action for PipeUnpackActionInput {
    fn exec(&self) -> MsgResult<()> {
        let mut inner_archive = match parse_file_to_archive(&self.input_file_name, &self.nested_file_names) {
            Err(e) => return Err(e),
            Ok(val) => val
        };
        let mut file = Rc::get_mut(&mut inner_archive).unwrap().by_name(self.unpack_target_file.as_ref()).unwrap();

        let mut buf = Vec::new();

        io::copy(&mut file, &mut BufWriter::new(&mut buf)).unwrap();
        io::stdout().write(&buf).unwrap();
        return Ok(());
    }
}
