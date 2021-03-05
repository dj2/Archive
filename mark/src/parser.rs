pub struct Parser<'a> {
    buf: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(buf: &'a str) -> Self {
        Self { buf }
    }
}
