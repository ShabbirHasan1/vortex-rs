use std::any::Any;
use std::fmt::Display;

use vortex_array::aliases::hash_set::HashSet;
use vortex_array::array::StructArray;
use vortex_array::variants::StructArrayTrait;
use vortex_array::Array;
use vortex_dtype::field::{Field, FieldPath};
use vortex_error::VortexResult;

use crate::{unbox_any, VortexExpr};

#[derive(Debug, PartialEq, Hash, Clone, Eq)]
pub struct Column {
    field_path: FieldPath,
}

impl Column {
    pub fn new(field: Field) -> Self {
        Self {
            field_path: FieldPath::from(field),
        }
    }

    pub fn new_path(field_path: FieldPath) -> Self {
        Self { field_path }
    }

    pub fn field_path(&self) -> &FieldPath {
        &self.field_path
    }

    // pub fn field(&self) -> &Field {
    //     &self.field
    // }
}

impl From<String> for Column {
    fn from(value: String) -> Self {
        Column::new(value.into())
    }
}

impl From<usize> for Column {
    fn from(value: usize) -> Self {
        Column::new(value.into())
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.field_path)
    }
}

impl VortexExpr for Column {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn evaluate(&self, batch: &Array) -> VortexResult<Array> {
        let mut array = batch.clone();
        for field in self.field_path.as_ref() {
            let struct_array = StructArray::try_from(array)?;
            array = struct_array.field(field)?;
        }
        Ok(array)
    }

    fn collect_references<'a>(&'a self, references: &mut HashSet<&'a FieldPath>) {
        references.insert(self.field_path());
    }
}

impl PartialEq<dyn Any> for Column {
    fn eq(&self, other: &dyn Any) -> bool {
        unbox_any(other)
            .downcast_ref::<Self>()
            .map(|x| x == self)
            .unwrap_or(false)
    }
}
