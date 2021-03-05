use std::fmt;
use std::iter::Iterator;

#[derive(Debug)]
pub struct Doc<'a> {
  blocks: Vec<Block<'a>>,
  open: Vec<Block<'a>>,
  text: Vec<&'a str>,
}
impl<'a> Doc<'a> {
  pub fn new() -> Self {
    Self {
      blocks: Vec::new(),
      open: Vec::new(),
      text: Vec::new(),
    }
  }

  pub fn push(&mut self, b: Block<'a>) {
    // Push any text blocks.
    if !self.text.is_empty() {
      self.push_hardbreak();
    }

    self.blocks.push(b);
  }

  pub fn push_hardbreak(&mut self) {
    if self.text.is_empty() {
      return;
    }

    let t = self.text.iter().map(|n| Inline::Text(n)).collect();
    self.text = vec![];
    self.push(Block::Paragraph(t));
  }

  pub fn push_text(&mut self, s: &'a str) {
    self.text.push(s);
  }

  // pub fn close_open_nodes(&mut self, b: &Block<'a>) {
  //   loop {
  //     if self.open.is_empty() {
  //       break;
  //     }
  //     if let Some(node) = self.open.last() {
  //       if node.is_closed_by(b) {
  //         let n = self.open.pop();

  //       }
  //     } else {
  //       panic!("Open not empty; but no last node");
  //     }
  //   }
  // }
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
impl<'a> Block<'a> {
  // pub fn is_closed_by(&self, b: &Block<'a>) -> bool {
  //   match self {
  //     Block::Blockquote(_) => true,
  //     Block::Paragraph(_) => {
  //       match b {
  //         Block::Blockquote(_) => true,
  //         Block::Paragraph(_) => false,
  //       }
  //     }
  //   }
  // }
}

#[derive(Debug, PartialEq)]
pub enum Inline<'a> {
  Text(&'a str),
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

impl<'a> fmt::Display for Inline<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Inline::Text(s) => write!(f, "{}", s)?,
    };
    Ok(())
  }
}
