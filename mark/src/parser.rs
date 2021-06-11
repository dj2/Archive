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

#![allow(clippy::trivial_regex)]

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct ListData {
    dist_to_marker: usize,
    dist_after_marker: usize,
    marker: Marker,
    close: MarkerClose,
    start_value: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Kind<'a> {
    Blockquote,
    Code(Option<&'a str>),
    Doc,
    Header(usize /* level */),
    List(ListData),
    ListElement,
    Paragraph,
    ThematicBreak,
    RawHtml,

    Text(&'a str),
    Inline(&'a str),
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
        if let Kind::Inline(_) = kind {
            return false;
        }
        if let Kind::Text(_) = kind {
            return false;
        }

        match self.kind {
            Kind::Doc | Kind::Blockquote | Kind::ListElement | Kind::RawHtml => false,
            Kind::Paragraph | Kind::Header(_) => kind != Kind::Paragraph,
            _ => true,
        }
    }

    /// Determines if a blank line in the document will close the current node.
    fn is_closed_by_hardbreak(&self) -> bool {
        self.kind == Kind::Paragraph
    }
}

fn is_inline_open(ch: char, left: Option<&(usize, char)>, right: Option<&(usize, char)>) -> bool {
    if let Some(&(_, left_char)) = left {
        if !left_char.is_whitespace() {
            return false;
        }
    }
    // Left was none, or whitespace, check right

    if let Some(&(_, right_char)) = right {
        if !right_char.is_whitespace() && right_char != ch {
            return true;
        }
    }
    // If right is none, we fail as we don't allow starting the inline at
    // end of a line.
    false
}

fn is_inline_close(ch: char, left: Option<&(usize, char)>, right: Option<&(usize, char)>) -> bool {
    if let Some(&(_, left_char)) = left {
        if left_char.is_whitespace() || ch == left_char {
            return false;
        }
    } else {
        // Don't close at the start of a line.
        return false;
    }

    if let Some(&(_, right_char)) = right {
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
            Kind::List(data) => {
                Block::List(data.marker, data.start_value, self.convert_blocks(idx))
            }
            Kind::ListElement => Block::ListElement(self.convert_blocks(idx)),
            Kind::Paragraph => Block::Paragraph(self.convert_blocks(idx)),
            Kind::ThematicBreak => Block::ThematicBreak,
            Kind::Text(txt) => Block::Text(txt),
            Kind::Inline(el) => Block::Inline(el, self.convert_blocks(idx)),
            Kind::RawHtml => Block::RawHtml(self.convert_blocks(idx)),
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
        to_marker: usize,
        after_marker: usize,
        marker: Marker,
        marker_close: MarkerClose,
    ) -> Option<usize> {
        let node = self.nodes[idx].blocks.last();
        if let Some(&i) = node {
            if self.nodes[i].open {
                if let Some(ret) =
                    self.find_parent_list(i, to_marker, after_marker, marker, marker_close)
                {
                    return Some(ret);
                }

                if let Kind::List(data) = self.nodes[i].kind {
                    // If the indent level matches, the marker is the same
                    // and the marker close are the same, then this is
                    // the list we attach too. For the space after the marker, we
                    // only compare it if it's non-zero.
                    if data.dist_to_marker <= to_marker
                        && (after_marker == 0 || data.dist_after_marker <= after_marker)
                        && to_marker <= (data.dist_to_marker + data.dist_after_marker)
                        && data.marker == marker
                        && data.close == marker_close
                    {
                        return Some(i);
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
            } else if self.try_setext_header(&lines, idx).is_some()
                || self.try_thematic_break(&lines, idx).is_some()
                || self.try_header(&lines, idx).is_some()
            {
                idx += 1;
            } else if let Some(consumed) = self.try_raw_html(&lines, idx) {
                idx += consumed;
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
                };
                self.parse_inlines(lines[idx].trim());
                idx += 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_inline_char(
        &mut self,
        kind: Kind<'a>,
        ch: char,
        line: &'a str,
        prev: Option<&(usize, char)>,
        next: Option<&(usize, char)>,
        start: usize,
        end: usize,
    ) -> bool {
        if is_inline_open(ch, prev, next) {
            self.add_text_node(&line[start..end]);
            self.add_node(kind);
            true
        } else if is_inline_close(ch, prev, next) {
            self.add_text_node(&line[start..end]);
            self.close_node(self.find_open_node(self.root));
            true
        } else {
            false
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
            let prev = if idx > 0 { chars.get(idx - 1) } else { None };
            let next = chars.get(idx + 1);
            let start = chars[start_idx].0;
            match ch {
                '_' => {
                    if self.process_inline_char(
                        Kind::Inline("em"),
                        '_',
                        line,
                        prev,
                        next,
                        start,
                        pos,
                    ) {
                        start_idx = idx + 1;
                    }
                }
                '*' => {
                    if self.process_inline_char(
                        Kind::Inline("strong"),
                        '*',
                        line,
                        prev,
                        next,
                        start,
                        pos,
                    ) {
                        start_idx = idx + 1;
                    }
                }
                '`' => {
                    if self.process_inline_char(
                        Kind::Inline("code"),
                        '`',
                        line,
                        prev,
                        next,
                        start,
                        pos,
                    ) {
                        start_idx = idx + 1;
                    }
                }
                '\\' => {
                    // Handle unescaping escaped characters
                    if let Some((_, nxt_ch)) = chars.get(idx + 1) {
                        match nxt_ch {
                            '#' | '*' | '!' | '$' | '%' | '\'' | '(' | ')' | '+' | ',' | '-'
                            | '.' | '/' | ':' | ';' | '=' | '?' | '@' | '[' | '\\' | ']' | '^'
                            | '_' | '`' | '{' | '|' | '}' | '~' => {
                                self.add_text_node(&line[start..pos]);
                                start_idx = idx + 1;
                                idx += 1;
                            }
                            '"' => {
                                self.add_text_node(&line[start..pos]);
                                self.add_text_node("&quot;");
                                start_idx = idx + 2;
                                idx += 1;
                            }
                            '&' => {
                                self.add_text_node(&line[start..pos]);
                                self.add_text_node("&amp;");
                                start_idx = idx + 2;
                                idx += 1;
                            }
                            '>' => {
                                self.add_text_node(&line[start..pos]);
                                self.add_text_node("&gt;");
                                start_idx = idx + 2;
                                idx += 1;
                            }
                            '<' => {
                                self.add_text_node(&line[start..pos]);
                                self.add_text_node("&lt;");
                                start_idx = idx + 2;
                                idx += 1;
                            }
                            _ => {}
                        }
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
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\s*>\s?)").unwrap();
        }

        let mut consumed = 0;
        let mut sub_lines: Vec<&'a str> = vec![];

        while idx + consumed < lines.len() {
            if let Some(cap) = RE.captures(lines[idx + consumed]) {
                let marker = cap.get(1).unwrap().as_str().len();

                // Strip the marker from the start of the blockquote.
                sub_lines.push(&lines[idx + consumed][marker..]);
                consumed += 1;
            } else {
                break;
            }
        }
        if consumed > 0 {
            let node_idx = self.add_node(Kind::Blockquote);
            self.parse_lines(&sub_lines);
            self.close_node(node_idx);
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
            if let Some(end_txt) = cap.get(3) {
                // In the case where the header is `### ###` we end up with the second `###`
                // as the match, so trim leading matches to turn it into a blank string.
                txt = end_txt.as_str().trim_start_matches('#');
            }

            let node_idx = self.add_node(Kind::Header(lvl));
            self.parse_inlines(txt);
            self.close_node(node_idx);
            return Some(());
        }
        None
    }

    /// Attempts to parse a Setext header `===` or `---`. Note, this deviates
    /// from Commonmark as we require at least 3 of the marker characters.
    fn try_setext_header(&mut self, lines: &[&'a str], idx: usize) -> Option<()> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^\s*(\-{3,}|={3,})\s*$").unwrap();
        }
        if let Some(cap) = RE.captures(lines[idx]) {
            let marker = cap.get(1).unwrap().as_str().trim();

            let node_idx = self.find_open_node(self.root);
            if self.nodes[node_idx].kind == Kind::Paragraph {
                let lvl = if marker.starts_with('-') { 2 } else { 1 };

                self.nodes[node_idx].kind = Kind::Header(lvl);
                self.close_node(node_idx);
                return Some(());
            }
        }
        None
    }

    /// Attempts to parse the thematic break of `***`, `---`, and `___`.
    fn try_thematic_break(&mut self, lines: &[&'a str], idx: usize) -> Option<()> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^((\s*\*){3,}|(\s*\-){3,}|(\s*_){3,})\s*$").unwrap();
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
            static ref START_RE: Regex =
                Regex::new(r"^(\s*)(`{3,}|~{3,})\s*([^\s]*)(.*)$").unwrap();
            static ref END_RE: Regex = Regex::new(r"^\s*(`{3,}|~{3,})\s*$").unwrap();
        }

        let mut consumed = 0;
        if let Some(cap) = START_RE.captures(lines[idx]) {
            let indent = cap.get(1).unwrap().as_str().len();
            let marker = cap.get(2).unwrap().as_str().trim();
            let lang_str = cap.get(3).unwrap().as_str();
            let rem = cap.get(4).unwrap().as_str();

            // The marker can't contain spaces, so if the end regular expression
            // matches this is an open and close code block which is invalid.
            if END_RE.is_match(lang_str) {
                return None;
            }
            // Backtick blocks can't contain backticks in lang block
            if marker.starts_with('`') && (lang_str.contains('`') || rem.contains('`')) {
                return None;
            }

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

                let line = &lines[idx + consumed];
                let chars: Vec<(usize, char)> = line.char_indices().collect();
                let mut char_idx = 0;

                // Skip indent whitespace if present.
                for _ in 0..indent {
                    let (_, ch) = chars[char_idx];
                    if !ch.is_whitespace() {
                        break;
                    }
                    char_idx += 1;
                }

                let mut start_idx = char_idx;
                while char_idx < chars.len() {
                    let (pos, ch) = chars[char_idx];
                    let start = chars[start_idx].0;
                    match ch {
                        '>' => {
                            self.add_text_node(&line[start..pos]);
                            self.add_text_node("&gt;");
                            start_idx = char_idx + 1;
                        }
                        '<' => {
                            self.add_text_node(&line[start..pos]);
                            self.add_text_node("&lt;");
                            start_idx = char_idx + 1;
                        }
                        _ => {}
                    }
                    char_idx += 1;
                }
                if char_idx > start_idx {
                    self.add_text_node(&line[chars[start_idx].0..]);
                }
                consumed += 1;
            }
            // Make sure to consume the end marker.
            consumed += 1;
            self.close_node(node);
            return Some(consumed);
        }
        None
    }

    fn try_raw_html(&mut self, lines: &[&'a str], idx: usize) -> Option<usize> {
        lazy_static! {
            static ref SCRIPT_PRE_OR_STYLE_OPEN_RE: Regex =
                Regex::new(r"^\s*<(?i)(script|pre|style)(\s|>|$)").unwrap();
            static ref SCRIPT_PRE_OR_STYLE_CLOSE_RE: Regex =
                Regex::new(r"</(?i)(script|pre|style)>").unwrap();

            static ref COMMENT_OPEN_RE: Regex = Regex::new(r"^\s*<!\-\-").unwrap();
            static ref COMMENT_CLOSE_RE: Regex = Regex::new(r"\-\->").unwrap();

            static ref PHP_OPEN_RE: Regex = Regex::new(r"^\s*<\?").unwrap();
            static ref PHP_CLOSE_RE: Regex = Regex::new(r"\?>").unwrap();

            static ref LESS_BANG_OPEN_RE: Regex = Regex::new(r"^\s*<![A-Z]").unwrap();
            static ref LESS_BANG_CLOSE_RE: Regex = Regex::new(r">").unwrap();

            static ref CDATA_OPEN_RE: Regex = Regex::new(r"^\s*<!\[CDATA\[").unwrap();
            static ref CDATA_CLOSE_RE: Regex = Regex::new(r"\]\]>").unwrap();

            static ref TAG_OPEN_RE: Regex = Regex::new(r"^\s*</?(?i)(address|article|aside|base|basefont|blockquote|body|caption|center|col|colgroup|dd|details|dialog|dir|div|dl|dt|fieldset|figcaption|figure|footer|form|frame|frameset|h[1-4]|head|header|hr|html|iframe|legend|li|link|main|menu|menuitem|nav|noframes|ol|optgroup|option|p|param|section|source|summary|table|tbody|td|tfoot|th|thead|title|tr|track|ul)(\s|/?>|$)").unwrap();

            static ref CUSTOM_TAG_OPEN1_RE: Regex = Regex::new(r"^\s*<[a-zA-Z0-9\-]+[^>]*?/?>\s*$").unwrap();
            static ref CUSTOM_TAG_OPEN2_RE: Regex = Regex::new(r"^\s*</[a-zA-Z0-9\-]+[^>]*?>\s*$").unwrap();

            static ref TAG_OR_CUSTOM_TAG_CLOSE_RE: Regex = Regex::new(r"^\s*$").unwrap();
        }

        let (is_custom, close_re): (bool, &Regex) =
            if SCRIPT_PRE_OR_STYLE_OPEN_RE.is_match(lines[idx]) {
                (false, &SCRIPT_PRE_OR_STYLE_CLOSE_RE)
            } else if COMMENT_OPEN_RE.is_match(lines[idx]) {
                (false, &COMMENT_CLOSE_RE)
            } else if PHP_OPEN_RE.is_match(lines[idx]) {
                (false, &PHP_CLOSE_RE)
            } else if LESS_BANG_OPEN_RE.is_match(lines[idx]) {
                (false, &LESS_BANG_CLOSE_RE)
            } else if CDATA_OPEN_RE.is_match(lines[idx]) {
                (false, &CDATA_CLOSE_RE)
            } else if TAG_OPEN_RE.is_match(lines[idx]) {
                (true, &TAG_OR_CUSTOM_TAG_CLOSE_RE)
            } else if CUSTOM_TAG_OPEN1_RE.is_match(lines[idx])
                || CUSTOM_TAG_OPEN2_RE.is_match(lines[idx])
            {
                // Custom tag does not break paragraphs.
                let open_node = self.find_open_node(self.root);
                if self.nodes[open_node].kind == Kind::Paragraph {
                    return None;
                }
                (true, &TAG_OR_CUSTOM_TAG_CLOSE_RE)
            } else {
                return None;
            };

        let node = self.add_node(Kind::RawHtml);
        let mut consumed = 0;
        while idx + consumed < lines.len() {
            if close_re.is_match(lines[idx + consumed]) {
                if !is_custom {
                    self.add_text_node(lines[idx + consumed]);
                    self.add_text_node("\n");
                }
                break;
            }
            self.add_text_node(lines[idx + consumed]);
            self.add_text_node("\n");
            consumed += 1;
        }
        // Make sure to consume the end marker.
        consumed += 1;
        self.close_node(node);

        Some(consumed)
    }

    /// Attempt to parse a list in `lines`. If a list is found, then
    /// consume the lines until the end of the list and returns the number
    /// of lines consumed.
    fn try_list(&mut self, lines: &[&'a str], idx: usize) -> Option<usize> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^(\s*(?:\*|\+|\-|(?:(?:[0-9]{1,9}|[a-z]|[A-Z])(?:\.|\)))))(?:(\s{1,4})(.*)|)?$"
            )
            .unwrap();
            static ref SPACE_RE: Regex = Regex::new(r"^(\s*)").unwrap();
        }

        if let Some(cap) = RE.captures(lines[idx]) {
            let marker = cap.get(1).unwrap().as_str();
            let sp = cap.get(2).map_or("", |v| v.as_str());
            let rem = cap.get(3).map_or("", |v| v.as_str());

            // Subsequent lines must indent to marker + 1. The exception is if
            // the list marker is blank, then there is no indent amount other
            // then the marker itself.
            let (indent, start_blank) = if rem.trim().is_empty() {
                (marker.len(), true)
            } else {
                (marker.len() + sp.len(), false)
            };
            let (marker_kind, marker_close, marker_start) = parse_marker(marker.trim());

            // Blank list marker can not interrupt a paragraph.
            let open_node = self.find_open_node(self.root);
            if start_blank && self.nodes[open_node].kind == Kind::Paragraph {
                return None;
            }

            let mut consumed = 1;
            let mut sub_lines: Vec<&'a str> = vec![&lines[idx][indent..]];
            while idx + consumed < lines.len() {
                if let Some(space_cap) = SPACE_RE.captures(lines[idx + consumed]) {
                    let start_sp = space_cap.get(1).unwrap().as_str();
                    // If the amount of space is at least as much as the marker
                    // but the line is not just whitespace.
                    if start_sp.len() < indent && start_sp.len() != lines[idx + consumed].len() {
                        break;
                    }
                } else {
                    break;
                }

                if lines[idx + consumed].trim().is_empty() {
                    // Only 1 blank line allowed at the start of the list.
                    if start_blank && consumed == 1 {
                        break;
                    }
                }
                sub_lines.push(&lines[idx + consumed]);
                consumed += 1;
            }
            if consumed > 0 {
                // We've found what could be a list element, we need to determine
                // if it's really a list element or we need to revert to text.

                // First, get the current open node, if it's a paragraph then we
                // look at the marker and only allow certain makers to break the
                // paragraph.
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

                let parent = self.find_parent_list(
                    self.root,
                    marker.len(),
                    sp.len(),
                    marker_kind,
                    marker_close,
                );
                let parent_idx = parent.map_or_else(
                    // We didn't find a parent to add too, so find the open node,
                    // and add the list.
                    || {
                        self.add_node(Kind::List(ListData {
                            dist_to_marker: marker.len(),
                            dist_after_marker: sp.len(),
                            marker: marker_kind,
                            close: marker_close,
                            start_value: marker_start,
                        }))
                    },
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
