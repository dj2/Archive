mod html;
mod parser;
mod tree;

#[macro_use]
extern crate lazy_static;

use crate::html::Html;
use crate::parser::Parser;
use crate::tree::Doc;

pub fn to_ast(buf: &'_ str) -> Doc<'_> {
    let mut p = Parser::new(buf);
    p.parse()
}

pub fn to_html(buf: &str) -> String {
    let mut p = Parser::new(buf);
    let doc = p.parse();
    let html = Html::new(&doc);
    html.to_string()
}
