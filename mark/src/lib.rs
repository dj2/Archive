mod html;
mod parser;
mod tree;

use crate::html::Html;
use crate::parser::Parser;

pub fn to_html(buf: &str) -> String {
    let mut p = Parser::new(buf);
    let doc = p.parse();
    let html = Html::new(&doc);
    html.to_string()
}
