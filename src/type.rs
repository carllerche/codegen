use std::fmt::{self, Write};

use crate::formatter::Formatter;

/// Defines a type.
#[derive(Debug, Clone)]
pub struct Type {
    name: String,
    generics: Vec<Type>,
}

impl Type {
    /// Return a new type with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            generics: vec![],
        }
    }

    /// Add a generic to the type.
    pub fn generic<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Self>,
    {
        // Make sure that the name doesn't already include generics
        assert!(
            !self.name.contains('<'),
            "type name already includes generics"
        );

        self.generics.push(ty.into());
        self
    }

    /// Rewrite the `Type` with the provided path
    ///
    /// TODO: Is this needed?
    pub fn path(&self, path: &str) -> Self {
        // TODO: This isn't really correct
        assert!(!self.name.contains("::"));

        let mut name = path.to_string();
        name.push_str("::");
        name.push_str(&self.name);

        Self {
            name,
            generics: self.generics.clone(),
        }
    }

    /// Formats the struct using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.name)?;
        Self::fmt_slice(&self.generics, fmt)
    }

    fn fmt_slice(generics: &[Self], fmt: &mut Formatter<'_>) -> fmt::Result {
        if !generics.is_empty() {
            write!(fmt, "<")?;

            for (i, ty) in generics.iter().enumerate() {
                if i != 0 {
                    write!(fmt, ", ")?
                }
                ty.fmt(fmt)?;
            }

            write!(fmt, ">")?;
        }

        Ok(())
    }
}

impl From<&str> for Type {
    fn from(src: &str) -> Self {
        Self::new(src)
    }
}

impl From<String> for Type {
    fn from(src: String) -> Self {
        Self {
            name: src,
            generics: vec![],
        }
    }
}

impl From<&String> for Type {
    fn from(src: &String) -> Self {
        Self::new(src)
    }
}

impl From<&Type> for Type {
    fn from(src: &Self) -> Self {
        src.clone()
    }
}
