mod html;
mod parser;

use crate::html::Html;
use crate::parser::Parser;

pub fn to_html(buf: &str) -> String {
    let _ = Parser::new(buf);
    let html = Html::new();

    html.to_string()
}
