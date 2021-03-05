mod html;
mod parser;
mod tree;

use crate::html::Html;
use crate::parser::Parser;

pub fn to_html(buf: &str) -> String {
    let p = Parser::new(buf);
    let doc = p.parse();
    println!("{:?}", doc);
    let html = Html::new(&doc);
    html.to_string()
}
