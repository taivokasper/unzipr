extern crate clap;
extern crate zip;

use std::io;
use clap::{Arg, App};
use zip::ZipArchive;
use zip::result::ZipError;
use std::io::{BufWriter, Cursor, Read, Write, ErrorKind};
use std::path::Path;
use std::fs;
use std::fs::File;
use std::rc::Rc;

type ByteArchive = ZipArchive<Cursor<Vec<u8>>>;
type MsgResult<T> = Result<T, &'static str>;

fn main() {
    let matches = App::new("unzipr")
        .version("0.2.0")
        .author("Taivo Käsper <taivo.kasper@gmail.com>")
        .about("An unzip library for unzipping a file from zip of zip of zip files")
        .arg(Arg::with_name("list")
            .short("l")
            .long("list")
            .required(false)
            .takes_value(false)
            .help("list files instead of unpacking"))
        .arg(Arg::with_name("pipe")
            .short("p")
            .long("pipe")
            .required(false)
            .takes_value(false)
            .help("extract files to pipe, no messages"))
        .arg(Arg::with_name("exdir")
            .short("d")
            .long("exdir")
            .required(false)
            .takes_value(true)
            .help("An  optional directory to which to extract files. By default, all files and subdirectories are recreated in the current directory."))
        .arg(Arg::with_name("files")
            .multiple(true)
            .required(true)
            .min_values(1))
        .get_matches();

    let list = matches.is_present("list");
    let pipe = matches.is_present("pipe");
    let files: Vec<&str> = matches.values_of("files").unwrap().collect();

    let action: MsgResult<Box<Action>>;
    if list {
        action = ListActionInput::new(files.clone());
    } else if pipe {
        action = PipeUnpackActionInput::new(files.clone());
    } else {
        let path_buf = std::env::current_dir().unwrap();
        let mut dir = path_buf.as_path().to_str().unwrap();
        if matches.is_present("exdir") {
            dir = matches.value_of("exdir").unwrap();
        }
        action = UnpackActionInput::new(dir, files.clone());
    }

    match action {
        Ok(a) => match a.exec() {
            Ok(_) => (),
            Err(msg) => unwrap_process_result(msg)
        },
        Err(msg) => unwrap_process_result(msg)
    };
}

fn unwrap_process_result(msg: &'static str) {
    println!("{}", msg);
    std::process::exit(1);
}

trait Action {
    fn exec(&self) -> MsgResult<()>;
}

struct ListActionInput {
    input_file_name: String,
    nested_file_names: Vec<String>
}

impl ListActionInput {
    fn new(input: Vec<&str>) -> MsgResult<Box<Action>> {
        match input.as_slice().split_first() {
            None => return Err("Not a valid input files argument. Should supply at least one value"),
            Some(split_result) => {
                let (input_file, nested_files) = split_result;

                return Ok(Box::new(ListActionInput {
                    input_file_name: input_file.to_string(),
                    nested_file_names: nested_files.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
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

struct PipeUnpackActionInput {
    input_file_name: String,
    nested_file_names: Vec<String>,
    unpack_target_file: String
}

impl PipeUnpackActionInput {
    fn new(input: Vec<&str>) -> MsgResult<Box<Action>> {
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
                            unpack_target_file: target_file.to_string()
                        }));
                    }
                }
            }
        };
    }
}

struct UnpackActionInput {
    unpack_target: String,
    input_file_name: String,
    nested_file_names: Vec<String>
}

impl UnpackActionInput {
    fn new(unpack_target: &str, input: Vec<&str>) -> MsgResult<Box<Action>> {
        match input.as_slice().split_first() {
            None => return Err("Not a valid input files argument. Should supply at least one value"),
            Some(split_result) => {
                let (input_file, nested_files) = split_result;

                return Ok(Box::new(UnpackActionInput {
                    unpack_target: unpack_target.to_string(),
                    input_file_name: input_file.to_string(),
                    nested_file_names: nested_files.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
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

fn new_from_file(zip_file_path: &Path) -> MsgResult<ByteArchive> {
    let mut opened_file: File = match File::open(&zip_file_path) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err("Input file does not exist"),
            kind => panic!("Unable to read input file: {:?}", kind)
        }
    };

    let mut data = Vec::new();
    opened_file.read_to_end(&mut data).unwrap();

    return match ZipArchive::new(Cursor::new(data)) {
        Ok(za) => Ok(za),
        Err(ZipError::InvalidArchive(_)) => Err("File is not a zip file"),
        Err(err) => panic!(err)
    };
}
#[test]
fn test_new_archive_from_file() {
    let archive = new_from_file(Path::new("tests/resources/test.zip"));
    assert_eq!(2, archive.unwrap().len());
}
#[test]
fn test_new_archive_from_nonexistent_file() {
    let archive = new_from_file(Path::new("tests/resources/does-not-exist.zip"));
    assert_eq!("Input file does not exist", archive.err().unwrap());
}
#[test]
fn test_new_archive_from_nonzip_file() {
    let archive = new_from_file(Path::new("tests/resources/test.txt"));
    assert_eq!("File is not a zip file", archive.err().unwrap());
}

fn new_from_bytes(bytes: Vec<u8>) -> ByteArchive {
    return ZipArchive::new(Cursor::new(bytes)).unwrap();
}
#[test]
fn test_new_archive_from_bytes() {
    let mut opened_file: File = File::open(Path::new("tests/resources/test.zip")).unwrap();

    let mut buf = Vec::new();
    io::copy(&mut opened_file, &mut BufWriter::new(&mut buf)).unwrap();
    let archive = new_from_bytes(buf);

    assert_eq!(2, archive.len());
}

fn get_files_list(archive: &mut ByteArchive) -> Vec<String> {
    let mut name_vec = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        name_vec.push(file.name().to_string())
    }
    return name_vec;
}

#[test]
fn test_get_files_list() {
    let mut archive = new_from_file(Path::new("tests/resources/test.zip")).unwrap();
    let files_list = get_files_list(&mut archive);

    assert_eq!(2, files_list.len());
    assert_eq!("test/", files_list[0]);
    assert_eq!("test/test.txt", files_list[1]);
}

fn parse_file_to_archive<'a>(input_file_name: &'a String, nested_file_names: &'a Vec<String>) -> MsgResult<Rc<ByteArchive>> {
    let archive = match new_from_file(Path::new(&input_file_name)) {
        Err(e) => return Err(e),
        Ok(val) => val
    };
    return parse_files_rec(Rc::new(archive), &string_vec_to_str_vec(&nested_file_names));
}

#[test]
fn test_parsing_file_to_archive() {
    let inner_files = Vec::new();
    let mut archive = parse_file_to_archive(&"tests/resources/test.zip".to_string(), &inner_files).unwrap();
    let files_list = get_files_list(Rc::get_mut(&mut archive).unwrap());

    assert_eq!(2, files_list.len());
    assert_eq!("test/", files_list[0]);
    assert_eq!("test/test.txt", files_list[1]);
}

#[test]
fn test_parsing_txt_file_to_archive() {
    let inner_files = Vec::new();
    let archive = parse_file_to_archive(&"tests/resources/test.txt".to_string(), &inner_files);
    assert_eq!("File is not a zip file", archive.err().unwrap());
}

fn string_vec_to_str_vec<'a>(input: &'a Vec<String>) -> Vec<&'a str> {
    return input.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
}

fn parse_files_rec(mut archive: Rc<ByteArchive>, rec_files: &Vec<&str>) -> MsgResult<Rc<ByteArchive>> {
    if rec_files.is_empty() {
        return Ok(archive);
    } else {
        let source_file = rec_files[0];
        let deep_files = rec_files[1..].to_vec();

        let mut file = Rc::get_mut(&mut archive).unwrap().by_name(source_file).unwrap();

        let mut buf = Vec::new();
        io::copy(&mut file, &mut BufWriter::new(&mut buf)).unwrap();

        let new_archive = new_from_bytes(buf);
        return parse_files_rec(Rc::new(new_archive), &deep_files);
    }
}

#[test]
fn test_parsing_files_in_nested_zip() {
    let test_archive = new_from_file(Path::new("tests/resources/test-test.zip")).unwrap();
    let nested_archives = vec!["test.zip"];
    let mut archive = parse_files_rec(Rc::new(test_archive), &nested_archives).unwrap();
    let files_list = get_files_list(Rc::get_mut(&mut archive).unwrap());

    assert_eq!(2, files_list.len());
    assert_eq!("test/", files_list[0]);
    assert_eq!("test/test.txt", files_list[1]);
}