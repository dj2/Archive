use crate::tree::{Block, Doc, Inline};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Kind {
    Doc,
    Blockquote,
    Paragraph,
}
#[derive(Clone, Debug)]
struct Node<'a> {
    kind: Kind,
    open: bool,
    blocks: Vec<usize>,
    text: Vec<&'a str>,
}
impl<'a> Node<'a> {
    fn new(kind: Kind) -> Self {
        Self {
            kind,
            open: true,
            blocks: vec![],
            text: vec![],
        }
    }

    fn is_closed_by(&self, kind: Kind) -> bool {
        match self.kind {
            Kind::Doc => false,
            Kind::Blockquote => true,
            Kind::Paragraph => !(kind == Kind::Paragraph),
        }
    }

    fn is_closed_by_hardbreak(&self) -> bool {
        self.kind == Kind::Paragraph
    }
}

pub struct Parser<'a> {
    root: usize,
    nodes: Vec<Node<'a>>,
    buf: &'a str,
}
impl<'a, 'b> Parser<'a> {
    pub fn new(buf: &'a str) -> Self {
        Self {
            root: 0,
            nodes: vec![Node::new(Kind::Doc)],
            buf,
        }
    }

    pub fn parse(&mut self) -> Doc<'a> {
        let lines: Vec<&'a str> = self.buf.lines().collect();
        self.parse_lines(&lines);
        self.build_doc()
    }

    fn build_doc(&mut self) -> Doc<'a> {
        let mut blocks = vec![];
        for idx in &self.nodes[self.root].blocks {
            blocks.push(self.to_block(*idx));
        }

        Doc { blocks }
    }

    fn to_block(&self, idx: usize) -> Block<'a> {
        match self.nodes[idx].kind {
            Kind::Doc => {
                panic!("Should not call to_block on a document");
            }
            Kind::Blockquote => {
                assert!(self.nodes[idx].text.is_empty());

                let mut blocks = vec![];
                for n in &self.nodes[idx].blocks {
                    blocks.push(self.to_block(*n));
                }
                Block::Blockquote(blocks)
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
        }
    }

    fn find_open_node(&self, idx: usize) -> usize {
        let node = self.nodes[idx].blocks.last();
        if let Some(i) = node {
            if self.nodes[*i].open {
                return self.find_open_node(*i);
            }
        }

        idx
    }

    fn get_open_parent_for(&mut self, kind: Kind) -> usize {
        loop {
            let i = self.find_open_node(self.root);
            if self.nodes[i].is_closed_by(kind) {
                self.nodes[i].open = false;
                continue;
            }
            return i;
        }
    }

    fn parse_lines(&mut self, lines: &[&'a str]) {
        let mut idx = 0;
        while idx < lines.len() {
            if lines[idx].trim().is_empty() {
                let node_idx = self.find_open_node(self.root);
                if self.nodes[node_idx].is_closed_by_hardbreak() {
                    self.nodes[node_idx].open = false
                }
            } else if let Some(consumed) = self.parse_blockquote(&lines, idx) {
                idx += consumed;
                continue;
            } else {
                let mut node_idx = self.find_open_node(self.root);
                if self.nodes[node_idx].text.is_empty() {
                    self.nodes.push(Node::new(Kind::Paragraph));
                    let val = self.nodes.len() - 1;

                    self.nodes[node_idx].blocks.push(val);
                    node_idx = val;
                }
                self.nodes[node_idx].text.push(&lines[idx]);
            }
            idx += 1;
        }
    }

    fn parse_blockquote(&mut self, lines: &[&'a str], idx: usize) -> Option<usize> {
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
            let node_idx = self.get_open_parent_for(Kind::Blockquote);
            self.nodes.push(Node::new(Kind::Blockquote));
            let val = self.nodes.len() - 1;
            self.nodes[node_idx].blocks.push(val);

            // Now that the blockquote is inserted, it will be the open node.
            let node_idx = self.find_open_node(self.root);
            assert!(self.nodes[node_idx].kind == Kind::Blockquote);

            self.parse_lines(&sub_lines);
            return Some(consumed);
        }
        None
    }
}
