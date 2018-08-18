use error::{Error, ErrorKind};
use failure::ResultExt;
use std::fs::File;
use std::io::{self, BufWriter, Cursor, Read};
use std::path::Path;
use std::rc::Rc;
use zip::result::ZipError as ZipErrorKind;
use zip::ZipArchive;

pub type ByteArchive = ZipArchive<Cursor<Vec<u8>>>;

pub trait Action {
    fn exec(&self) -> Result<(), Error>;
}

pub fn parse_file_to_archive(input_file_name: &str, nested_file_names: &[String]) -> Result<Rc<ByteArchive>, Error> {
    let archive = new_from_file(Path::new(&input_file_name))?;
    parse_files_rec(Rc::new(archive), &string_vec_to_str_vec(nested_file_names))
}

fn string_vec_to_str_vec(input: &[String]) -> Vec<&str> {
    input.iter().map(|s| s.as_ref()).collect::<Vec<&str>>()
}

fn parse_files_rec(mut archive: Rc<ByteArchive>, rec_files: &[&str]) -> Result<Rc<ByteArchive>, Error> {
    if rec_files.is_empty() {
        Ok(archive)
    } else {
        let source_file = rec_files[0];
        let deep_files = rec_files[1..].to_vec();

        let mut file = Rc::get_mut(&mut archive).unwrap().by_name(source_file)
            .context(ErrorKind::ZipFileEntryNotFound(source_file.to_string()))?;

        let mut buf = Vec::new();
        io::copy(&mut file, &mut BufWriter::new(&mut buf))?;

        let new_archive = new_from_bytes(buf)
            .context(ErrorKind::ZipEntryNotZipArchive(source_file.to_string()))?;
        parse_files_rec(Rc::new(new_archive), &deep_files)
    }
}

fn new_from_file(zip_file_path: &Path) -> Result<ByteArchive, Error> {
    let file_path_str = zip_file_path.to_str().unwrap();

    let mut opened_file: File = File::open(&zip_file_path)
        .context(ErrorKind::DoesNotExist(file_path_str.to_string()))?;

    let mut data = Vec::new();
    opened_file.read_to_end(&mut data)?;

    let zip_file_archive = ZipArchive::new(Cursor::new(data))
        .context(ErrorKind::NotZipArchive(file_path_str.to_string()))?;

    Ok(zip_file_archive)
}

fn new_from_bytes(bytes: Vec<u8>) -> Result<ByteArchive, ZipErrorKind> {
    ZipArchive::new(Cursor::new(bytes))
}

pub fn get_files_list(archive: &mut ByteArchive) -> Result<Vec<String>, Error> {
    let mut name_vec = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        name_vec.push(file.name().to_string())
    }
    Ok(name_vec)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::BufWriter;
    use std::path::Path;
    use std::rc::Rc;
    use super::*;

    #[test]
    fn test_parsing_file_to_archive() {
        let inner_files = Vec::new();
        let mut archive = parse_file_to_archive(&"tests/resources/test.zip".to_string(), &inner_files).unwrap();
        let files_list = get_files_list(Rc::get_mut(&mut archive).unwrap()).unwrap();

        assert_eq!(2, files_list.len());
        assert_eq!("test/", files_list[0]);
        assert_eq!("test/test.txt", files_list[1]);
    }

    #[test]
    fn test_parsing_txt_file_to_archive() {
        let txt_file = "tests/resources/test.txt".to_string();
        let inner_files = Vec::new();
        let archive = parse_file_to_archive(&txt_file, &inner_files);
        assert_eq!(ErrorKind::NotZipArchive(txt_file), archive.err().unwrap().kind());
    }

    #[test]
    fn test_parsing_files_in_nested_zip() {
        let test_archive = new_from_file(Path::new("tests/resources/test-test.zip")).unwrap();
        let nested_archives = vec!["test.zip"];
        let mut archive = parse_files_rec(Rc::new(test_archive), &nested_archives).unwrap();
        let files_list = get_files_list(Rc::get_mut(&mut archive).unwrap()).unwrap();

        assert_eq!(2, files_list.len());
        assert_eq!("test/", files_list[0]);
        assert_eq!("test/test.txt", files_list[1]);
    }

    #[test]
    fn test_new_archive_from_file() {
        let archive = new_from_file(Path::new("tests/resources/test.zip"));
        assert_eq!(2, archive.unwrap().len());
    }

    #[test]
    fn test_new_archive_from_nonexistent_file() {
        let non_existent_file = "tests/resources/does-not-exist.zip";
        let archive = new_from_file(Path::new(non_existent_file));
        assert_eq!(ErrorKind::DoesNotExist(non_existent_file.to_string()), archive.err().unwrap().kind());
    }

    #[test]
    fn test_new_archive_from_nonzip_file() {
        let test_txt_file = "tests/resources/test.txt";
        let archive = new_from_file(Path::new(test_txt_file));
        assert_eq!(ErrorKind::NotZipArchive(test_txt_file.to_string()), archive.err().unwrap().kind());
    }

    #[test]
    fn test_new_archive_from_bytes() {
        let mut opened_file: File = File::open(Path::new("tests/resources/test.zip")).unwrap();

        let mut buf = Vec::new();
        io::copy(&mut opened_file, &mut BufWriter::new(&mut buf)).unwrap();
        let archive = new_from_bytes(buf).unwrap();

        assert_eq!(2, archive.len());
    }

    #[test]
    fn test_get_files_list() {
        let mut archive = new_from_file(Path::new("tests/resources/test.zip")).unwrap();
        let files_list = get_files_list(&mut archive).unwrap();

        assert_eq!(2, files_list.len());
        assert_eq!("test/", files_list[0]);
        assert_eq!("test/test.txt", files_list[1]);
    }
}
