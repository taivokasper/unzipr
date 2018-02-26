use zip::ZipArchive;
use zip::result::ZipError;
use std::io::{BufWriter, Cursor, Read, ErrorKind};
use std::path::Path;
use std::fs::File;
use std::rc::Rc;
use std::io;

pub type ByteArchive = ZipArchive<Cursor<Vec<u8>>>;
pub type MsgResult<T> = Result<T, &'static str>;

pub trait Action {
    fn exec(&self) -> MsgResult<()>;
}

pub fn parse_file_to_archive<'a>(input_file_name: &'a String, nested_file_names: &'a Vec<String>) -> MsgResult<Rc<ByteArchive>> {
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

pub fn get_files_list(archive: &mut ByteArchive) -> Vec<String> {
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
