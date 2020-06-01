use std::fmt::{self, Write};

use block::Block;
use formatter::Formatter;


#[derive(Debug, Clone)]
pub enum Body {
    String(String),
    Block(Block),
}


impl Body {
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            Body::String(ref s) => write!(fmt, "{}\n", s),
            Body::Block(ref b) => b.fmt(fmt),
        }
    }
}
