use std::fmt::{self, Write};

use formatter::Formatter;


#[derive(Debug, Clone)]
pub struct Docs {
    docs: String,
}


impl Docs {
    pub fn new(docs: &str) -> Self {
        Docs {
            docs: docs.to_string(),
        }
    }

    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        for line in self.docs.lines() {
            write!(fmt, "/// {}\n", line)?;
        }

        Ok(())
    }
}
