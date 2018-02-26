use common::*;

use std::rc::Rc;

pub struct ListActionInput {
    input_file_name: String,
    nested_file_names: Vec<String>,
}

impl ListActionInput {
    pub fn new(input: Vec<&str>) -> MsgResult<Box<Action>> {
        match input.as_slice().split_first() {
            None => return Err("Not a valid input files argument. Should supply at least one value"),
            Some(split_result) => {
                let (input_file, nested_files) = split_result;

                return Ok(Box::new(ListActionInput {
                    input_file_name: input_file.to_string(),
                    nested_file_names: nested_files.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                }));
            }
        };
    }
}

#[test]
fn test_empty_input_for_list_action() {
    let err = ListActionInput::new(Vec::new()).err().unwrap();
    assert_eq!("Not a valid input files argument. Should supply at least one value", err);
}

#[test]
fn test_single_input_for_list_action() {
    ListActionInput::new(["test_input_file"].to_vec()).unwrap();
}

#[test]
fn test_nested_input_for_list_action() {
    ListActionInput::new(["test_input_file", "inner_nested_file"].to_vec()).unwrap();
}

impl Action for ListActionInput {
    fn exec(&self) -> MsgResult<()> {
        let mut inner_archive = match parse_file_to_archive(&self.input_file_name, &self.nested_file_names) {
            Err(e) => return Err(e),
            Ok(val) => val
        };
        for file_name in get_files_list(Rc::get_mut(&mut inner_archive).unwrap()) {
            println!("{}", file_name);
        }
        return Ok(());
    }
}
