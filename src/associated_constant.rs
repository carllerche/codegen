use std::fmt::{self, Write};
use crate::formatter::Formatter;

/// Defines an associated constant for use in impls and traits
#[derive(Clone, Debug)]
pub struct AssociatedConstant {
    name: String,
    datatype: crate::r#type::Type,
    value: Option<String>,
}

impl AssociatedConstant {
    /// Returns an associated constant with the given name and datatype
    pub fn new<Datatype>(name: &str, datatype: Datatype) -> Self
    where
        Datatype: Into<crate::r#type::Type>
    {
        Self {
            name: name.into(),
            datatype: datatype.into(),
            value: None,
        }
    }

    /// Adds a value expression to the associated constant
    pub fn value(&mut self, expression: &str) -> &mut Self {
        self.value = Some(expression.to_string());
        self
    }

    /// Formats the scope using the given formatter.
    pub fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let value_expression = match &self.value {
            Some(expression) => format!(" = {};", expression),
            None => ";".to_string(),
        };
        write!(fmt, "const {}: ", self.name)?;
        self.datatype.fmt(fmt)?;
        write!(fmt, "{}", value_expression)?;
        Ok(())
    }
}