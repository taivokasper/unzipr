use common::*;
use error::Error;
use std::rc::Rc;

pub struct ListActionInput {
    input_file_name: String,
    nested_file_names: Vec<String>,
}

impl ListActionInput {
    pub fn new(input: Vec<&str>) -> Result<Box<Action>, Error> {
        let (input_file, nested_files) = input.as_slice().split_first().unwrap();

        let action_input = ListActionInput {
            input_file_name: input_file.to_string(),
            nested_file_names: nested_files.iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
        };

        Ok(Box::new(action_input))
    }
}

impl Action for ListActionInput {
    fn exec(&self) -> Result<(), Error> {
        let mut inner_archive = parse_file_to_archive(&self.input_file_name, &self.nested_file_names)?;
        for file_name in get_files_list(Rc::get_mut(&mut inner_archive).unwrap())? {
            println!("{}", file_name);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_input_for_list_action() {
        ListActionInput::new(["test_input_file"].to_vec()).unwrap();
    }

    #[test]
    fn test_nested_input_for_list_action() {
        ListActionInput::new(["test_input_file", "inner_nested_file"].to_vec()).unwrap();
    }
}
