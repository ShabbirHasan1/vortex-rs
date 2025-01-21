use arrow_array::BooleanArray;
use vortex_error::{vortex_bail, VortexError, VortexExpect, VortexResult};
use vortex_mask::{Mask, MaskIter};

use crate::array::ConstantArray;
use crate::arrow::FromArrowArray;
use crate::compute::scalar_at;
use crate::encoding::Encoding;
use crate::{ArrayDType, ArrayData, Canonical, IntoArrayData, IntoCanonical};

pub type FilterMask = Mask;
pub type FilterIter<'a> = MaskIter<'a>;

pub trait FilterFn<Array> {
    /// Filter an array by the provided predicate.
    fn filter(&self, array: &Array, mask: &FilterMask) -> VortexResult<ArrayData>;
}

impl<E: Encoding> FilterFn<ArrayData> for E
where
    E: FilterFn<E::Array>,
    for<'a> &'a E::Array: TryFrom<&'a ArrayData, Error = VortexError>,
{
    fn filter(&self, array: &ArrayData, mask: &FilterMask) -> VortexResult<ArrayData> {
        let (array_ref, encoding) = array.try_downcast_ref::<E>()?;
        FilterFn::filter(encoding, array_ref, mask)
    }
}

/// Return a new array by applying a boolean predicate to select items from a base Array.
///
/// # Performance
///
/// This function attempts to amortize the cost of copying
///
/// # Panics
///
/// The `predicate` must receive an Array with type non-nullable bool, and will panic if this is
/// not the case.
pub fn filter(array: &ArrayData, mask: &FilterMask) -> VortexResult<ArrayData> {
    if mask.len() != array.len() {
        vortex_bail!(
            "mask.len() is {}, does not equal array.len() of {}",
            mask.len(),
            array.len()
        );
    }

    let true_count = mask.true_count();

    // Fast-path for empty mask.
    if true_count == 0 {
        return Ok(Canonical::empty(array.dtype())?.into());
    }

    // Fast-path for full mask
    if true_count == mask.len() {
        return Ok(array.clone());
    }

    let filtered = filter_impl(array, mask)?;

    debug_assert_eq!(
        filtered.len(),
        true_count,
        "Filter length mismatch {}",
        array.encoding().id()
    );
    debug_assert_eq!(
        filtered.dtype(),
        array.dtype(),
        "Filter dtype mismatch {}",
        array.encoding().id()
    );

    Ok(filtered)
}

fn filter_impl(array: &ArrayData, mask: &FilterMask) -> VortexResult<ArrayData> {
    if let Some(filter_fn) = array.encoding().filter_fn() {
        return filter_fn.filter(array, mask);
    }

    // We can use scalar_at if the mask has length 1.
    if mask.true_count() == 1 && array.encoding().scalar_at_fn().is_some() {
        let idx = mask.first().vortex_expect("true_count == 1");
        return Ok(ConstantArray::new(scalar_at(array, idx)?, 1).into_array());
    }

    // Fallback: implement using Arrow kernels.
    log::debug!(
        "No filter implementation found for {}",
        array.encoding().id(),
    );

    let array_ref = array.clone().into_arrow()?;
    let mask_array = BooleanArray::new(mask.boolean_buffer().clone(), None);
    let filtered = arrow_select::filter::filter(array_ref.as_ref(), &mask_array)?;

    Ok(ArrayData::from_arrow(filtered, array.dtype().is_nullable()))
}
