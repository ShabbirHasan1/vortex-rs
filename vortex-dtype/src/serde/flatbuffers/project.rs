use std::sync::Arc;

use vortex_error::{vortex_bail, vortex_err, VortexResult};

use crate::field::Field;
use crate::{flatbuffers as fb, DType, StructDType};

fn find_name<'a>(
    names: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<&'a str>>,
    name: &str,
) -> VortexResult<usize> {
    names
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| vortex_err!("Unknown field name {name}"))
}

/// Information about a field in a struct dtype
pub struct FieldInfo<'a> {
    /// The position index of the field within the enclosing struct
    pub index: usize,
    /// The name of the field
    pub name: &'a str,
    /// The dtype of the field
    pub dtype: fb::DType<'a>,
}

/// Get information about the referenced field, either by name or index
/// Returns an error if the field is not found
pub fn field_info<'a, 'b: 'a>(
    fb: fb::Struct_<'b>,
    field: &'a Field,
) -> VortexResult<FieldInfo<'b>> {
    let names = fb
        .names()
        .ok_or_else(|| vortex_err!("Missing field names"))?;
    let index = match field {
        Field::Name(name) => find_name(names, name)?,
        Field::Index(index) => {
            if *index
                >= fb
                    .names()
                    .ok_or_else(|| vortex_err!("Missing field names"))?
                    .len()
            {
                vortex_bail!("field index out of bounds: {}", index)
            }
            *index
        }
    };
    Ok(FieldInfo {
        index,
        name: names.get(index),
        dtype: fb
            .dtypes()
            .ok_or_else(|| vortex_err!("Missing dtypes"))?
            .get(index),
    })
}

/// Convert name references in projection list into index references.
///
/// This is mostly useful if you want to deduplicate multiple projections against serialized schema.
pub fn resolve_field<'a, 'b: 'a>(fb: fb::Struct_<'b>, field: &'a Field) -> VortexResult<usize> {
    match field {
        Field::Name(n) => {
            let names = fb
                .names()
                .ok_or_else(|| vortex_err!("Missing field names"))?;
            find_name(names, n)
        }
        Field::Index(i) => Ok(*i),
    }
}

/// Deserialize flatbuffer schema selecting only columns defined by projection
pub fn deserialize_and_project(
    fb_dtype: fb::DType<'_>,
    projection: &[Field],
) -> VortexResult<DType> {
    let fb_struct = fb_dtype
        .type__as_struct_()
        .ok_or_else(|| vortex_err!("The top-level type should be a struct"))?;
    let nullability = fb_struct.nullable().into();

    let (names, dtypes): (Vec<Arc<str>>, Vec<DType>) = projection
        .iter()
        .map(|f| resolve_field(fb_struct, f))
        .map(|idx| idx.and_then(|i| read_field(fb_struct, i)))
        .collect::<VortexResult<Vec<_>>>()?
        .into_iter()
        .unzip();

    Ok(DType::Struct(
        StructDType::new(names.into(), dtypes),
        nullability,
    ))
}

fn read_field(fb_struct: fb::Struct_, idx: usize) -> VortexResult<(Arc<str>, DType)> {
    let name = fb_struct
        .names()
        .ok_or_else(|| vortex_err!("Missing field names"))?
        .get(idx);
    let fb_dtype = fb_struct
        .dtypes()
        .ok_or_else(|| vortex_err!("Missing field dtypes"))?
        .get(idx);
    let dtype = DType::try_from(fb_dtype)?;

    Ok((name.into(), dtype))
}
