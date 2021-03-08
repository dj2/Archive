//! The abstract syntax tree for the document. The root of the tree is a `Doc`
//! node, the rest of the tree is made up of `Block` elements.

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Marker {
    Bullet,
    Dash,
    Plus,
    UpperAlpha,
    LowerAlpha,
    UpperRoman,
    LowerRoman,
    Numeric,
}

/// The block level elements in the document.
#[derive(Debug, PartialEq)]
pub enum Block<'a> {
    /// A blockquote containing a set of blocks.
    Blockquote(Vec<Block<'a>>),
    /// A code block. Provides an optional language and the text lines.
    Code(Option<&'a str>, Vec<Block<'a>>),
    /// A header with a given level and set of inline text.
    Header(usize, Vec<Block<'a>>),
    List(Marker, u32 /* start */, Vec<Block<'a>>),
    ListElement(Vec<Block<'a>>),
    /// A paragraph with a given set of inline text.
    Paragraph(Vec<Block<'a>>),
    /// A thematic break.
    ThematicBreak,
    /// A text block
    Text(&'a str),
    /// A raw text block. No formatting is done on the output
    RawText(&'a str),
    /// An emphasis block
    Emphasis(Vec<Block<'a>>),
}

fn write_blocks<'a>(f: &mut fmt::Formatter, blocks: &[Block<'a>]) -> fmt::Result {
    for block in blocks.iter() {
        write!(f, "{}", block.to_string())?;
    }
    Ok(())
}

impl<'a> fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Block::Blockquote(blocks) => {
                write!(f, "<blockquote>")?;
                write_blocks(f, blocks)?;
                writeln!(f, "</blockquote>")?;
            }
            Block::Code(lang, lines) => {
                write!(f, "<pre><code")?;
                if let Some(lang) = lang {
                    write!(f, " class='language-{}'", lang)?;
                }
                write!(f, ">")?;
                write_blocks(f, lines)?;
                writeln!(f, "</code></pre>")?;
            }
            Block::Header(lvl, content) => {
                write!(f, "<h{}>", lvl)?;
                write_blocks(f, content)?;
                writeln!(f, "</h{}>", lvl)?;
            }
            Block::List(marker, start, blocks) => {
                let (list, attr) = match marker {
                    Marker::Bullet => ("ul", ""),
                    Marker::Dash => ("ul", " style='list-style-type:circle'"),
                    Marker::Plus => ("ul", " style='list-style-type:square'"),
                    Marker::UpperAlpha => ("ol", " type='A'"),
                    Marker::LowerAlpha => ("ol", " type='a'"),
                    Marker::UpperRoman => ("ol", " type='I'"),
                    Marker::LowerRoman => ("ol", " type='i'"),
                    Marker::Numeric => ("ol", ""),
                };
                let mut attr = attr.to_string();
                if *start != 1 {
                    attr = format!("{} start='{}'", attr, *start);
                }
                writeln!(f, "<{}{}>", list, attr)?;
                write_blocks(f, blocks)?;
                writeln!(f, "</{}>", list)?;
            }
            Block::ListElement(blocks) => {
                write!(f, "<li>")?;
                write_blocks(f, blocks)?;
                writeln!(f, "</li>")?;
            }
            Block::Paragraph(blocks) => {
                write!(f, "<p>")?;
                write_blocks(f, blocks)?;
                writeln!(f, "</p>")?;
            }
            Block::Text(txt) => {
                let s = (*txt).to_string();
                let strs: Vec<&str> = s.split_whitespace().collect();
                write!(f, "{}", strs.join(" "))?;
            }
            Block::ThematicBreak => writeln!(f, "<hr />")?,
            Block::RawText(txt) => write!(f, "{}", txt)?,
            Block::Emphasis(blocks) => {
                write!(f, "<em>")?;
                write_blocks(f, blocks)?;
                write!(f, "</em>")?;
            }
        };
        Ok(())
    }
}
