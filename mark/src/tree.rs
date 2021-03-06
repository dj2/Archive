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
        Self {
            blocks
        }
    }
}
impl<'a> fmt::Display for Doc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for idx in 0..self.blocks.len() {
            if idx > 0 {
                writeln!(f,)?;
            }
            write!(f, "{}", self.blocks[idx].to_string())?;
        }
        Ok(())
    }
}

/// The block level elements in the document.
#[derive(Debug, PartialEq)]
pub enum Block<'a> {
    /// A blockquote containing a set of blocks
    Blockquote(Vec<Block<'a>>),
    /// A header with a given level and set of inline text.
    Header(usize, Vec<Inline<'a>>),
    /// A paragraph with a given set of inline text.
    Paragraph(Vec<Inline<'a>>),
}

impl<'a> fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Block::Blockquote(blocks) => {
                write!(f, "<blockquote>")?;
                for (i, block) in blocks.iter().enumerate() {
                    if i != 0 {
                        writeln!(f,)?;
                    }
                    write!(f, "{}", block.to_string())?;
                }
                write!(f, "</blockquote>")?;
            }
            Block::Header(lvl, inlines) => {
                write!(f, "<h{}>", lvl)?;
                for (i, inline) in inlines.iter().enumerate() {
                    if i != 0 {
                        writeln!(f,)?;
                    }
                    write!(f, "{}", inline.to_string())?;
                }
                write!(f, "</h{}>", lvl)?;
            }
            Block::Paragraph(inlines) => {
                write!(f, "<p>")?;
                for (i, inline) in inlines.iter().enumerate() {
                    if i != 0 {
                        writeln!(f,)?;
                    }
                    write!(f, "{}", inline.to_string())?;
                }
                write!(f, "</p>")?;
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
