extern crate clap;
extern crate zip;

use std::io;
use clap::{Arg, App};
use std::path::Path;
use std::error::Error;
use std::fs::File;
use zip::ZipArchive;
use zip::result::ZipError;
use std::io::BufWriter;
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

        let z_file: File = match File::open(&source_file) {
            Err(why) => {
                println!("Can not open {:?} : {}", &source_file, why.description());
                return;
            },
            Ok(file) => file
        };

        let mut archive = match ZipArchive::new(z_file) {
            Ok(archive) => archive,
            Err(e) => {
                println!("Unable to list contents for file {:?} because: {}", source_file, e);
                return;
            }
        };

        list_files_rec(&mut archive, &rec_files)
    } else {
        println!("Unzip is not implemented yet!")
    }
}

fn list_files_rec<R: std::io::Read + io::Seek>(archive: &mut ZipArchive<R>, rec_files: &Vec<&str>) {
    if rec_files.is_empty() {
        list_files_in_archive(archive);
    } else {
        let source_file = rec_files[0];
        let deep_files = rec_files[1..].to_vec();

        let print_help: bool;
        {
            let file_result = archive.by_name(source_file);
            print_help = match file_result {
                Ok(mut file) => {
                    let mut buf = Vec::new();
                    io::copy(&mut file, &mut BufWriter::new(&mut buf)).unwrap();

                    match ZipArchive::new(Cursor::new(buf)) {
                        Ok(mut a) => list_files_rec(&mut a, &deep_files),
                        Err(e) => println!("Unable to list contents for file {} because: {}", source_file, e)
                    }
                    false
                },
                Err(ZipError::FileNotFound) => {
                    true
                },
                Err(e) => {
                    println!("Couldn't read entry {} because: {}", source_file, e);
                    false
                }
            };
        }

        if print_help {
            println!("Couldn't find {}. Did you mean any of these:", source_file);
            list_files_in_archive(archive);
        }
    }
}

fn list_files_in_archive<R: std::io::Read + io::Seek>(archive: &mut ZipArchive<R>) {
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        println!("{}", file.name());
    }
}