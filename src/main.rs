extern crate clap;
extern crate zip;

use std::io;
use clap::{Arg, App};
use std::path::Path;
use std::fs::File;
use zip::{ZipArchive};
use std::io::{Read, BufWriter};
use std::io::Cursor;

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

    if list {
        let source_file = Path::new(files[0]);
        let rec_files = files[1..].to_vec();
        list_files(source_file, &rec_files)
    } else {
        println!("Unzip is not implemented yet!")
    }
}

fn list_files(file_name: &Path, rec_files: &Vec<&str>) {
    let z_file = File::open(&file_name).unwrap();
    let mut archive = ZipArchive::new(z_file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        println!("File in zip is {}", file.name());

        let mut buf = Vec::new();

        io::copy(&mut file, &mut BufWriter::new(&mut buf)).unwrap();
        println!("{:?}", buf);

        let mut archive = ZipArchive::new(Cursor::new(buf)).unwrap();
        list_files_of_files(archive, &rec_files);
    }
}

fn list_files_of_files(mut archive: ZipArchive<Cursor<Vec<u8>>>, rec_files: &Vec<&str>) {
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        println!("File in zip is {}", file.name());
    }
}