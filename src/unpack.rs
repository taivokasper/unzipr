use common::*;

use std;
use std::io;
use std::path::Path;
use std::fs;
use std::rc::Rc;

pub struct UnpackActionInput {
    unpack_target: String,
    input_file_name: String,
    nested_file_names: Vec<String>,
}

impl UnpackActionInput {
    pub fn new(unpack_target: &str, input: Vec<&str>) -> MsgResult<Box<Action>> {
        match input.as_slice().split_first() {
            None => return Err("Not a valid input files argument. Should supply at least one value"),
            Some(split_result) => {
                let (input_file, nested_files) = split_result;

                return Ok(Box::new(UnpackActionInput {
                    unpack_target: unpack_target.to_string(),
                    input_file_name: input_file.to_string(),
                    nested_file_names: nested_files.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                }));
            }
        };
    }
}

impl Action for UnpackActionInput {
    fn exec(&self) -> MsgResult<()> {
        let mut inner_archive = match parse_file_to_archive(&self.input_file_name, &self.nested_file_names) {
            Err(e) => return Err(e),
            Ok(val) => val
        };
        let archive = Rc::get_mut(&mut inner_archive).unwrap();

        for index in 0..archive.len() {
            let mut zip_file = archive.by_index(index).unwrap();
            let out_path = abs_filename(&self.unpack_target, zip_file.name());

            if zip_file.name().ends_with('/') {
                fs::create_dir_all(&out_path).unwrap();
            } else {
                if out_path.exists() {
                    return Err("Target file already exists");
                }
                let parent_dir = out_path.parent().unwrap();
                if !parent_dir.exists() {
                    fs::create_dir_all(&parent_dir).unwrap();
                }
                let mut out_file = fs::File::create(&out_path).unwrap();
                io::copy(&mut zip_file, &mut out_file).unwrap();
            }

            #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;

                    if let Some(mode) = zip_file.unix_mode() {
                        fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)).unwrap();
                    }
                }
        }

        return Ok(());
    }
}

fn abs_filename(base: &String, filename: &str) -> std::path::PathBuf {
    return Path::new(&base)
        .join(filename);
}