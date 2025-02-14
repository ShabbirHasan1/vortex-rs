use std::ops::Deref;

use num_traits::AsPrimitive;
use vortex_buffer::{Buffer, ByteBuffer};
use vortex_dtype::match_each_integer_ptype;
use vortex_error::{vortex_bail, VortexResult};
use vortex_mask::Mask;

use crate::array::{BinaryView, VarBinViewArray, VarBinViewEncoding};
use crate::builders::{ArrayBuilder, VarBinViewBuilder};
use crate::compute::TakeFn;
use crate::variants::PrimitiveArrayTrait;
use crate::{Array, IntoArray, IntoArrayVariant};

/// Take involves creating a new array that references the old array, just with the given set of views.
impl TakeFn<VarBinViewArray> for VarBinViewEncoding {
    fn take(&self, array: &VarBinViewArray, indices: &Array) -> VortexResult<Array> {
        // Compute the new validity

        // This is valid since all elements (of all arrays) even null values are inside must be the
        // min-max valid range.
        let validity = array.validity().take(indices)?;
        let indices = indices.clone().into_primitive()?;

        let views_buffer = match_each_integer_ptype!(indices.ptype(), |$I| {
        // This is valid since all elements even null values are inside the min-max valid range.
            take_views(array.views(), indices.as_slice::<$I>())
        });

        Ok(VarBinViewArray::try_new(
            views_buffer,
            array.buffers().collect(),
            array.dtype().with_nullability(
                (array.dtype().is_nullable() || indices.dtype().is_nullable()).into(),
            ),
            validity,
        )?
        .into_array())
    }

    unsafe fn take_unchecked(
        &self,
        array: &VarBinViewArray,
        indices: &Array,
    ) -> VortexResult<Array> {
        // Compute the new validity
        let validity = array.validity().take(indices)?;
        let indices = indices.clone().into_primitive()?;

        let views_buffer = match_each_integer_ptype!(indices.ptype(), |$I| {
            take_views_unchecked(array.views(), indices.as_slice::<$I>())
        });

        Ok(VarBinViewArray::try_new(
            views_buffer,
            array.buffers().collect(),
            array.dtype().clone(),
            validity,
        )?
        .into_array())
    }

    fn take_into(
        &self,
        array: &VarBinViewArray,
        indices: &Array,
        builder: &mut dyn ArrayBuilder,
    ) -> VortexResult<()> {
        let Some(builder) = builder.as_any_mut().downcast_mut::<VarBinViewBuilder>() else {
            vortex_bail!(
                "Cannot take_into a non-varbinview builder {:?}",
                builder.as_any().type_id()
            );
        };
        // Compute the new validity

        // This is valid since all elements (of all arrays) even null values are inside must be the
        // min-max valid range.
        // TODO(joe): impl validity_mask take
        let validity = array.validity().take(indices)?;
        let mask = validity.to_logical(array.len())?;
        let indices = indices.clone().into_primitive()?;

        match_each_integer_ptype!(indices.ptype(), |$I| {
            // This is valid since all elements even null values are inside the min-max valid range.
            take_views_into(array.views(), array.buffers(), indices.as_slice::<$I>(), mask, builder)?;
        });

        Ok(())
    }
}

fn take_views_into<I: AsPrimitive<usize>>(
    views: Buffer<BinaryView>,
    buffers: impl Iterator<Item = ByteBuffer>,
    indices: &[I],
    mask: Mask,
    _builder: &mut VarBinViewBuilder,
) -> VortexResult<()> {
    let buffers_offset = u32::try_from(_builder.completed_block_count())?;
    // NOTE(ngates): this deref is not actually trivial, so we run it once.
    let views_ref = views.deref();
    _builder.push_buffer_and_adjusted_views(
        buffers,
        indices.iter().map(|i| {
            let view = views_ref[i.as_()];
            // TODO(joe): can we make this branchless creating new value of bianry view and bit something
            // with is_inline?
            if view.is_inlined() {
                view
            } else {
                // Referencing views must have their buffer_index adjusted with new offsets
                let view_ref = view.as_view();
                BinaryView::new_view(
                    view.len(),
                    *view_ref.prefix(),
                    buffers_offset + view_ref.buffer_index(),
                    view_ref.offset(),
                )
            }
        }),
        mask,
    );
    Ok(())
}

fn take_views<I: AsPrimitive<usize>>(
    views: Buffer<BinaryView>,
    indices: &[I],
) -> Buffer<BinaryView> {
    // NOTE(ngates): this deref is not actually trivial, so we run it once.
    let views_ref = views.deref();
    Buffer::<BinaryView>::from_iter(indices.iter().map(|i| views_ref[i.as_()]))
}

fn take_views_unchecked<I: AsPrimitive<usize>>(
    views: Buffer<BinaryView>,
    indices: &[I],
) -> Buffer<BinaryView> {
    // NOTE(ngates): this deref is not actually trivial, so we run it once.
    let views_ref = views.deref();
    Buffer::from_iter(
        indices
            .iter()
            .map(|i| unsafe { *views_ref.get_unchecked(i.as_()) }),
    )
}

pub fn map_views(iter: impl IntoIterator<Item = BinaryView>, offset: u32) -> Vec<BinaryView> {
    iter.into_iter()
        .map(|view| {
            if view.is_inlined() {
                view
            } else {
                // Referencing views must have their buffer_index adjusted with new offsets
                let view_ref = view.as_view();
                BinaryView::new_view(
                    view_ref.size(),
                    *view_ref.prefix(),
                    offset + view_ref.buffer_index(),
                    view_ref.offset(),
                )
            }
        })
        .collect()
}

pub fn map_views_crazy(iter: impl IntoIterator<Item = BinaryView>, offset: u32) -> Vec<BinaryView> {
    iter.into_iter()
        .map(|view| {
            let value = view.as_u128();

            let block = ((value >> 64) & 0xFFFFFFFF) as u32;
            let block = if view.is_inlined() {
                block
            } else {
                block + offset
            };
            // println!("b {:b}", value & 0xFFFF_FFFF_0000_0000_FFFF_FFFF_FFFF_FFFF);
            let v = (value & 0xFFFF_FFFF_0000_0000_FFFF_FFFF_FFFF_FFFF) | ((block as u128) << 64);
            BinaryView::from(v)
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use itertools::Itertools;
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};

    use crate::array::{map_views, map_views_crazy, BinaryView};

    #[test]
    fn test_q_view_swap() {
        let mut rng = StdRng::seed_from_u64(23324);

        let views = (0..2000000)
            .map(|_| {
                if rng.gen_bool(0.5) {
                    BinaryView::new_inlined(&[])
                } else {
                    BinaryView::new_view(
                        rng.gen_range(0..1000),
                        [0u8; 4],
                        10,
                        rng.gen_range(0..1000),
                    )
                }
            })
            .collect::<Vec<_>>();

        let buffers_offset = 10;

        let res2 = map_views(views.clone(), buffers_offset);
        let res = map_views_crazy(views.clone(), buffers_offset);

        assert_eq!(
            res2.iter().map(|r| r.as_u128()).collect_vec(),
            res.iter().map(|r| r.as_u128()).collect_vec()
        )
    }
}
