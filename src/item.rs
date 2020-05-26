use function::Function;
use module::Module;

use r#enum::Enum;
use r#impl::Impl;
use r#struct::Struct;
use r#trait::Trait;


#[derive(Debug, Clone)]
pub enum Item {
    Module(Module),
    Struct(Struct),
    Function(Function),
    Trait(Trait),
    Enum(Enum),
    Impl(Impl),
    Raw(String),
}
