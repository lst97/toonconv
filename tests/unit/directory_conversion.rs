use tempfile::TempDir;
use std::fs::File;
use std::io::Write;
use toonconv::parser::directory as parser_dir;
use std::path::PathBuf;

#[test]
fn test_find_json_files_nonrecursive() {
    let td = TempDir::new().unwrap();
    let a = td.path().join("a.json");
    let mut fa = File::create(&a).unwrap();
    write!(fa, "{{\"name\": \"A\"}}\n").unwrap();

    let files = parser_dir::find_json_files(&PathBuf::from(td.path()), false).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn test_find_json_files_recursive() {
    let td = TempDir::new().unwrap();
    let sub = td.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();

    let a = td.path().join("a.json");
    let mut fa = File::create(&a).unwrap();
    write!(fa, "{{\"name\": \"A\"}}\n").unwrap();

    let b = sub.join("b.json");
    let mut fb = File::create(&b).unwrap();
    write!(fb, "{{\"name\": \"B\"}}\n").unwrap();

    let files = parser_dir::find_json_files(&PathBuf::from(td.path()), true).unwrap();
    assert!(files.len() >= 2);
}
