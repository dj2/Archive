use crate::tree::{Doc, Inline, Block};

pub struct Parser<'a> {
    buf: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(buf: &'a str) -> Self {
        Self { buf }
    }

    pub fn parse(&self) -> Doc<'a> {
      let mut doc = Doc::new();

      let lines: Vec<&'a str> = self.buf.lines().collect();
      let mut idx = 0;
      while idx < lines.len() {
        if lines[idx].trim().is_empty() {
          doc.push_hardbreak();
        } else if let Some((block, consumed)) = self.parse_blockquote(&lines, idx) {
          doc.push(block);
          idx += consumed;
          continue;
        } else {
          doc.push_text(&lines[idx]);
        }
        idx += 1;
      }
      doc.push_hardbreak();
      doc
    }

    fn parse_blockquote(&self, lines: &Vec<&'a str>, idx: usize) -> Option<(Block<'a>, usize)> {
      let mut consumed = 0;
      let mut content: Vec<Inline> = vec![];

      while idx + consumed < lines.len() {
        if !lines[idx + consumed].starts_with("> ") {
          break;
        }
        // Strip the '> ' from the start of the blockquote.
        let slice: &'a str = &lines[idx + consumed][2..];
        content.push(Inline::Text(&slice));
        consumed += 1;
      }
      if consumed > 0 {
        return Some((Block::Blockquote(
            vec![Block::Paragraph(content)]), consumed));
      }
      None
    }
}
