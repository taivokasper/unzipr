extern crate clap;
extern crate zip;

use std::io;
use clap::{Arg, App};
use std::path::Path;
use std::fs::File;
use zip::ZipArchive;
use std::io::BufWriter;
use std::io::Cursor;

struct FileArchive<'a> {
    file_path: &'a Path
}

impl<'a> FileArchive<'a> {
    fn new(zip_file_path: &'a Path) -> FileArchive {
        return FileArchive { file_path: &zip_file_path }
    }
    fn to_zip_archive(&self) -> ZipArchive<File> {
        let opened_file: File = File::open(&self.file_path).unwrap();
        return ZipArchive::new(opened_file).unwrap();
    }
}

struct BytesArchive {
    bytes: Vec<u8>
}

impl BytesArchive {
    fn new(archive_bytes: Vec<u8>) -> BytesArchive {
        return BytesArchive { bytes: archive_bytes }
    }
    fn to_zip_archive(&self) -> ZipArchive<Cursor<Vec<u8>>> {
        return ZipArchive::new(Cursor::new(self.bytes.clone())).unwrap();
    }
}

fn get_files_list<R: std::io::Read + io::Seek>(archive: &mut ZipArchive<R>) -> Vec<String> {
    let mut name_vec = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        name_vec.push(file.name().to_string())
    }
    return name_vec;
}

#[test]
fn test_get_files_list() {
    let mut test_archive = FileArchive::new(Path::new("tests/resources/test.zip")).to_zip_archive();
    let files_list = get_files_list(&mut test_archive);

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
    let file_archive = FileArchive::new(source_file);

    if list {
        let rec_files = files[1..].to_vec();
        let files_list = list_files_rec(&mut file_archive.to_zip_archive(), &rec_files);
        for file_name in files_list {
            println!("{}", file_name);
        }
    } else {
        println!("Unzip is not implemented yet!")
    }
}

fn list_files_rec<R: std::io::Read + io::Seek>(archive: &mut ZipArchive<R>, rec_files: &Vec<&str>) -> Vec<String> {
    if rec_files.is_empty() {
        return get_files_list(archive);
    } else {
        let source_file = rec_files[0];
        let deep_files = rec_files[1..].to_vec();

        let mut file = archive.by_name(source_file).unwrap();

        let mut buf = Vec::new();
        io::copy(&mut file, &mut BufWriter::new(&mut buf)).unwrap();

        let bytes_archive = BytesArchive::new(buf);
        return list_files_rec(&mut bytes_archive.to_zip_archive(), &deep_files);
    }
}

#[test]
fn test_listing_files_in_nested_zip() {
    let mut test_archive = FileArchive::new(Path::new("tests/resources/test-test.zip")).to_zip_archive();
    let nested_archives = vec!["test.zip"];
    let files_list = list_files_rec(&mut test_archive, &nested_archives);

    assert_eq!(2, files_list.len());
    assert_eq!("test/", files_list[0]);
    assert_eq!("test/test.txt", files_list[1]);
}