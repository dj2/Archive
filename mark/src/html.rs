use crate::tree::Doc;
use std::fmt;

pub struct Html<'a> {
    doc: &'a Doc<'a>,
}

impl<'a> Html<'a> {
    pub fn new(doc: &'a Doc) -> Self {
        Html { doc }
    }
}
impl<'a> fmt::Display for Html<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.doc.to_string())
    }
}
