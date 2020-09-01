use std::fmt::{self, Write};

use crate::block::Block;
use crate::formatter::Formatter;

#[derive(Debug, Clone)]
pub enum Body {
    String(String),
    Block(Block),
}

impl Body {
    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Body::String(s) => write!(fmt, "{}\n", s),
            Body::Block(b) => b.fmt(fmt),
        }
    }
}
