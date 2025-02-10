use std::hash::Hash;
use std::sync::Arc;

use vortex_expr::ExprRef;

/// An expression wrapper that performs pointer equality.
/// NOTE(ngates): we should consider if this shoud live in vortex-expr crate?
#[derive(Clone)]
pub(crate) struct ExactExpr(pub(crate) ExprRef);

impl PartialEq for ExactExpr {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ExactExpr {}

impl Hash for ExactExpr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state)
    }
}
