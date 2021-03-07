//! The abstract syntax tree for the document. The root of the tree is a `Doc`
//! node, the rest of the tree is made up of `Block` and `Inline` elements.

use std::fmt;

/// Representation of a markdown document.
#[derive(Debug)]
pub struct Doc<'a> {
    blocks: Vec<Block<'a>>,
}
impl<'a> Doc<'a> {
    /// Create a new document with `blocks`
    pub fn new(blocks: Vec<Block<'a>>) -> Self {
        Self { blocks }
    }
}
impl<'a> fmt::Display for Doc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for idx in 0..self.blocks.len() {
            writeln!(f, "{}", self.blocks[idx].to_string())?;
        }
        Ok(())
    }
}

/// The block level elements in the document.
#[derive(Debug, PartialEq)]
pub enum Block<'a> {
    /// A blockquote containing a set of blocks.
    Blockquote(Vec<Block<'a>>),
    /// A code block. Provides an optional language and the text lines.
    Code(Option<&'a str>, Vec<Inline<'a>>),
    /// A header with a given level and set of inline text.
    Header(usize, Vec<Inline<'a>>),
    /// A paragraph with a given set of inline text.
    Paragraph(Vec<Inline<'a>>),
    /// A thematic break.
    ThematicBreak,
}

fn write_inlines<'a>(f: &mut fmt::Formatter, inlines: &[Inline<'a>]) -> fmt::Result {
    // Note, writeln! is not used here because we don't want to inject a newline
    // into the last line of output.
    for (i, inline) in inlines.iter().enumerate() {
        if i != 0 {
            writeln!(f,)?;
        }
        let s = inline.to_string();
        let strs: Vec<&str> = s.split_whitespace().collect();
        write!(f, "{}", strs.join(" "))?;
    }
    Ok(())
}

impl<'a> fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Block::Blockquote(blocks) => {
                write!(f, "<blockquote>")?;
                for block in blocks.iter() {
                    write!(f, "{}", block.to_string())?;
                }
                writeln!(f, "</blockquote>")?;
            }
            Block::Code(lang, inlines) => {
                write!(f, "<pre><code")?;
                if let Some(lang) = lang {
                    write!(f, " class='language-{}'", lang)?;
                }
                write!(f, ">")?;
                // This does not use the generic `write_inlines` method because
                // the the generic method trims and collapses whitespace which
                // is not desired for code blocks.
                //
                // writeln! is not used here because we don't want a newline on
                // the last line of text.
                for (i, inline) in inlines.iter().enumerate() {
                    if i != 0 {
                        writeln!(f,)?;
                    }
                    write!(f, "{}", inline.to_string())?;
                }
                writeln!(f, "</code></pre>")?;
            }
            Block::Header(lvl, inlines) => {
                write!(f, "<h{}>", lvl)?;
                write_inlines(f, inlines)?;
                writeln!(f, "</h{}>", lvl)?;
            }
            Block::Paragraph(inlines) => {
                write!(f, "<p>")?;
                write_inlines(f, inlines)?;
                writeln!(f, "</p>")?;
            }
            Block::ThematicBreak => {
                writeln!(f, "<hr />")?;
            }
        };
        Ok(())
    }
}

/// Inline elements in the document.
#[derive(Debug, PartialEq)]
pub enum Inline<'a> {
    /// Text content.
    Text(&'a str),
}
impl<'a> fmt::Display for Inline<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inline::Text(s) => write!(f, "{}", s)?,
        };
        Ok(())
    }
}
