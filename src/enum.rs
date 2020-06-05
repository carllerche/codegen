use std::fmt;

use formatter::Formatter;
use type_def::TypeDef;
use variant::Variant;

use r#trait::AbsTrait;
use r#struct::AbsStruct;


/// Defines an enumeration.
#[derive(Debug, Clone)]
pub struct Enum {
    type_def: TypeDef,
    variants: Vec<Variant>,
}


impl AbsTrait for Enum{
    fn type_def(&mut self) -> &mut TypeDef {
        &mut self.type_def
    }
}
impl AbsStruct for Enum{}


impl Enum {
    /// Return a enum definition with the provided name.
    pub fn new(name: &str) -> Self {
        Enum {
            type_def: TypeDef::new(name),
            variants: vec![],
        }
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
