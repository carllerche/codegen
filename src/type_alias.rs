use core::fmt;
use std::fmt::Write;

use crate::{type_def::TypeDef, Formatter, Type};

/// https://rust-lang.github.io/chalk/book/types/rust_types/alias.html#alias-types
#[derive(Debug, Clone)]
pub struct TypeAlias {
    type_def: TypeDef,
    ty: Type,
}

impl TypeAlias {
    pub fn new(name: &str, ty: &str) -> Self {
        Self {
            type_def: TypeDef::new(name),
            ty: Type::new(ty),
        }
    }
    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        self.type_def.fmt_head("type", &[], fmt)?;
        write!(fmt, " = ")?;
        self.ty.fmt(fmt)?;
        write!(fmt, ";")?;
        Ok(())
    }

    /// Returns a reference to the type
    pub fn type_def(&self) -> &Type {
        &self.type_def.ty
    }

    /// Set the TypeAliasure visibility.
    pub fn vis(&mut self, vis: &str) -> &mut Self {
        self.type_def.vis(vis);
        self
    }

    /// Add a generic to the TypeAlias.
    pub fn generic(&mut self, name: &str) -> &mut Self {
        self.type_def.ty.generic(name);
        self
    }

    /// Add a `where` bound to the TypeAlias.
    pub fn bound<T>(&mut self, name: &str, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.type_def.bound(name, ty);
        self
    }

    /// Set the TypeAliasure documentation.
    pub fn doc(&mut self, docs: &str) -> &mut Self {
        self.type_def.doc(docs);
        self
    }

    /// Add a new type that the TypeAlias should derive.
    pub fn derive(&mut self, name: &str) -> &mut Self {
        self.type_def.derive(name);
        self
    }

    /// Specify lint attribute to supress a warning or error.
    pub fn allow(&mut self, allow: &str) -> &mut Self {
        self.type_def.allow(allow);
        self
    }

    /// Specify representation.
    pub fn repr(&mut self, repr: &str) -> &mut Self {
        self.type_def.repr(repr);
        self
    }

    /// Set the type alias's ty.
    pub fn set_ty(&mut self, ty: Type) {
        self.ty = ty;
    }

    /// Get a reference to the type alias's ty.
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}
