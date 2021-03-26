#![deny(missing_debug_implementations, missing_docs)]
#![doc(html_root_url = "https://docs.rs/codegen/0.1.1")]
#![warn(rust_2018_idioms)]

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

mod associated_const;
mod associated_type;
mod block;
mod body;
mod bound;
mod docs;
mod field;
mod fields;
mod formatter;
mod function;
mod import;
mod item;
mod module;
mod scope;
mod type_def;
mod variant;

mod r#enum;
mod r#impl;
mod r#struct;
mod r#trait;
mod r#type;

pub use associated_const::*;
pub use associated_type::*;
pub use block::*;
pub use field::*;
pub use formatter::*;
pub use function::*;
pub use import::*;
pub use module::*;
pub use scope::*;
pub use variant::*;

pub use r#enum::*;
pub use r#impl::*;
pub use r#struct::*;
pub use r#trait::*;
pub use r#type::*;
