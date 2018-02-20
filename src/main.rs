extern crate clap;
extern crate zip;

use std::io;
use clap::{Arg, App};
use std::path::Path;
use std::fs::File;
use zip::ZipArchive;
use std::io::BufWriter;
use std::io::Cursor;
use std::io::Read;
use std::rc::Rc;

type ByteArchive = ZipArchive<Cursor<Vec<u8>>>;

fn new_from_file(zip_file_path: &Path) -> ByteArchive {
    let mut opened_file: File = File::open(&zip_file_path).unwrap();

    let mut data = Vec::new();
    opened_file.read_to_end(&mut data).unwrap();

    return ZipArchive::new(Cursor::new(data)).unwrap();
}

fn new_from_bytes(bytes: Vec<u8>) -> ByteArchive {
    return ZipArchive::new(Cursor::new(bytes)).unwrap();
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
        .arg(Arg::with_name("files")
            .multiple(true)
            .required(true)
            .min_values(1))
        .get_matches();

    let list = matches.is_present("list");
    let files: Vec<&str> = matches.values_of("files").unwrap().collect();

    let source_file = Path::new(files[0]);
    let archive = new_from_file(source_file);

    if list {
        let rec_files = files[1..].to_vec();
        let mut inner_archive = parse_files_rec(Rc::new(archive), &rec_files);
        for file_name in get_files_list(Rc::get_mut(&mut inner_archive).unwrap()) {
            println!("{}", file_name);
        }
    } else {
        println!("Unzip is not implemented yet!")
    }
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