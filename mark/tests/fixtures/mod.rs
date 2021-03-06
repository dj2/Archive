use std::fs::File;
use std::io::Read;
use std::path::Path;
use difference::assert_diff;

fn compare(name: &str) {
    let html = format!("tests/fixtures/data/{}.html", name);
    let md = format!("tests/fixtures/data/{}.md", name);

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
    assert_diff!(&mark::to_html(&src), &result, " ", 0);
}

#[test]
pub fn blockquote() {
    compare("blockquote")
}

#[test]
pub fn paragraphs() {
    compare("paragraphs")
}

#[test]
pub fn headers() {
    compare("headers")
}