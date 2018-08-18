extern crate clap;
extern crate zip;
#[macro_use]
extern crate failure;

use clap::{Arg, App};

mod error;
mod common;
mod list;
mod pipe;
mod unpack;

use error::Error;
use common::Action;
use list::ListActionInput;
use pipe::PipeUnpackActionInput;
use unpack::UnpackActionInput;

pub fn main() {
    let matches = App::new("unzipr")
        .version("0.4.0")
        .author("Taivo Käsper <taivo.kasper@gmail.com>")
        .about("For in-memory listing and unpacking files from nested zip archives")
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
            .help("An optional directory to which to extract files. By default, all files and subdirectories are recreated in the current directory."))
        .arg(Arg::with_name("files")
            .multiple(true)
            .required(true)
            .min_values(1))
        .get_matches();

    let list = matches.is_present("list");
    let pipe = matches.is_present("pipe");
    let files: Vec<&str> = matches.values_of("files").unwrap().collect();

    let action: Result<Box<Action>, Error>;
    if list {
        action = ListActionInput::new(files.clone());
    } else if pipe {
        action = PipeUnpackActionInput::new(files.clone());
    } else {
        let path_buf = std::env::current_dir().unwrap();
        let dir = if matches.is_present("exdir") {
            matches.value_of("exdir").unwrap()
        } else {
            path_buf.as_path().to_str().unwrap()
        };
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

fn unwrap_process_result(msg: Error) {
    println!("{}", msg);
    std::process::exit(1);
}
