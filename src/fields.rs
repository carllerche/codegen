use std::fmt::{self, Write};

use crate::field::Field;
use crate::formatter::Formatter;

use crate::r#type::Type;

/// Defines a set of fields.
#[derive(Debug, Clone)]
pub enum Fields {
    Empty,
    Tuple(Vec<Type>),
    Named(Vec<Field>),
}

impl Fields {
    pub fn push_named(&mut self, field: Field) -> &mut Self {
        match *self {
            Self::Empty => {
                *self = Self::Named(vec![field]);
            }
            Self::Named(fields) => {
                fields.push(field);
            }
            _ => panic!("field list is named"),
        }

        self
    }

    pub fn named<T>(&mut self, name: &str, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.push_named(Field {
            name: name.to_string(),
            ty: ty.into(),
            documentation: Vec::new(),
            annotation: Vec::new(),
        })
    }

    pub fn tuple<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        match self {
            Self::Empty => {
                *self = Self::Tuple(vec![ty.into()]);
            }
            Self::Tuple(fields) => {
                fields.push(ty.into());
            }
            _ => panic!("field list is tuple"),
        }

        self
    }

    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Named(ref fields) => {
                assert!(!fields.is_empty());

                fmt.block(|fmt| {
                    for f in fields {
                        if !f.documentation.is_empty() {
                            for doc in &f.documentation {
                                writeln!(fmt, "/// {}", doc)?;
                            }
                        }
                        if !f.annotation.is_empty() {
                            for ann in &f.annotation {
                                writeln!(fmt, "{}", ann)?;
                            }
                        }
                        write!(fmt, "{}: ", f.name)?;
                        f.ty.fmt(fmt)?;
                        writeln!(fmt, ",")?;
                    }

                    Ok(())
                })?;
            }
            Self::Tuple(tys) => {
                assert!(!tys.is_empty());

                write!(fmt, "(")?;

                for (i, ty) in tys.iter().enumerate() {
                    if i != 0 {
                        write!(fmt, ", ")?;
                    }
                    ty.fmt(fmt)?;
                }

                write!(fmt, ")")?;
            }
            Self::Empty => {}
        }

        Ok(())
    }
}
