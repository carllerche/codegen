use std::fmt::{self, Write};

use associated_type::AssociatedType;
use bound::Bound;
use formatter::{Formatter, fmt_bound_rhs};
use function::Function;
use type_def::TypeDef;

use r#type::Type;


/// Define a trait.
#[derive(Debug, Clone)]
pub struct Trait {
    type_def: TypeDef,
    parents: Vec<Type>,
    associated_tys: Vec<AssociatedType>,
    fns: Vec<Function>,
    macros: Vec<String>,
}


impl Trait {
    /// Return a trait definition with the provided name
    pub fn new(name: &str) -> Self {
        Trait {
            type_def: TypeDef::new(name),
            parents: vec![],
            associated_tys: vec![],
            fns: vec![],
            macros: vec![],
        }
    }

    /// Returns a reference to the type
    pub fn ty(&self) -> &Type {
        &self.type_def.ty
    }

    /// Set the trait visibility.
    pub fn vis(&mut self, vis: &str) -> &mut Self {
        self.type_def.vis(vis);
        self
    }

    /// Add a generic to the trait
    pub fn generic(&mut self, name: &str) -> &mut Self {
        self.type_def.ty.generic(name);
        self
    }

    /// Add a `where` bound to the trait.
    pub fn bound<T>(&mut self, name: &str, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.type_def.bound(name, ty);
        self
    }

    /// Add a macro to the trait def (e.g. `"#[async_trait]"`)
    pub fn r#macro(&mut self, r#macro: &str) -> &mut Self {
        self.type_def.r#macro(r#macro);
        self
    }

    /// Add a parent trait.
    pub fn parent<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.parents.push(ty.into());
        self
    }

    /// Set the trait documentation.
    pub fn doc(&mut self, docs: &str) -> &mut Self {
        self.type_def.doc(docs);
        self
    }

    /// Add an associated type. Returns a mutable reference to the new
    /// associated type for futher configuration.
    pub fn associated_type(&mut self, name: &str) -> &mut AssociatedType {
        self.associated_tys.push(AssociatedType(Bound {
            name: name.to_string(),
            bound: vec![],
        }));

        self.associated_tys.last_mut().unwrap()
    }

    /// Push a new function definition, returning a mutable reference to it.
    pub fn new_fn(&mut self, name: &str) -> &mut Function {
        let mut func = Function::new(name);
        func.body = None;

        self.push_fn(func);
        self.fns.last_mut().unwrap()
    }

    /// Push a function definition.
    pub fn push_fn(&mut self, item: Function) -> &mut Self {
        self.fns.push(item);
        self
    }

    /// Formats the scope using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.type_def.fmt_head("trait", &self.parents, fmt)?;

        fmt.block(|fmt| {
            let assoc = &self.associated_tys;

            // format associated types
            if !assoc.is_empty() {
                for ty in assoc {
                    let ty = &ty.0;

                    write!(fmt, "type {}", ty.name)?;

                    if !ty.bound.is_empty() {
                        write!(fmt, ": ")?;
                        fmt_bound_rhs(&ty.bound, fmt)?;
                    }

                    write!(fmt, ";\n")?;
                }
            }

            for (i, func) in self.fns.iter().enumerate() {
                if i != 0 || !assoc.is_empty() {
                    write!(fmt, "\n")?;
                }

                func.fmt(true, fmt)?;
            }

            Ok(())
        })
    }
}
