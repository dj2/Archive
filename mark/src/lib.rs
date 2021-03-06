#![deny(clippy::all, clippy::pedantic)]

//! Marked provides a library to convert a simple markup format to HTML. The
//! format is similar to Markdown, but does not strictly follow markdown.
//! Specifically, things like indented code blocks are not supported and strong
//! and emphasis are not differentiated by the number of markers.

mod parser;
mod tree;

#[macro_use]
extern crate lazy_static;

use crate::parser::Parser;
use crate::tree::Doc;

#[must_use]
pub fn to_ast(buf: &'_ str) -> Doc<'_> {
    let mut p = Parser::new(buf);
    p.parse()
}

#[must_use]
pub fn to_html(buf: &str) -> String {
    let mut p = Parser::new(buf);
    let doc = p.parse();
    doc.to_string()
}
