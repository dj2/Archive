use pretty_assertions::assert_eq;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn compare(name: &str) {
    let html = format!("tests/fixtures/{}.html", name);
    let md = format!("tests/fixtures/{}.md", name);

    let mut result = String::new();
    File::open(Path::new(&html))
        .unwrap()
        .read_to_string(&mut result)
        .unwrap();
    let result = result.trim_end();

    let mut src = String::new();
    File::open(Path::new(&md))
        .unwrap()
        .read_to_string(&mut src)
        .unwrap();

    println!("{}", src);
    let actual = &mark::to_html(&src);
    assert_eq!(result, actual.trim_end());
}

mod cm;

#[test]
pub fn blockquote() {
    compare("data/blockquote")
}

#[test]
pub fn paragraphs() {
    compare("data/paragraphs")
}

#[test]
pub fn headers() {
    compare("data/headers")
}

#[test]
pub fn thematic_breaks() {
    compare("data/thematic_breaks")
}

#[test]
pub fn fenced_code() {
    compare("data/fenced_code")
}

#[test]
pub fn list() {
    compare("data/list")
}

#[test]
pub fn em() {
    compare("data/em")
}

#[test]
pub fn strong() {
    compare("data/strong")
}

#[test]
pub fn code() {
    compare("data/code")
}
