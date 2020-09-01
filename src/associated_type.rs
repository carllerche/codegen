use crate::bound::Bound;
use crate::r#type::Type;

/// Defines an associated type.
#[derive(Debug, Clone)]
pub struct AssociatedType(pub Bound);

impl AssociatedType {
    /// Add a bound to the associated type.
    pub fn bound<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.0.bound.push(ty.into());
        self
    }
}
