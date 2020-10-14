use crate::r#type::Type;

#[derive(Debug, Clone)]
pub struct Bound {
    pub name: String,
    pub bound: Vec<Type>,
}
