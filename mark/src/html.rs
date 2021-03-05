use std::fmt;

pub struct Html {}

impl Html {
    pub fn new() -> Self {
        Html {}
    }
}
impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HTML")
    }
}
