use common::*;
use error::{Error, ErrorKind};
use failure::ResultExt;
use std::io::{self, BufWriter, Write};
use std::rc::Rc;

pub struct PipeUnpackActionInput {
    input_file_name: String,
    nested_file_names: Vec<String>,
    unpack_target_file: String,
}

impl PipeUnpackActionInput {
    pub fn new(input: Vec<&str>) -> Result<Box<Action>, Error> {
        let (input_file, nested_files) = input.as_slice().split_first().unwrap();

        let (target_file, middle_files) = nested_files.split_last()
            .ok_or(ErrorKind::UnpackTargetMissing)?;

        let action_input = PipeUnpackActionInput {
            input_file_name: input_file.to_string(),
            nested_file_names: middle_files.iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            unpack_target_file: target_file.to_string(),
        };

        Ok(Box::new(action_input))
    }
}

impl Action for PipeUnpackActionInput {
    fn exec(&self) -> Result<(), Error> {
        let mut inner_archive = parse_file_to_archive(&self.input_file_name, &self.nested_file_names)?;
        let mut file = Rc::get_mut(&mut inner_archive).unwrap().by_name(&self.unpack_target_file)
            .context(ErrorKind::ZipFileEntryNotFound(self.unpack_target_file.clone()))?;

        let mut buf = Vec::new();

        io::copy(&mut file, &mut BufWriter::new(&mut buf))?;
        io::stdout().write_all(&buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_input_for_pipe_unpack_action() {
        let err_kind = PipeUnpackActionInput::new(["test_input_file"].to_vec()).err().unwrap().kind();
        assert_eq!(ErrorKind::UnpackTargetMissing, err_kind);
    }

    #[test]
    fn test_nested_input_for_pipe_unpack_action() {
        PipeUnpackActionInput::new(["test_input_file", "inner_nested_file"].to_vec()).unwrap();
    }
}
