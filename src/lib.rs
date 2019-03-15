#![deny(warnings, missing_debug_implementations, missing_docs)]
#![doc(html_root_url = "https://docs.rs/codegen/0.1.1")]

//! Provides a builder API for generating Rust code.
//!
//! The general strategy for using the crate is as follows:
//!
//! 1. Create a `Scope` instance.
//! 2. Use the builder API to add elements to the scope.
//! 3. Call `Scope::to_string()` to get the generated code.
//!
//! For example:
//!
//! ```rust
//! use codegen::Scope;
//!
//! let mut scope = Scope::new();
//!
//! scope.new_struct("Foo")
//!     .derive("Debug")
//!     .field("one", "usize")
//!     .field("two", "String");
//!
//! println!("{}", scope.to_string());
//! ```

extern crate indexmap;

use indexmap::IndexMap;
use std::fmt::{self, Write};

/// Defines a scope.
///
/// A scope contains modules, types, etc...
#[derive(Debug, Clone)]
pub struct Scope {
    /// Scope documentation
    docs: Option<Docs>,

    /// Imports
    imports: IndexMap<String, IndexMap<String, Import>>,

    /// Contents of the documentation,
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
enum Item {
    Module(Module),
    Struct(Struct),
    Trait(Trait),
    Enum(Enum),
    Impl(Impl),
    Raw(String),
}

/// Defines a module.
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name
    name: String,

    /// Visibility
    vis: Option<String>,

    /// Module documentation
    docs: Option<Docs>,

    /// Contents of the module
    scope: Scope,
}

/// Defines an enumeration.
#[derive(Debug, Clone)]
pub struct Enum {
    type_def: TypeDef,
    variants: Vec<Variant>,
}

/// Defines a struct.
#[derive(Debug, Clone)]
pub struct Struct {
    type_def: TypeDef,

    /// Struct fields
    fields: Fields,
}

/// Define a trait.
#[derive(Debug, Clone)]
pub struct Trait {
    type_def: TypeDef,
    parents: Vec<Type>,
    associated_tys: Vec<AssociatedType>,
    fns: Vec<Function>,
}

/// Defines a type.
#[derive(Debug, Clone)]
pub struct Type {
    name: String,
    generics: Vec<Type>,
}

/// Defines a type definition.
#[derive(Debug, Clone)]
struct TypeDef {
    ty: Type,
    vis: Option<String>,
    docs: Option<Docs>,
    derive: Vec<String>,
    allow: Option<String>,
    repr: Option<String>,
    bounds: Vec<Bound>,
}

/// Defines an enum variant.
#[derive(Debug, Clone)]
pub struct Variant {
    name: String,
    fields: Fields,
}

/// Defines a set of fields.
#[derive(Debug, Clone)]
enum Fields {
    Empty,
    Tuple(Vec<Type>),
    Named(Vec<Field>),
}

/// Defines a struct field.
#[derive(Debug, Clone)]
struct Field {
    /// Field name
    name: String,

    /// Field type
    ty: Type,
}

/// Defines an associated type.
#[derive(Debug, Clone)]
pub struct AssociatedType(Bound);

#[derive(Debug, Clone)]
struct Bound {
    name: String,
    bound: Vec<Type>,
}

/// Defines an impl block.
#[derive(Debug, Clone)]
pub struct Impl {
    /// The struct being implemented
    target: Type,

    /// Impl level generics
    generics: Vec<String>,

    /// If implementing a trait
    impl_trait: Option<Type>,

    /// Associated types
    assoc_tys: Vec<Field>,

    /// Bounds
    bounds: Vec<Bound>,

    fns: Vec<Function>,
}

/// Defines an import (`use` statement).
#[derive(Debug, Clone)]
pub struct Import {
    line: String,
    vis: Option<String>,
}

/// Defines a function.
#[derive(Debug, Clone)]
pub struct Function {
    /// Name of the function
    name: String,

    /// Function documentation
    docs: Option<Docs>,

    /// A lint attribute used to suppress a warning or error
    allow: Option<String>,

    /// Function visibility
    vis: Option<String>,

    /// Function generics
    generics: Vec<String>,

    /// If the function takes `&self` or `&mut self`
    arg_self: Option<String>,

    /// Function arguments
    args: Vec<Field>,

    /// Return type
    ret: Option<Type>,

    /// Where bounds
    bounds: Vec<Bound>,

    /// Body contents
    body: Option<Vec<Body>>,
}

/// Defines a code block. This is used to define a function body.
#[derive(Debug, Clone)]
pub struct Block {
    before: Option<String>,
    after: Option<String>,
    body: Vec<Body>,
}

#[derive(Debug, Clone)]
enum Body {
    String(String),
    Block(Block),
}

#[derive(Debug, Clone)]
struct Docs {
    docs: String,
}

/// Configures how a scope is formatted.
#[derive(Debug)]
pub struct Formatter<'a> {
    /// Write destination
    dst: &'a mut String,

    /// Number of spaces to start a new line with.
    spaces: usize,

    /// Number of spaces per indentiation
    indent: usize,
}

const DEFAULT_INDENT: usize = 4;

// ===== impl Scope =====

impl Scope {
    /// Returns a new scope
    pub fn new() -> Self {
        Scope {
            docs: None,
            imports: IndexMap::new(),
            items: vec![],
        }
    }

    /// Import a type into the scope.
    ///
    /// This results in a new `use` statement being added to the beginning of
    /// the scope.
    pub fn import(&mut self, path: &str, ty: &str) -> &mut Import {
        // handle cases where the caller wants to refer to a type namespaced
        // within the containing namespace, like "a::B".
        let ty = ty.split("::").next().unwrap_or(ty);
        self.imports.entry(path.to_string())
            .or_insert(IndexMap::new())
            .entry(ty.to_string())
            .or_insert_with(|| Import::new(path, ty))
    }

    /// Push a new module definition, returning a mutable reference to it.
    ///
    /// # Panics
    ///
    /// Since a module's name must uniquely identify it within the scope in
    /// which it is defined, pushing a module whose name is already defined
    /// in this scope will cause this function to panic.
    ///
    /// In many cases, the [`get_or_new_module`] function is preferrable, as it
    /// will return the existing definition instead.
    ///
    /// [`get_or_new_module`]: #method.get_or_new_module
    pub fn new_module(&mut self, name: &str) -> &mut Module {
        self.push_module(Module::new(name));

        match *self.items.last_mut().unwrap() {
            Item::Module(ref mut v) => v,
            _ => unreachable!(),
        }
    }

    /// Returns a mutable reference to a module if it is exists in this scope.
    pub fn get_module_mut<Q: ?Sized>(&mut self,
                                     name: &Q)
                                     -> Option<&mut Module>
    where
        String: PartialEq<Q>,
    {
        self.items.iter_mut()
            .filter_map(|item| match item {
                &mut Item::Module(ref mut module) if module.name == *name =>
                    Some(module),
                _ => None,
            })
            .next()
    }

    /// Returns a mutable reference to a module if it is exists in this scope.
    pub fn get_module<Q: ?Sized>(&self, name: &Q) -> Option<&Module>
    where
        String: PartialEq<Q>,
    {
        self.items.iter()
            .filter_map(|item| match item {
                &Item::Module(ref module) if module.name == *name =>
                    Some(module),
                _ => None,
            })
            .next()
    }

    /// Returns a mutable reference to a module, creating it if it does
    /// not exist.
    pub fn get_or_new_module(&mut self, name: &str) -> &mut Module {
        if self.get_module(name).is_some() {
            self.get_module_mut(name).unwrap()
        } else {
            self.new_module(name)
        }
    }

    /// Push a module definition.
    ///
    /// # Panics
    ///
    /// Since a module's name must uniquely identify it within the scope in
    /// which it is defined, pushing a module whose name is already defined
    /// in this scope will cause this function to panic.
    ///
    /// In many cases, the [`get_or_new_module`] function is preferrable, as it will
    /// return the existing definition instead.
    ///
    /// [`get_or_new_module`]: #method.get_or_new_module
    pub fn push_module(&mut self, item: Module) -> &mut Self {
        assert!(self.get_module(&item.name).is_none());
        self.items.push(Item::Module(item));
        self
    }

    /// Push a new struct definition, returning a mutable reference to it.
    pub fn new_struct(&mut self, name: &str) -> &mut Struct {
        self.push_struct(Struct::new(name));

        match *self.items.last_mut().unwrap() {
            Item::Struct(ref mut v) => v,
            _ => unreachable!(),
        }
    }

    /// Push a struct definition
    pub fn push_struct(&mut self, item: Struct) -> &mut Self {
        self.items.push(Item::Struct(item));
        self
    }

    /// Push a new trait definition, returning a mutable reference to it.
    pub fn new_trait(&mut self, name: &str) -> &mut Trait {
        self.push_trait(Trait::new(name));

        match *self.items.last_mut().unwrap() {
            Item::Trait(ref mut v) => v,
            _ => unreachable!(),
        }
    }

    /// Push a trait definition
    pub fn push_trait(&mut self, item: Trait) -> &mut Self {
        self.items.push(Item::Trait(item));
        self
    }

    /// Push a new struct definition, returning a mutable reference to it.
    pub fn new_enum(&mut self, name: &str) -> &mut Enum {
        self.push_enum(Enum::new(name));

        match *self.items.last_mut().unwrap() {
            Item::Enum(ref mut v) => v,
            _ => unreachable!(),
        }
    }

    /// Push a structure definition
    pub fn push_enum(&mut self, item: Enum) -> &mut Self {
        self.items.push(Item::Enum(item));
        self
    }

    /// Push a new `impl` block, returning a mutable reference to it.
    pub fn new_impl(&mut self, target: &str) -> &mut Impl {
        self.push_impl(Impl::new(target));

        match *self.items.last_mut().unwrap() {
            Item::Impl(ref mut v) => v,
            _ => unreachable!(),
        }
    }

    /// Push an `impl` block.
    pub fn push_impl(&mut self, item: Impl) -> &mut Self {
        self.items.push(Item::Impl(item));
        self
    }

    /// Push a raw string to the scope.
    ///
    /// This string will be included verbatim in the formatted string.
    pub fn raw(&mut self, val: &str) -> &mut Self {
        self.items.push(Item::Raw(val.to_string()));
        self
    }

    /// Return a string representation of the scope.
    pub fn to_string(&self) -> String {
        let mut ret = String::new();

        self.fmt(&mut Formatter::new(&mut ret)).unwrap();

        // Remove the trailing newline
        if ret.as_bytes().last() == Some(&b'\n') {
            ret.pop();
        }

        ret
    }

    /// Formats the scope using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.fmt_imports(fmt)?;

        if !self.imports.is_empty() {
            write!(fmt, "\n")?;
        }

        for (i, item) in self.items.iter().enumerate() {
            if i != 0 {
                write!(fmt, "\n")?;
            }

            match *item {
                Item::Module(ref v) => v.fmt(fmt)?,
                Item::Struct(ref v) => v.fmt(fmt)?,
                Item::Trait(ref v) => v.fmt(fmt)?,
                Item::Enum(ref v) => v.fmt(fmt)?,
                Item::Impl(ref v) => v.fmt(fmt)?,
                Item::Raw(ref v) => {
                    write!(fmt, "{}\n", v)?;
                }
            }
        }

        Ok(())
    }

    fn fmt_imports(&self, fmt: &mut Formatter) -> fmt::Result {
        // First, collect all visibilities
        let mut visibilities = vec![];

        for (_, imports) in &self.imports {
            for (_, import) in imports {
                if !visibilities.contains(&import.vis) {
                    visibilities.push(import.vis.clone());
                }
            }
        }

        let mut tys = vec![];

        // Loop over all visibilities and format the associated imports
        for vis in &visibilities {
            for (path, imports) in &self.imports {
                tys.clear();

                for (ty, import) in imports {
                    if *vis == import.vis {
                        tys.push(ty);
                    }
                }

                if !tys.is_empty() {
                    if let Some(ref vis) = *vis {
                        write!(fmt, "{} ", vis)?;
                    }

                    write!(fmt, "use {}::", path)?;

                    if tys.len() > 1 {
                        write!(fmt, "{{")?;

                        for (i, ty) in tys.iter().enumerate() {
                            if i != 0 { write!(fmt, ", ")?; }
                            write!(fmt, "{}", ty)?;
                        }

                        write!(fmt, "}};\n")?;
                    } else if tys.len() == 1 {
                        write!(fmt, "{};\n", tys[0])?;
                    }
                }
            }
        }

        Ok(())
    }
}

// ===== impl Module =====

impl Module {
    /// Return a new, blank module
    pub fn new(name: &str) -> Self {
        Module {
            name: name.to_string(),
            vis: None,
            docs: None,
            scope: Scope::new(),
        }
    }

    /// Returns a mutable reference to the module's scope.
    pub fn scope(&mut self) -> &mut Scope {
        &mut self.scope
    }

    /// Set the module visibility.
    pub fn vis(&mut self, vis: &str) -> &mut Self {
        self.vis = Some(vis.to_string());
        self
    }

    /// Import a type into the module's scope.
    ///
    /// This results in a new `use` statement bein added to the beginning of the
    /// module.
    pub fn import(&mut self, path: &str, ty: &str) -> &mut Self {
        self.scope.import(path, ty);
        self
    }

    /// Push a new module definition, returning a mutable reference to it.
    ///
    /// # Panics
    ///
    /// Since a module's name must uniquely identify it within the scope in
    /// which it is defined, pushing a module whose name is already defined
    /// in this scope will cause this function to panic.
    ///
    /// In many cases, the [`get_or_new_module`] function is preferrable, as it
    /// will return the existing definition instead.
    ///
    /// [`get_or_new_module`]: #method.get_or_new_module
    pub fn new_module(&mut self, name: &str) -> &mut Module {
        self.scope.new_module(name)
    }

    /// Returns a reference to a module if it is exists in this scope.
    pub fn get_module<Q: ?Sized>(&self, name: &Q) -> Option<&Module>
    where
        String: PartialEq<Q>,
    {
        self.scope.get_module(name)
    }

    /// Returns a mutable reference to a module if it is exists in this scope.
    pub fn get_module_mut<Q: ?Sized>(&mut self,
                                     name: &Q)
                                     -> Option<&mut Module>
    where
        String: PartialEq<Q>,
    {
        self.scope.get_module_mut(name)
    }

    /// Returns a mutable reference to a module, creating it if it does
    /// not exist.
    pub fn get_or_new_module(&mut self, name: &str) -> &mut Module {
        self.scope.get_or_new_module(name)
    }

    /// Push a module definition.
    ///
    /// # Panics
    ///
    /// Since a module's name must uniquely identify it within the scope in
    /// which it is defined, pushing a module whose name is already defined
    /// in this scope will cause this function to panic.
    ///
    /// In many cases, the [`get_or_new_module`] function is preferrable, as it will
    /// return the existing definition instead.
    ///
    /// [`get_or_new_module`]: #method.get_or_new_module
    pub fn push_module(&mut self, item: Module) -> &mut Self {
        self.scope.push_module(item);
        self
    }

    /// Push a new struct definition, returning a mutable reference to it.
    pub fn new_struct(&mut self, name: &str) -> &mut Struct {
        self.scope.new_struct(name)
    }

    /// Push a structure definition
    pub fn push_struct(&mut self, item: Struct) -> &mut Self {
        self.scope.push_struct(item);
        self
    }

    /// Push a new enum definition, returning a mutable reference to it.
    pub fn new_enum(&mut self, name: &str) -> &mut Enum {
        self.scope.new_enum(name)
    }

    /// Push an enum definition
    pub fn push_enum(&mut self, item: Enum) -> &mut Self {
        self.scope.push_enum(item);
        self
    }

    /// Push a new `impl` block, returning a mutable reference to it.
    pub fn new_impl(&mut self, target: &str) -> &mut Impl {
        self.scope.new_impl(target)
    }

    /// Push an `impl` block.
    pub fn push_impl(&mut self, item: Impl) -> &mut Self {
        self.scope.push_impl(item);
        self
    }

    /// Formats the module using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        if let Some(ref vis) = self.vis {
            write!(fmt, "{} ", vis)?;
        }

        write!(fmt, "mod {}", self.name)?;
        fmt.block(|fmt| {
            self.scope.fmt(fmt)
        })
    }
}

// ===== impl Struct =====

impl Struct {
    /// Return a structure definition with the provided name
    pub fn new(name: &str) -> Self {
        Struct {
            type_def: TypeDef::new(name),
            fields: Fields::Empty,
        }
    }

    /// Returns a reference to the type
    pub fn ty(&self) -> &Type {
        &self.type_def.ty
    }

    /// Set the structure visibility.
    pub fn vis(&mut self, vis: &str) -> &mut Self {
        self.type_def.vis(vis);
        self
    }

    /// Add a generic to the struct.
    pub fn generic(&mut self, name: &str) -> &mut Self {
        self.type_def.ty.generic(name);
        self
    }

    /// Add a `where` bound to the struct.
    pub fn bound<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.type_def.bound(name, ty);
        self
    }

    /// Set the structure documentation.
    pub fn doc(&mut self, docs: &str) -> &mut Self {
        self.type_def.doc(docs);
        self
    }

    /// Add a new type that the struct should derive.
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

    /// Add a named field to the struct.
    ///
    /// A struct can either set named fields with this function or tuple fields
    /// with `tuple_field`, but not both.
    pub fn field<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.fields.named(name, ty);
        self
    }

    /// Add a tuple field to the struct.
    ///
    /// A struct can either set tuple fields with this function or named fields
    /// with `field`, but not both.
    pub fn tuple_field<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.fields.tuple(ty);
        self
    }

    /// Formats the struct using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.type_def.fmt_head("struct", &[], fmt)?;
        self.fields.fmt(fmt)?;

        match self.fields {
            Fields::Empty => {
                write!(fmt, ";\n")?;
            }
            Fields::Tuple(..) => {
                write!(fmt, ";\n")?;
            }
            _ => {}
        }

        Ok(())
    }
}

// ===== impl Trait =====

impl Trait {
    /// Return a trait definition with the provided name
    pub fn new(name: &str) -> Self {
        Trait {
            type_def: TypeDef::new(name),
            parents: vec![],
            associated_tys: vec![],
            fns: vec![],
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
    where T: Into<Type>,
    {
        self.type_def.bound(name, ty);
        self
    }

    /// Add a parent trait.
    pub fn parent<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
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
                if i != 0 || !assoc.is_empty() { write!(fmt, "\n")?; }

                func.fmt(true, fmt)?;
            }

            Ok(())
        })
    }
}

// ===== impl Enum =====

impl Enum {
    /// Return a enum definition with the provided name.
    pub fn new(name: &str) -> Self {
        Enum {
            type_def: TypeDef::new(name),
            variants: vec![],
        }
    }

    /// Returns a reference to the type.
    pub fn ty(&self) -> &Type {
        &self.type_def.ty
    }

    /// Set the enum visibility.
    pub fn vis(&mut self, vis: &str) -> &mut Self {
        self.type_def.vis(vis);
        self
    }

    /// Add a generic to the enum.
    pub fn generic(&mut self, name: &str) -> &mut Self {
        self.type_def.ty.generic(name);
        self
    }

    /// Add a `where` bound to the enum.
    pub fn bound<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.type_def.bound(name, ty);
        self
    }

    /// Set the enum documentation.
    pub fn doc(&mut self, docs: &str) -> &mut Self {
        self.type_def.doc(docs);
        self
    }

    /// Add a new type that the struct should derive.
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

    /// Push a variant to the enum, returning a mutable reference to it.
    pub fn new_variant(&mut self, name: &str) -> &mut Variant {
        self.push_variant(Variant::new(name));
        self.variants.last_mut().unwrap()
    }

    /// Push a variant to the enum.
    pub fn push_variant(&mut self, item: Variant) -> &mut Self {
        self.variants.push(item);
        self
    }

    /// Formats the enum using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.type_def.fmt_head("enum", &[], fmt)?;

        fmt.block(|fmt| {
            for variant in &self.variants {
                variant.fmt(fmt)?;
            }

            Ok(())
        })
    }
}

// ===== impl Variant =====

impl Variant {
    /// Return a new enum variant with the given name.
    pub fn new(name: &str) -> Self {
        Variant {
            name: name.to_string(),
            fields: Fields::Empty,
        }
    }

    /// Add a named field to the variant.
    pub fn named<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.fields.named(name, ty);
        self
    }

    /// Add a tuple field to the variant.
    pub fn tuple(&mut self, ty: &str) -> &mut Self {
        self.fields.tuple(ty);
        self
    }

    /// Formats the variant using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)?;
        self.fields.fmt(fmt)?;
        write!(fmt, ",\n")?;

        Ok(())
    }
}

// ===== impl Type =====

impl Type {
    /// Return a new type with the given name.
    pub fn new(name: &str) -> Self {
        Type {
            name: name.to_string(),
            generics: vec![],
        }
    }

    /// Add a generic to the type.
    pub fn generic<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        // Make sure that the name doesn't already include generics
        assert!(!self.name.contains("<"), "type name already includes generics");

        self.generics.push(ty.into());
        self
    }

    /// Rewrite the `Type` with the provided path
    ///
    /// TODO: Is this needed?
    pub fn path(&self, path: &str) -> Type {
        // TODO: This isn't really correct
        assert!(!self.name.contains("::"));

        let mut name = path.to_string();
        name.push_str("::");
        name.push_str(&self.name);

        Type {
            name,
            generics: self.generics.clone(),
        }
    }

    /// Formats the struct using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)?;
        Type::fmt_slice(&self.generics, fmt)
    }

    fn fmt_slice(generics: &[Type], fmt: &mut Formatter) -> fmt::Result {
        if !generics.is_empty() {
            write!(fmt, "<")?;

            for (i, ty) in generics.iter().enumerate() {
                if i != 0 { write!(fmt, ", ")? }
                ty.fmt(fmt)?;
            }

            write!(fmt, ">")?;
        }

        Ok(())
    }
}

impl<'a> From<&'a str> for Type {
    fn from(src: &'a str) -> Self {
        Type::new(src)
    }
}

impl From<String> for Type {
    fn from(src: String) -> Self {
        Type {
            name: src,
            generics: vec![],
        }
    }
}

impl<'a> From<&'a String> for Type {
    fn from(src: &'a String) -> Self {
        Type::new(src)
    }
}

impl<'a> From<&'a Type> for Type {
    fn from(src: &'a Type) -> Self {
        src.clone()
    }
}

// ===== impl TypeDef =====

impl TypeDef {
    /// Return a structure definition with the provided name
    fn new(name: &str) -> Self {
        TypeDef {
            ty: Type::new(name),
            vis: None,
            docs: None,
            derive: vec![],
            allow: None,
            repr: None,
            bounds: vec![],
        }
    }

    fn vis(&mut self, vis: &str) {
        self.vis = Some(vis.to_string());
    }

    fn bound<T>(&mut self, name: &str, ty: T)
    where T: Into<Type>,
    {
        self.bounds.push(Bound {
            name: name.to_string(),
            bound: vec![ty.into()],
        });
    }

    fn doc(&mut self, docs: &str) {
        self.docs = Some(Docs::new(docs));
    }

    fn derive(&mut self, name: &str) {
        self.derive.push(name.to_string());
    }

    fn allow(&mut self, allow: &str) {
        self.allow = Some(allow.to_string());
    }

    fn repr(&mut self, repr: &str) {
        self.repr = Some(repr.to_string());
    }

    fn fmt_head(&self,
                keyword: &str,
                parents: &[Type],
                fmt: &mut Formatter) -> fmt::Result
    {
        if let Some(ref docs) = self.docs {
            docs.fmt(fmt)?;
        }

        self.fmt_allow(fmt)?;
        self.fmt_derive(fmt)?;
        self.fmt_repr(fmt)?;

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

    fn fmt_allow(&self, fmt: &mut Formatter) -> fmt::Result {
        if let Some(ref allow) = self.allow {
            write!(fmt, "#[allow({})]\n", allow)?;
        }

        Ok(())
    }

    fn fmt_repr(&self, fmt: &mut Formatter) -> fmt::Result {
        if let Some(ref repr) = self.repr {
            write!(fmt, "#[repr({})]\n", repr)?;
        }

        Ok(())
    }

    fn fmt_derive(&self, fmt: &mut Formatter) -> fmt::Result {
        if !self.derive.is_empty() {
            write!(fmt, "#[derive(")?;

            for (i, name) in self.derive.iter().enumerate() {
                if i != 0 { write!(fmt, ", ")? }
                write!(fmt, "{}", name)?;
            }

            write!(fmt, ")]\n")?;
        }

        Ok(())
    }
}

fn fmt_generics(generics: &[String], fmt: &mut Formatter) -> fmt::Result {
    if !generics.is_empty() {
        write!(fmt, "<")?;

        for (i, ty) in generics.iter().enumerate() {
            if i != 0 { write!(fmt, ", ")? }
            write!(fmt, "{}", ty)?;
        }

        write!(fmt, ">")?;
    }

    Ok(())
}

fn fmt_bounds(bounds: &[Bound], fmt: &mut Formatter) -> fmt::Result {
    if !bounds.is_empty() {
        write!(fmt, "\n")?;

        // Write first bound
        write!(fmt, "where {}: ", bounds[0].name)?;
        fmt_bound_rhs(&bounds[0].bound, fmt)?;
        write!(fmt, ",\n")?;

        for bound in &bounds[1..] {
            write!(fmt, "      {}: ", bound.name)?;
            fmt_bound_rhs(&bound.bound, fmt)?;
            write!(fmt, ",\n")?;
        }
    }

    Ok(())
}

fn fmt_bound_rhs(tys: &[Type], fmt: &mut Formatter) -> fmt::Result {
    for (i, ty) in tys.iter().enumerate() {
        if i != 0 { write!(fmt, " + ")? }
        ty.fmt(fmt)?;
    }

    Ok(())
}

// ===== impl AssociatedType =====

impl AssociatedType {
    /// Add a bound to the associated type.
    pub fn bound<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.0.bound.push(ty.into());
        self
    }
}

// ===== impl Fields =====

impl Fields {
    fn named<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        match *self {
            Fields::Empty => {
                *self = Fields::Named(vec![Field {
                    name: name.to_string(),
                    ty: ty.into(),
                }]);
            }
            Fields::Named(ref mut fields) => {
                fields.push(Field {
                    name: name.to_string(),
                    ty: ty.into(),
                });
            }
            _ => panic!("field list is named"),
        }

        self
    }

    fn tuple<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        match *self {
            Fields::Empty => {
                *self = Fields::Tuple(vec![ty.into()]);
            }
            Fields::Tuple(ref mut fields) => {
                fields.push(ty.into());
            }
            _ => panic!("field list is tuple"),
        }

        self
    }

    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            Fields::Named(ref fields) => {
                assert!(!fields.is_empty());

                fmt.block(|fmt| {
                    for f in fields {
                        write!(fmt, "{}: ", f.name)?;
                        f.ty.fmt(fmt)?;
                        write!(fmt, ",\n")?;
                    }

                    Ok(())
                })?;
            }
            Fields::Tuple(ref tys) => {
                assert!(!tys.is_empty());

                write!(fmt, "(")?;

                for (i, ty) in tys.iter().enumerate() {
                    if i != 0 { write!(fmt, ", ")?; }
                    ty.fmt(fmt)?;
                }

                write!(fmt, ")")?;
            }
            Fields::Empty => {}
        }

        Ok(())
    }
}

// ===== impl Impl =====

impl Impl {
    /// Return a new impl definition
    pub fn new<T>(target: T) -> Self
    where T: Into<Type>,
    {
        Impl {
            target: target.into(),
            generics: vec![],
            impl_trait: None,
            assoc_tys: vec![],
            bounds: vec![],
            fns: vec![],
        }
    }

    /// Add a generic to the impl block.
    ///
    /// This adds the generic for the block (`impl<T>`) and not the target type.
    pub fn generic(&mut self, name: &str) -> &mut Self {
        self.generics.push(name.to_string());
        self
    }

    /// Add a generic to the target type.
    pub fn target_generic<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.target.generic(ty);
        self
    }

    /// Set the trait that the impl block is implementing.
    pub fn impl_trait<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.impl_trait = Some(ty.into());
        self
    }

    /// Set an associated type.
    pub fn associate_type<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.assoc_tys.push(Field {
            name: name.to_string(),
            ty: ty.into(),
        });

        self
    }

    /// Add a `where` bound to the impl block.
    pub fn bound<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.bounds.push(Bound {
            name: name.to_string(),
            bound: vec![ty.into()],
        });
        self
    }

    /// Push a new function definition, returning a mutable reference to it.
    pub fn new_fn(&mut self, name: &str) -> &mut Function {
        self.push_fn(Function::new(name));
        self.fns.last_mut().unwrap()
    }

    /// Push a function definition.
    pub fn push_fn(&mut self, item: Function) -> &mut Self {
        self.fns.push(item);
        self
    }

    /// Formats the impl block using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "impl")?;
        fmt_generics(&self.generics[..], fmt)?;

        if let Some(ref t) = self.impl_trait {
            write!(fmt, " ")?;
            t.fmt(fmt)?;
            write!(fmt, " for")?;
        }

        write!(fmt, " ")?;
        self.target.fmt(fmt)?;

        fmt_bounds(&self.bounds, fmt)?;

        fmt.block(|fmt| {
            // format associated types
            if !self.assoc_tys.is_empty() {
                for ty in &self.assoc_tys {
                    write!(fmt, "type {} = ", ty.name)?;
                    ty.ty.fmt(fmt)?;
                    write!(fmt, ";\n")?;
                }
            }

            for (i, func) in self.fns.iter().enumerate() {
                if i != 0 || !self.assoc_tys.is_empty() { write!(fmt, "\n")?; }

                func.fmt(false, fmt)?;
            }

            Ok(())
        })
    }
}

// ===== impl Import =====

impl Import {
    /// Return a new import.
    pub fn new(path: &str, ty: &str) -> Self {
        Import {
            line: format!("{}::{}", path, ty),
            vis: None,
        }
    }

    /// Set the import visibility.
    pub fn vis(&mut self, vis: &str) -> &mut Self {
        self.vis = Some(vis.to_string());
        self
    }
}

// ===== impl Func =====

impl Function {
    /// Return a new function definition.
    pub fn new(name: &str) -> Self {
        Function {
            name: name.to_string(),
            docs: None,
            allow: None,
            vis: None,
            generics: vec![],
            arg_self: None,
            args: vec![],
            ret: None,
            bounds: vec![],
            body: Some(vec![]),
        }
    }

    /// Set the function documentation.
    pub fn doc(&mut self, docs: &str) -> &mut Self {
        self.docs = Some(Docs::new(docs));
        self
    }

    /// Specify lint attribute to supress a warning or error.
    pub fn allow(&mut self, allow: &str) -> &mut Self {
        self.allow = Some(allow.to_string());
        self
    }

    /// Set the function visibility.
    pub fn vis(&mut self, vis: &str) -> &mut Self {
        self.vis = Some(vis.to_string());
        self
    }

    /// Add a generic to the function.
    pub fn generic(&mut self, name: &str) -> &mut Self {
        self.generics.push(name.to_string());
        self
    }

    /// Add `self` as a function argument.
    pub fn arg_self(&mut self) -> &mut Self {
        self.arg_self = Some("self".to_string());
        self
    }

    /// Add `&self` as a function argument.
    pub fn arg_ref_self(&mut self) -> &mut Self {
        self.arg_self = Some("&self".to_string());
        self
    }

    /// Add `&mut self` as a function argument.
    pub fn arg_mut_self(&mut self) -> &mut Self {
        self.arg_self = Some("&mut self".to_string());
        self
    }

    /// Add a function argument.
    pub fn arg<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.args.push(Field {
            name: name.to_string(),
            ty: ty.into(),
        });

        self
    }

    /// Set the function return type.
    pub fn ret<T>(&mut self, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.ret = Some(ty.into());
        self
    }

    /// Add a `where` bound to the function.
    pub fn bound<T>(&mut self, name: &str, ty: T) -> &mut Self
    where T: Into<Type>,
    {
        self.bounds.push(Bound {
            name: name.to_string(),
            bound: vec![ty.into()],
        });
        self
    }

    /// Push a line to the function implementation.
    pub fn line<T>(&mut self, line: T) -> &mut Self
    where T: ToString,
    {
        self.body.get_or_insert(vec![])
            .push(Body::String(line.to_string()));

        self
    }

    /// Push a block to the function implementation
    pub fn push_block(&mut self, block: Block) -> &mut Self {
        self.body.get_or_insert(vec![])
            .push(Body::Block(block));

        self
    }

    /// Formats the function using the given formatter.
    pub fn fmt(&self, is_trait: bool, fmt: &mut Formatter) -> fmt::Result {
        if let Some(ref docs) = self.docs {
            docs.fmt(fmt)?;
        }

        if let Some(ref allow) = self.allow {
            write!(fmt, "#[allow({})]\n", allow)?;
        }

        if is_trait {
            assert!(self.vis.is_none(), "trait fns do not have visibility modifiers");
        }

        if let Some(ref vis) = self.vis {
            write!(fmt, "{} ", vis)?;
        }

        write!(fmt, "fn {}", self.name)?;
        fmt_generics(&self.generics, fmt)?;

        write!(fmt, "(")?;

        if let Some(ref s) = self.arg_self {
            write!(fmt, "{}", s)?;
        }

        for (i, arg) in self.args.iter().enumerate() {
            if i != 0 || self.arg_self.is_some() {
                write!(fmt, ", ")?;
            }

            write!(fmt, "{}: ", arg.name)?;
            arg.ty.fmt(fmt)?;
        }

        write!(fmt, ")")?;

        if let Some(ref ret) = self.ret {
            write!(fmt, " -> ")?;
            ret.fmt(fmt)?;
        }

        fmt_bounds(&self.bounds, fmt)?;

        match self.body {
            Some(ref body) => {
                fmt.block(|fmt| {
                    for b in body {
                        b.fmt(fmt)?;
                    }

                    Ok(())
                })
            }
            None => {
                if !is_trait {
                    panic!("impl blocks must define fn bodies");
                }

                write!(fmt, ";\n")
            }
        }
    }
}

// ===== impl Block =====

impl Block {
    /// Returns an empty code block.
    pub fn new(before: &str) -> Self {
        Block {
            before: Some(before.to_string()),
            after: None,
            body: vec![],
        }
    }

    /// Push a line to the code block.
    pub fn line<T>(&mut self, line: T) -> &mut Self
    where T: ToString,
    {
        self.body.push(Body::String(line.to_string()));
        self
    }

    /// Push a nested block to this block.
    pub fn push_block(&mut self, block: Block) -> &mut Self {
        self.body.push(Body::Block(block));
        self
    }

    /// Add a snippet after the block.
    pub fn after(&mut self, after: &str) -> &mut Self {
        self.after = Some(after.to_string());
        self
    }

    /// Formats the block using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        if let Some(ref before) = self.before {
            write!(fmt, "{}", before)?;
        }

        // Inlined `Formatter::fmt`

        if !fmt.is_start_of_line() {
            write!(fmt, " ")?;
        }

        write!(fmt, "{{\n")?;

        fmt.indent(|fmt| {
            for b in &self.body {
                b.fmt(fmt)?;
            }

            Ok(())
        })?;

        write!(fmt, "}}")?;

        if let Some(ref after) = self.after {
            write!(fmt, "{}", after)?;
        }

        write!(fmt, "\n")?;
        Ok(())
    }
}

// ===== impl Body =====

impl Body {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            Body::String(ref s) => {
                write!(fmt, "{}\n", s)
            }
            Body::Block(ref b) => {
                b.fmt(fmt)
            }
        }
    }
}

// ===== impl Docs =====

impl Docs {
    fn new(docs: &str) -> Self {
        Docs { docs: docs.to_string() }
    }

    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        for line in self.docs.lines() {
            write!(fmt, "/// {}\n", line)?;
        }

        Ok(())
    }
}

// ===== impl Formatter =====

impl<'a> Formatter<'a> {
    /// Return a new formatter that writes to the given string.
    pub fn new(dst: &'a mut String) -> Self {
        Formatter {
            dst,
            spaces: 0,
            indent: DEFAULT_INDENT,
        }
    }

    fn block<F>(&mut self, f: F) -> fmt::Result
    where F: FnOnce(&mut Self) -> fmt::Result
    {
        if !self.is_start_of_line() {
            write!(self, " ")?;
        }

        write!(self, "{{\n")?;
        self.indent(f)?;
        write!(self, "}}\n")?;
        Ok(())
    }

    /// Call the given function with the indentation level incremented by one.
    fn indent<F, R>(&mut self, f: F) -> R
    where F: FnOnce(&mut Self) -> R
    {
        self.spaces += self.indent;
        let ret = f(self);
        self.spaces -= self.indent;
        ret
    }

    fn is_start_of_line(&self) -> bool {
        self.dst.is_empty() ||
            self.dst.as_bytes().last() == Some(&b'\n')
    }

    fn push_spaces(&mut self) {
        for _ in 0..self.spaces {
            self.dst.push_str(" ");
        }
    }
}

impl<'a> fmt::Write for Formatter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut first = true;
        let mut should_indent = self.is_start_of_line();

        for line in s.lines() {
            if !first {
                self.dst.push_str("\n");
            }

            first = false;

            let do_indent = should_indent &&
                !line.is_empty() &&
                line.as_bytes()[0] != b'\n';

            if do_indent {
                self.push_spaces();
            }

            // If this loops again, then we just wrote a new line
            should_indent = true;

            self.dst.push_str(line);
        }

        if s.as_bytes().last() == Some(&b'\n') {
            self.dst.push_str("\n");
        }

        Ok(())
    }
}
