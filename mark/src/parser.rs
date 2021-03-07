//! Parser to convert from a string into the tree AST.
//!
//! The parser works in two passes. The first pass builds up an internal
//! representation of `Node` objects for each block in the document. Each node
//! is added to a list of nodes and any sub-nodes are then tracked as indices
//! into the main list. This makes it easy to find things like the current open
//! node without having to pass around mutable references. The first pass leaves
//! any text as text fragments to be processed in the second pass. Any link
//! references found during this pass are recorded to be used in link resolving
//! in the second pass.
//!
//! The second pass passes any inlines like emphasis as well as resolving link
//! targets. The output of the second pass a tree of `Block`s which are the
//! the final representation of the document.
use crate::tree::{Block, Doc, Inline};
use regex::Regex;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Kind<'a> {
    Blockquote,
    Code(Option<&'a str>),
    Doc,
    Header(usize),
    Paragraph,
    ThematicBreak,
}

/// A node holds information about a given block in the document. The node
/// will hold _either_ `blocks` or `text` but not both. These aren't stored as
/// optional as we want to be able to push new entries into the lists.
#[derive(Clone, Debug)]
struct Node<'a> {
    kind: Kind<'a>,
    open: bool,
    blocks: Vec<usize>,
    text: Vec<&'a str>,
}
impl<'a> Node<'a> {
    fn new(kind: Kind<'a>) -> Self {
        Self {
            kind,
            open: true,
            blocks: vec![],
            text: vec![],
        }
    }

    /// Determines if the current node is closed by a node of `kind`.
    fn is_closed_by(&self, kind: Kind) -> bool {
        match self.kind {
            Kind::Doc => false,
            Kind::Blockquote | Kind::Paragraph => !(kind == Kind::Paragraph),
            Kind::Header(_) | Kind::Code(_) | Kind::ThematicBreak => true,
        }
    }

    /// Determines if a blank line in the document will close the current node.
    fn is_closed_by_hardbreak(&self) -> bool {
        self.kind == Kind::Paragraph
    }
}

/// The parser object. Given a string will turn it into a document AST.
pub struct Parser<'a> {
    root: usize,
    nodes: Vec<Node<'a>>,
    buf: &'a str,
}
impl<'a, 'b> Parser<'a> {
    /// Create a new parser for the markdown document `str`.
    pub fn new(buf: &'a str) -> Self {
        Self {
            root: 0,
            nodes: vec![Node::new(Kind::Doc)],
            buf,
        }
    }

    /// Parse the document and generate an AST.
    pub fn parse(&mut self) -> Doc<'a> {
        let lines: Vec<&'a str> = self.buf.lines().collect();
        self.parse_lines(&lines);
        self.build_doc()
    }

    /// Takes the internal node tree and converts to the final AST.
    fn build_doc(&mut self) -> Doc<'a> {
        let mut blocks = vec![];
        for idx in &self.nodes[self.root].blocks {
            blocks.push(self.to_block(*idx));
        }
        Doc::new(blocks)
    }

    /// Converts the node at `idx` into a corresponding block.
    fn to_block(&self, idx: usize) -> Block<'a> {
        match self.nodes[idx].kind {
            Kind::Doc => {
                panic!("Should not call to_block on a document");
            }
            Kind::Code(lang) => {
                assert!(self.nodes[idx].blocks.is_empty());

                let mut inlines = vec![];
                for text in &self.nodes[idx].text {
                    inlines.push(Inline::Text(text));
                }
                Block::Code(lang, inlines)
            }
            Kind::Blockquote => {
                assert!(self.nodes[idx].text.is_empty());

                let mut blocks = vec![];
                for n in &self.nodes[idx].blocks {
                    blocks.push(self.to_block(*n));
                }
                Block::Blockquote(blocks)
            }
            Kind::Header(lvl) => {
                assert!(self.nodes[idx].blocks.is_empty());

                // TODO(dj2): Parse inlines
                let mut inlines = vec![];
                for text in &self.nodes[idx].text {
                    inlines.push(Inline::Text(text));
                }
                Block::Header(lvl, inlines)
            }
            Kind::Paragraph => {
                assert!(self.nodes[idx].blocks.is_empty());

                // TODO(dj2): Parse inlines
                let mut inlines = vec![];
                for text in &self.nodes[idx].text {
                    inlines.push(Inline::Text(text));
                }
                Block::Paragraph(inlines)
            }
            Kind::ThematicBreak => Block::ThematicBreak,
        }
    }

    /// Finds the deepest open node in the tree. Open nodes are always the
    /// last node in a blocks child list, so we just have to check the last entry
    /// to determine if there is a deeper open node.
    ///
    /// The `root` node should never be closed, so this will always return a
    /// valid index.
    fn find_open_node(&self, idx: usize) -> usize {
        let node = self.nodes[idx].blocks.last();
        if let Some(i) = node {
            if self.nodes[*i].open {
                return self.find_open_node(*i);
            }
        }
        idx
    }

    /// Given a node of `kind` find the first open node in which we can append
    /// `kind`. Any open nodes which are closed by `kind` will be marked as
    /// closed.
    fn get_open_parent_for(&mut self, kind: Kind) -> usize {
        loop {
            let i = self.find_open_node(self.root);
            if self.nodes[i].is_closed_by(kind) {
                self.close_node(i);
                continue;
            }
            return i;
        }
    }

    /// Creates a new node of `kind` and adds to the current open node. The
    /// id of the new node is returned.
    fn add_node(&mut self, kind: Kind<'a>) -> usize {
        let parent = self.get_open_parent_for(kind);
        self.nodes.push(Node::new(kind));

        let val = self.nodes.len() - 1;
        self.nodes[parent].blocks.push(val);
        val
    }

    /// Marks node at `idx` as closed.
    fn close_node(&mut self, idx: usize) {
        self.nodes[idx].open = false;
    }

    /// Adds the given `txt` to the node at `idx`.
    fn node_add_text(&mut self, idx: usize, txt: &'a str) {
        self.nodes[idx].text.push(txt);
    }

    /// Returns true if the node `idx` contains text.
    fn node_has_text(&self, idx: usize) -> bool {
        !self.nodes[idx].text.is_empty()
    }

    /// Returns true if the node at `idx` is closed by a hardbreak
    fn node_is_closed_by_hardbreak(&self, idx: usize) -> bool {
        self.nodes[idx].is_closed_by_hardbreak()
    }

    /// Parse the set of `lines` and add to the node tree.
    fn parse_lines(&mut self, lines: &[&'a str]) {
        let mut idx = 0;
        while idx < lines.len() {
            if lines[idx].trim().is_empty() {
                let node_idx = self.find_open_node(self.root);
                if self.node_is_closed_by_hardbreak(node_idx) {
                    self.close_node(node_idx);
                }
                idx += 1;
            } else if self.try_thematic_break(&lines, idx).is_some() {
                idx += 1;
            } else if let Some(consumed) = self.try_fenced_code(&lines, idx) {
                idx += consumed;
            } else if let Some(consumed) = self.try_blockquote(&lines, idx) {
                idx += consumed;
            } else if self.try_header(&lines, idx).is_some() {
                idx += 1;
            } else {
                let mut node_idx = self.find_open_node(self.root);
                if !self.node_has_text(node_idx) {
                    node_idx = self.add_node(Kind::Paragraph);
                }
                self.node_add_text(node_idx, &lines[idx]);
                idx += 1;
            }
        }
    }

    /// Attempt to parse a blockquote in `lines`. If a blockquote is found, then
    /// consume the lines until the end of the blockquote and return the number
    /// of lines consumed.
    ///
    /// Note, unlike markdown, we require each line of the blockquote to start
    /// with a '>'.
    fn try_blockquote(&mut self, lines: &[&'a str], idx: usize) -> Option<usize> {
        let mut consumed = 0;
        let mut sub_lines: Vec<&'a str> = vec![];

        while idx + consumed < lines.len() {
            if !lines[idx + consumed].starts_with("> ") {
                break;
            }
            // Strip the '> ' from the start of the blockquote.
            sub_lines.push(&lines[idx + consumed][2..]);
            consumed += 1;
        }
        if consumed > 0 {
            let _ = self.add_node(Kind::Blockquote);
            self.parse_lines(&sub_lines);
            return Some(consumed);
        }
        None
    }

    /// Attempt to parse a header of up to 6 #'s.
    fn try_header(&mut self, lines: &[&'a str], idx: usize) -> Option<()> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^\s*(\#{1,6})(\s+(.*?))?(\s+\#*)?\s*$").unwrap();
        }
        if let Some(cap) = RE.captures(lines[idx]) {
            let lvl = cap.get(1).unwrap().as_str().len();
            let mut txt: &str = &"";
            if let Some(end_txt) = cap.get(2) {
                let end_pos = end_txt.as_str().len();
                txt = &lines[idx][lvl + 1..lvl + end_pos];
            }

            let node_idx = self.add_node(Kind::Header(lvl));
            self.node_add_text(node_idx, txt);
            self.close_node(node_idx);
            return Some(());
        }
        None
    }

    /// Attempts to parse the thematic break of `***`, `---`, and `___`.
    fn try_thematic_break(&mut self, lines: &[&'a str], idx: usize) -> Option<()> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\s*(\*|-|_)){3,}\s*$").unwrap();
        }
        if RE.is_match(lines[idx]) {
            let node_idx = self.add_node(Kind::ThematicBreak);
            self.close_node(node_idx);
            return Some(());
        }
        None
    }

    /// Attempt to parse a fenced code block in `lines`. If a code block is
    /// found, then consume the lines until the end of the block and return the
    /// number of lines consumed.
    fn try_fenced_code(&mut self, lines: &[&'a str], idx: usize) -> Option<usize> {
        lazy_static! {
            static ref START_RE: Regex = Regex::new(r"^(\s*)(`{3,}|~{3,})\s*([^\s]*).*$").unwrap();
            static ref END_RE: Regex = Regex::new(r"^\s*(`{3,}|~{3,})\s*$").unwrap();
        }

        let mut consumed = 0;
        if let Some(cap) = START_RE.captures(lines[idx]) {
            let indent = cap.get(1).unwrap().as_str().len();
            let marker = cap.get(2).unwrap().as_str().trim();
            let lang_str = cap.get(3).unwrap().as_str();
            let lang = if lang_str.is_empty() {
                None
            } else {
                Some(lang_str)
            };

            let node = self.add_node(Kind::Code(lang));
            consumed += 1;
            while idx + consumed < lines.len() {
                if let Some(cap) = END_RE.captures(lines[idx + consumed]) {
                    let end_marker = cap.get(1).unwrap().as_str().trim();
                    // End marker must be at least as long as the start marker
                    // and it must be of the same type
                    if end_marker.len() >= marker.len()
                        && end_marker.chars().next() == marker.chars().next()
                    {
                        break;
                    }
                }
                self.node_add_text(node, &lines[idx + consumed][indent..]);
                consumed += 1;
            }
            // Make sure to consume the end marker.
            consumed += 1;
            if consumed > 0 {
                self.close_node(node);
                return Some(consumed);
            }
        }
        None
    }
}
