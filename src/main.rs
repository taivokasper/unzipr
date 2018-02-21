extern crate clap;
extern crate zip;

use std::io;
use clap::{Arg, App};
use zip::ZipArchive;
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::Path;
use std::fs::File;
use std::rc::Rc;

type ByteArchive = ZipArchive<Cursor<Vec<u8>>>;
type MsgResult<T> = Result<T, &'static str>;

fn main() {
    let matches = App::new("unzipr")
        .version("0.1.0")
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
        unimplemented!("Unpack to current dir not yet implemented");
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
        let mut inner_archive = parse_file_to_archive(&self.input_file_name, &self.nested_file_names);
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
        let mut inner_archive = parse_file_to_archive(&self.input_file_name, &self.nested_file_names);
        let mut file = Rc::get_mut(&mut inner_archive).unwrap().by_name(self.unpack_target_file.as_ref()).unwrap();

        let mut buf = Vec::new();

        io::copy(&mut file, &mut BufWriter::new(&mut buf)).unwrap();
        io::stdout().write(&buf).unwrap();
        return Ok(());
    }
}

fn new_from_file(zip_file_path: &Path) -> ByteArchive {
    let mut opened_file: File = File::open(&zip_file_path).unwrap();

    let mut data = Vec::new();
    opened_file.read_to_end(&mut data).unwrap();

    return ZipArchive::new(Cursor::new(data)).unwrap();
}
#[test]
fn test_new_archive_from_file() {
    let archive = new_from_file(Path::new("tests/resources/test.zip"));
    assert_eq!(2, archive.len());
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
    let mut archive = new_from_file(Path::new("tests/resources/test.zip"));
    let files_list = get_files_list(&mut archive);

    assert_eq!(2, files_list.len());
    assert_eq!("test/", files_list[0]);
    assert_eq!("test/test.txt", files_list[1]);
}

fn parse_file_to_archive<'a>(input_file_name: &'a String, nested_file_names: &'a Vec<String>) -> Rc<ByteArchive> {
    let archive = new_from_file(Path::new(&input_file_name));
    return parse_files_rec(Rc::new(archive), &string_vec_to_str_vec(&nested_file_names));
}

fn string_vec_to_str_vec<'a>(input: &'a Vec<String>) -> Vec<&'a str> {
    return input.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
}

fn parse_files_rec(mut archive: Rc<ByteArchive>, rec_files: &Vec<&str>) -> Rc<ByteArchive> {
    if rec_files.is_empty() {
        return archive;
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
    let test_archive = new_from_file(Path::new("tests/resources/test-test.zip"));
    let nested_archives = vec!["test.zip"];
    let mut archive = parse_files_rec(Rc::new(test_archive), &nested_archives);
    let files_list = get_files_list(Rc::get_mut(&mut archive).unwrap());

    assert_eq!(2, files_list.len());
    assert_eq!("test/", files_list[0]);
    assert_eq!("test/test.txt", files_list[1]);
}