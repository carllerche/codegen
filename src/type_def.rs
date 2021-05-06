use std::fmt::{self, Write};

use crate::bound::Bound;
use crate::docs::Docs;
use crate::formatter::{fmt_bounds, Formatter};

use crate::r#type::Type;

/// Defines a type definition.
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub ty: Type,
    vis: Option<String>,
    docs: Option<Docs>,
    derive: Vec<String>,
    allow: Vec<String>,
    repr: Option<String>,
    bounds: Vec<Bound>,
    macros: Vec<String>,
}

impl TypeDef {
    /// Return a structure definition with the provided name
    pub fn new(name: &str) -> Self {
        TypeDef {
            ty: Type::new(name),
            vis: None,
            docs: None,
            derive: vec![],
            allow: vec![],
            repr: None,
            bounds: vec![],
            macros: vec![],
        }
    }

    pub fn vis(&mut self, vis: &str) {
        self.vis = Some(vis.to_string());
    }

    pub fn bound<T>(&mut self, name: &str, ty: T)
    where
        T: Into<Type>,
    {
        self.bounds.push(Bound {
            name: name.to_string(),
            bound: vec![ty.into()],
        });
    }

    pub fn r#macro(&mut self, r#macro: &str) {
        self.macros.push(r#macro.to_string());
    }

    pub fn doc(&mut self, docs: &str) {
        self.docs = Some(Docs::new(docs));
    }

    pub fn derive(&mut self, name: &str) {
        self.derive.push(name.to_string());
    }

    pub fn allow(&mut self, allow: &str) {
        self.allow.push(allow.to_string());
    }

    pub fn repr(&mut self, repr: &str) {
        self.repr = Some(repr.to_string());
    }

    pub fn fmt_head(
        &self,
        keyword: &str,
        parents: &[Type],
        fmt: &mut Formatter<'_>,
    ) -> fmt::Result {
        if let Some(ref docs) = self.docs {
            docs.fmt(fmt)?;
        }

        self.fmt_allow(fmt)?;
        self.fmt_derive(fmt)?;
        self.fmt_repr(fmt)?;
        self.fmt_macros(fmt)?;

        if let Some(ref vis) = self.vis {
            write!(fmt, "{} ", vis)?;
        }

        write!(fmt, "{} ", keyword)?;
        self.ty.fmt(fmt)?;

        if !parents.is_empty() {
            for (i, ty) in parents.iter().enumerate() {
                if i == 0 {
                    write!(fmt, ": ")?;
                } else {
                    write!(fmt, " + ")?;
                }

                ty.fmt(fmt)?;
            }
        }

        fmt_bounds(&self.bounds, fmt)?;

        Ok(())
    }

    fn fmt_allow(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        for allow in &self.allow {
            write!(fmt, "#[allow({})]\n", allow)?;
        }

        Ok(())
    }

    fn fmt_repr(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref repr) = self.repr {
            write!(fmt, "#[repr({})]\n", repr)?;
        }

        Ok(())
    }

    fn fmt_derive(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        if !self.derive.is_empty() {
            write!(fmt, "#[derive(")?;

            for (i, name) in self.derive.iter().enumerate() {
                if i != 0 {
                    write!(fmt, ", ")?
                }
                write!(fmt, "{}", name)?;
            }

            write!(fmt, ")]\n")?;
        }

        Ok(())
    }

    fn fmt_macros(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        for m in self.macros.iter() {
            write!(fmt, "{}\n", m)?;
        }
        Ok(())
    }
}
