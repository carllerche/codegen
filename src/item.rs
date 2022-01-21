use crate::function::Function;
use crate::module::Module;

use crate::r#enum::Enum;
use crate::r#impl::Impl;
use crate::r#struct::Struct;
use crate::r#trait::Trait;

/// The types of items that can be defined.
#[derive(Debug, Clone)]
pub enum Item {
    /// Module
    Module(Module),
    /// Struct
    Struct(Struct),
    /// Function
    Function(Function),
    /// Trait
    Trait(Trait),
    /// Enum
    Enum(Enum),
    /// Impl
    Impl(Impl),
    /// Raw
    Raw(String),
}
