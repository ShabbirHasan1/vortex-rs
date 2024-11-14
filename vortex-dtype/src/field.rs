//! Selectors for fields in (possibly nested) `StructDType`s
//!
//! A `Field` can either be a direct child field of the top-level struct (selected by name or index),
//! or a nested field (selected by a sequence of such selectors)

use core::fmt;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::StructDType;

/// A selector for a field in a struct
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Field {
    /// A field selector by name
    Name(String),
    /// A field selector by index (position)
    Index(usize),
}

impl From<&str> for Field {
    fn from(value: &str) -> Self {
        Field::Name(value.into())
    }
}

impl From<String> for Field {
    fn from(value: String) -> Self {
        Field::Name(value)
    }
}

impl From<usize> for Field {
    fn from(value: usize) -> Self {
        Field::Index(value)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Field::Name(name) => write!(f, "${name}"),
            Field::Index(idx) => write!(f, "[{idx}]"),
        }
    }
}

/// A path through a (possibly nested) struct, composed of a sequence of field selectors
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldPath(Vec<Field>);

/// The empty or "root" field path.
pub const EMPTY_FIELD_PATH: FieldPath = FieldPath(vec![]);

/// A reference to the empty or "root" field path.
pub const EMPTY_FIELD_PATH_REF: &FieldPath = &EMPTY_FIELD_PATH;

impl AsRef<[Field]> for FieldPath {
    fn as_ref(&self) -> &[Field] {
        &self.0
    }
}

impl FieldPath {
    /// Constructs a new `FieldPath` from a single field selector (i.e., a direct child field of the top-level struct)
    pub fn from_name<F: Into<Field>>(name: F) -> Self {
        Self(vec![name.into()])
    }

    /// An empty/root path..
    pub fn empty() -> Self {
        FieldPath(vec![])
    }

    /// The selector for the root (i.e., the top-level struct itself)
    pub fn root() -> Self {
        Self(vec![])
    }
    /// Returns the sequence of field selectors that make up this path
    pub fn path(&self) -> &[Field] {
        &self.0
    }

    /// Pushes a new field selector to the end of this path
    pub fn push<F: Into<Field>>(&mut self, field: F) {
        self.0.push(field.into());
    }

    /// Is this the root/empty path?
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The number of components in this path. The root path has length zero.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromIterator<Field> for FieldPath {
    fn from_iter<T: IntoIterator<Item = Field>>(iter: T) -> Self {
        FieldPath(iter.into_iter().collect())
    }
}

impl From<Field> for FieldPath {
    fn from(value: Field) -> Self {
        FieldPath(vec![value])
    }
}

impl From<Vec<Field>> for FieldPath {
    fn from(value: Vec<Field>) -> Self {
        FieldPath(value)
    }
}

impl Display for FieldPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0.iter().format("."), f)
    }
}

/// FIXME(DK)
pub struct FieldPathSet {}

impl FieldPathSet {
    /// FIXME(DK)
    pub fn contains(&self, _: FieldPath) {
        todo!()
    }

    /// FIXME(DK)
    pub fn difference(&self, _: &FieldPathSet) -> Self {
        todo!()
    }

    /// FIXME(DK)
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// FIXME(DK)
    pub fn len(&self) -> usize {
        todo!()
    }

    /// FIXME(DK)
    pub fn to_map(&self) -> FieldPathMap<()> {
        todo!()
    }
}

impl From<&StructDType> for FieldPathSet {
    fn from(_: &StructDType) -> Self {
        todo!()
    }
}

impl FromIterator<FieldPath> for FieldPathSet {
    fn from_iter<T: IntoIterator<Item = FieldPath>>(_: T) -> Self {
        todo!()
    }
}

/// FIXME(DK)
pub struct FieldPathMap<V> {
    _v: V,
}

impl<V> FieldPathMap<V> {
    /// FIXME(DK)
    pub fn map<W>(&self, _f: impl FnOnce(V) -> W) -> FieldPathMap<W> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_path() {
        let mut path = FieldPath::from_name("A");
        path.push("B");
        path.push("C");
        assert_eq!(path.to_string(), "$A.$B.$C");

        let fields = vec!["A", "B", "C"]
            .into_iter()
            .map(Field::from)
            .collect_vec();
        assert_eq!(path.path(), &fields);

        let vec_path = FieldPath::from(fields);
        assert_eq!(vec_path.to_string(), "$A.$B.$C");
        assert_eq!(path, vec_path);
    }
}
