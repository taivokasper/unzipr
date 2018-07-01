use common::*;

use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::path::Component;
use std::fs;
use std::rc::Rc;

pub struct UnpackActionInput {
    unpack_target: String,
    input_file_name: String,
    nested_file_names: Vec<String>,
}

impl UnpackActionInput {
    pub fn new(unpack_target: &str, input: Vec<&str>) -> MsgResult<Box<Action>> {
        let (input_file, nested_files) = input.as_slice().split_first().unwrap();
        return Ok(Box::new(UnpackActionInput {
            unpack_target: unpack_target.to_string(),
            input_file_name: input_file.to_string(),
            nested_file_names: nested_files.iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
        }));
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
            let out_path = to_file_path(self.unpack_target.as_str(), zip_file.name());

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

fn to_file_path(base: &str, filename: &str) -> PathBuf {
    let base_path = Path::new(base);
    return base_path.join(sanitize(filename));
}

fn sanitize(path: &str) -> PathBuf {
    return Path::new(path)
        .components()
        // Filter out everything not part of a normal path e.g. ., .. etc.
        .filter(|component| match *component {
            Component::Normal(..) => true,
            _ => false,
        })
        .map(Component::as_os_str)
        .collect::<PathBuf>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs_filename_not_out_of_dir() {
        let file_name = to_file_path("/tmp/test/test2", "../../../etc/passwd");
        assert_eq!(Path::new("/tmp/test/test2/etc/passwd"), file_name);
    }

    #[test]
    fn test_relative_paths_in_zip() {
        let file_name = to_file_path("/tmp/test/test2", "test/test/../test.txt");
        assert_eq!(Path::new("/tmp/test/test2/test/test/test.txt"), file_name);
    }

    #[test]
    fn test_sanitize_removes_parent() {
        let val = sanitize("test/test/../test.txt");
        assert_eq!(Path::new("test/test/test.txt"), val);
    }

    #[test]
    fn test_sanitize_removes_root() {
        let val = sanitize("/test/test.txt");
        assert_eq!(Path::new("test/test.txt"), val);
    }

    #[test]
    fn test_sanitize_removes_current_dir() {
        let val = sanitize("test/./test.txt");
        assert_eq!(Path::new("test/test.txt"), val);
    }
}
