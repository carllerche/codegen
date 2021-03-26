use crate::bound::Bound;
use crate::r#type::Type;

/// Defines an associated constant.
#[derive(Debug, Clone)]
pub struct AssociatedConst(pub Bound);

impl AssociatedConst {
    /// Set the bound on the associated constant.
    pub fn bound<T>(&mut self, ty: T) -> &mut Self
    where
        T: Into<Type>,
    {
        self.0.bound = vec![ty.into()];
        self
    }
}
