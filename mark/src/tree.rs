use std::fmt;

#[derive(Debug)]
pub struct Doc<'a> {
    pub blocks: Vec<Block<'a>>,
}
impl<'a> fmt::Display for Doc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for idx in 0..self.blocks.len() {
            if idx > 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", self.blocks[idx].to_string())?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Block<'a> {
    Blockquote(Vec<Block<'a>>),
    Paragraph(Vec<Inline<'a>>),
}

impl<'a> fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Block::Blockquote(blocks) => {
                write!(f, "<blockquote>")?;
                for i in 0..blocks.len() {
                    if i != 0 {
                        write!(f, "\n")?;
                    }
                    write!(f, "{}", blocks[i].to_string())?;
                }
                write!(f, "</blockquote>")?;
            }
            Block::Paragraph(inlines) => {
                write!(f, "<p>")?;
                for i in 0..inlines.len() {
                    if i != 0 {
                        write!(f, "\n")?;
                    }
                    write!(f, "{}", inlines[i].to_string())?;
                }
                write!(f, "</p>")?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Inline<'a> {
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
