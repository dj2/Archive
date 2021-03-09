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
use crate::tree::{Block, Doc, Marker};
use regex::Regex;

#[derive(Copy, Clone, Debug, PartialEq)]
enum MarkerClose {
    None,
    Dot,
    Bracket,
}

fn parse_marker(marker: &'_ str) -> (Marker, MarkerClose, u32) {
    let mut chars = marker.chars();
    let marker_first = chars.next();
    let marker_close = match chars.last() {
        Some(')') => MarkerClose::Bracket,
        Some('.') => MarkerClose::Dot,
        _ => MarkerClose::None,
    };

    let mut marker_start = 1;
    let marker_kind = match marker_first {
        Some('*') => Marker::Bullet,
        Some('-') => Marker::Dash,
        Some('+') => Marker::Plus,
        Some('i') => Marker::LowerRoman,
        Some('I') => Marker::UpperRoman,
        Some(x) if ('a'..='z').contains(&x) => {
            marker_start = (x as u32) - ('a' as u32) + 1;
            Marker::LowerAlpha
        }
        Some(x) if ('A'..='Z').contains(&x) => {
            marker_start = (x as u32) - ('A' as u32) + 1;
            Marker::UpperAlpha
        }
        _ => {
            if let Ok(val) = marker[0..marker.len() - 1].to_string().parse::<u32>() {
                marker_start = val;
            }
            Marker::Numeric
        }
    };
    (marker_kind, marker_close, marker_start)
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Kind<'a> {
    Blockquote,
    Code(Option<&'a str>),
    Doc,
    Header(usize /* level */),
    List(
        usize, /* indent */
        Marker,
        MarkerClose,
        u32, /* start value */
    ),
    ListElement,
    Paragraph,
    ThematicBreak,

    Text(&'a str),
    Emphasis,
}

/// A node holds information about a given block in the document. The node
/// will hold _either_ `blocks` or `text` but not both. These aren't stored as
/// optional as we want to be able to push new entries into the lists.
#[derive(Clone, Debug)]
struct Node<'a> {
    kind: Kind<'a>,
    open: bool,
    blocks: Vec<usize>,
}
impl<'a> Node<'a> {
    fn new(kind: Kind<'a>) -> Self {
        Self {
            kind,
            open: true,
            blocks: vec![],
        }
    }

    /// Determines if the current node is closed by a node of `kind`.
    fn is_closed_by(&self, kind: Kind) -> bool {
        if kind == Kind::Emphasis {
            return false;
        }
        if let Kind::Text(_) = kind {
            return false;
        }

        match self.kind {
            Kind::Doc | Kind::ListElement => false,
            Kind::Blockquote | Kind::Paragraph | Kind::Header(_) => kind != Kind::Paragraph,
            _ => true,
        }
    }

    /// Determines if a blank line in the document will close the current node.
    fn is_closed_by_hardbreak(&self) -> bool {
        self.kind == Kind::Paragraph
    }
}


fn is_inline_open(left: Option<&(usize, char)>, right: Option<&(usize, char)>) -> bool {
    if let Some((_, left_char)) = left {
        if !left_char.is_whitespace() {
            return false;
        }
    }
    // Left was none, or whitespace, check right

    if let Some((_, right_char)) = right {
        if !right_char.is_whitespace() {
            return true;
        }
    }
    // If right is none, we fail as we don't allow starting the inline at
    // end of a line.
    false
}

fn is_inline_close(left: Option<&(usize, char)>, right: Option<&(usize, char)>) -> bool {
    if let Some((_, left_char)) = left {
        if left_char.is_whitespace() {
            return false;
        }
    } else {
        // Don't close at the start of a line.
        return false;
    }

    if let Some((_, right_char)) = right {
        if right_char.is_whitespace() {
            return true;
        }
    }
    true
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

    fn convert_blocks(&self, idx: usize) -> Vec<Block<'a>> {
        let mut blocks = vec![];
        for n in &self.nodes[idx].blocks {
            blocks.push(self.to_block(*n));
        }
        blocks
    }

    /// Converts the node at `idx` into a corresponding block.
    fn to_block(&self, idx: usize) -> Block<'a> {
        match self.nodes[idx].kind {
            Kind::Doc => panic!("Should not call to_block on a document"),
            Kind::Code(lang) => Block::Code(lang, self.convert_blocks(idx)),
            Kind::Blockquote => Block::Blockquote(self.convert_blocks(idx)),
            Kind::Header(lvl) => Block::Header(lvl, self.convert_blocks(idx)),
            Kind::List(_, marker, _, start) => Block::List(marker, start, self.convert_blocks(idx)),
            Kind::ListElement => Block::ListElement(self.convert_blocks(idx)),
            Kind::Paragraph => Block::Paragraph(self.convert_blocks(idx)),
            Kind::ThematicBreak => Block::ThematicBreak,
            Kind::Text(txt) => Block::Text(txt),
            Kind::Emphasis => Block::Emphasis(self.convert_blocks(idx)),
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

    fn find_parent_list(
        &self,
        idx: usize,
        indent: usize,
        marker: Marker,
        marker_close: MarkerClose,
    ) -> Option<usize> {
        let node = self.nodes[idx].blocks.last();
        if let Some(i) = node {
            if self.nodes[*i].open {
                if let Some(ret) = self.find_parent_list(*i, indent, marker, marker_close) {
                    return Some(ret);
                }

                if let Kind::List(ind, marker_kind, close, _) = self.nodes[*i].kind {
                    // If the indent level matches, the marker is the same
                    // and the marker close are the same, then this is
                    // the list we attach too.
                    if ind == indent && marker_kind == marker && close == marker_close {
                        return Some(*i);
                    }
                }
            }
        }
        None
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

    fn add_node_to_parent(&mut self, parent: usize, kind: Kind<'a>) -> usize {
        self.nodes.push(Node::new(kind));

        let val = self.nodes.len() - 1;
        self.nodes[parent].blocks.push(val);
        val
    }

    /// Creates a new node of `kind` and adds to the current open node. The
    /// id of the new node is returned.
    fn add_node(&mut self, kind: Kind<'a>) -> usize {
        let parent = self.get_open_parent_for(kind);
        self.add_node_to_parent(parent, kind)
    }

    fn add_text_node(&mut self, txt: &'a str) {
        let idx = self.add_node(Kind::Text(txt));
        self.nodes[idx].open = false;
    }

    /// Marks node at `idx` as closed.
    fn close_node(&mut self, idx: usize) {
        self.nodes[idx].open = false;
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
            } else if self.try_thematic_break(&lines, idx).is_some()
                || self.try_header(&lines, idx).is_some()
            {
                idx += 1;
            } else if let Some(consumed) = self.try_fenced_code(&lines, idx) {
                idx += consumed;
            } else if let Some(consumed) = self.try_blockquote(&lines, idx) {
                idx += consumed;
            } else if let Some(consumed) = self.try_list(&lines, idx) {
                idx += consumed;
            } else {
                let node_idx = self.find_open_node(self.root);
                if self.nodes[node_idx].kind == Kind::Paragraph {
                    self.add_text_node("\n");
                } else {
                    self.add_node(Kind::Paragraph);
                }
                self.parse_inlines(&lines[idx]);
                idx += 1;
            }
        }
    }

    /// Parses the given line for inline elements
    fn parse_inlines(&mut self, line: &'a str) {
        let chars: Vec<(usize, char)> = line.char_indices().collect();
        let count = chars.len();
        let mut start_idx = 0;
        let mut idx = 0;
        while idx < count {
            let (pos, ch) = chars[idx];
            match ch {
                '*' => {
                    if is_inline_open(chars.get(idx - 1), chars.get(idx + 1)) {
                        self.add_text_node(&line[chars[start_idx].0..pos]);
                        self.add_node(Kind::Emphasis);
                        start_idx = idx + 1;
                    } else if is_inline_close(chars.get(idx - 1), chars.get(idx + 1)) {
                        self.add_text_node(&line[chars[start_idx].0..pos]);
                        self.close_node(self.find_open_node(self.root));
                        start_idx = idx + 1;
                    }
                }
                _ => {}
            }
            idx += 1;
        }
        if idx > start_idx {
            self.add_text_node(&line[chars[start_idx].0..]);
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
            self.parse_inlines(txt);
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
                if consumed > 1 {
                    self.add_text_node("\n");
                }
                self.add_text_node(&lines[idx + consumed][indent..]);
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

    /// Attempt to parse a list in `lines`. If a list is found, then
    /// consume the lines until the end of the list and returns the number
    /// of lines consumed.
    fn try_list(&mut self, lines: &[&'a str], idx: usize) -> Option<usize> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^(\s*(?:\*|\+|\-|(?:(?:[0-9]{1,9}|[a-z]|[A-Z])(?:\.|\)))))(\s+.*)?$")
                    .unwrap();
            static ref SPACE_RE: Regex = Regex::new(r"^(\s*)").unwrap();
        }

        if let Some(cap) = RE.captures(lines[idx]) {
            let marker = cap.get(1).unwrap().as_str();
            // Subsequent lines must indent to marker + 1.
            let indent = marker.len() + 1;
            let (marker_kind, marker_close, marker_start) = parse_marker(marker.trim());

            let mut consumed = 1;
            let mut sub_lines: Vec<&'a str> = vec![&lines[idx][indent..]];
            while idx + consumed < lines.len() {
                if let Some(space_cap) = SPACE_RE.captures(lines[idx + consumed]) {
                    let sp = space_cap.get(1).unwrap().as_str();
                    // If the amount of space is at least as much as the marker
                    // but the line is not just whitespace.
                    if sp.len() < indent && sp.len() != lines[idx + consumed].len() {
                        break;
                    }
                } else {
                    break;
                }

                if lines[idx + consumed].trim().is_empty() {
                    sub_lines.push(&lines[idx + consumed]);
                } else {
                    // Strip the marker space from the start of the line.
                    sub_lines.push(&lines[idx + consumed][indent..]);
                }
                consumed += 1;
            }
            if consumed > 0 {
                // We've found what could be a list element, we need to determine
                // if it's really a list element or we need to revert to text.

                // First, get the current open node, if it's a paragraph then we
                // look at the marker and only allow certain makers to break the
                // paragraph.
                let open_node = self.find_open_node(self.root);
                if self.nodes[open_node].kind == Kind::Paragraph {
                    // Ordered markers must start with 1 to break a paragraph
                    if (marker_kind == Marker::Numeric
                        || marker_kind == Marker::UpperAlpha
                        || marker_kind == Marker::LowerAlpha)
                        && marker_start != 1
                    {
                        return None;
                    }
                }

                let parent = self.find_parent_list(self.root, indent, marker_kind, marker_close);
                let parent_idx = parent.map_or_else(
                    // We didn't find a parent to add too, so find the open node,
                    // and add the list.
                    || self.add_node(Kind::List(indent, marker_kind, marker_close, marker_start)),
                    // Found a list which matches this new element so we'll append
                    // to that list instead of creating a new one.
                    |idx| idx,
                );

                // Add the element, parse it's contents and then close the element.
                let li = self.add_node_to_parent(parent_idx, Kind::ListElement);
                self.parse_lines(&sub_lines);
                self.close_node(li);
                return Some(consumed);
            }
        }
        None
    }
}
