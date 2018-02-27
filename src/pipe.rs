use common::*;

use std::io;
use std::io::{BufWriter, Write};
use std::rc::Rc;

const UNPACK_TARGET_MISSING: &str = "Cannot get unpack target file";

pub struct PipeUnpackActionInput {
    input_file_name: String,
    nested_file_names: Vec<String>,
    unpack_target_file: String,
}

impl PipeUnpackActionInput {
    pub fn new(input: Vec<&str>) -> MsgResult<Box<Action>> {
        let (input_file, nested_files) = input.as_slice().split_first().unwrap();

        match nested_files.split_last() {
            None => return Err(UNPACK_TARGET_MISSING),
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
}

#[test]
fn test_single_input_for_pipe_unpack_action() {
    let err = PipeUnpackActionInput::new(["test_input_file"].to_vec()).err().unwrap();
    assert_eq!(UNPACK_TARGET_MISSING, err);
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
        io::stdout().write_all(&buf).unwrap();
        return Ok(());
    }
}
