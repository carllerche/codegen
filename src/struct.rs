use std::fmt::{self, Write};

use field::Field;
use fields::Fields;
use formatter::Formatter;
use type_def::TypeDef;

use r#trait::AbsTrait;
use r#type::Type;


/// Defines a struct.
#[derive(Debug, Clone)]
pub struct Struct {
    type_def: TypeDef,

    /// Struct fields
    fields: Fields,
}


/// AbsStruct
pub trait AbsStruct : AbsTrait {
    /// Specify lint attribute to supress a warning or error.
    fn allow(&mut self, allow: &str) -> &mut Self {
        self.type_def().allow(allow);
        self
    }

    /// Add a new type that the struct should derive.
    fn derive(&mut self, name: &str) -> &mut Self {
        self.type_def().derive(name);
        self
    }

    /// Specify representation.
    fn repr(&mut self, repr: &str) -> &mut Self {
        self.type_def().repr(repr);
        self
    }
}


impl AbsTrait for Struct{
    fn type_def(&mut self) -> &mut TypeDef {
        &mut self.type_def
    }
}
impl AbsStruct for Struct{}

impl Struct {
    /// Return a structure definition with the provided name
    pub fn new(name: &str) -> Self {
        Struct {
            type_def: TypeDef::new(name),
            fields: Fields::Empty,
        }
    }

    /// Push a named field to the struct.
    ///
    /// A struct can either set named fields with this function or tuple fields
    /// with `push_tuple_field`, but not both.
    pub fn push_field(&mut self, field: Field) -> &mut Self
    {
        self.fields.push_named(field);
        self
    }

    /// Add a named field to the struct.
    ///
    /// A struct can either set named fields with this function or tuple fields
    /// with `tuple_field`, but not both.
    pub fn field<T>(&mut self, name: &str, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.fields.named(name, ty);
        self
    }

    /// Add a tuple field to the struct.
    ///
    /// A struct can either set tuple fields with this function or named fields
    /// with `field`, but not both.
    pub fn tuple_field<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
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
