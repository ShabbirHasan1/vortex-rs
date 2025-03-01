use std::cell::LazyCell;
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use vortex_array::arcref::ArcRef;
use vortex_array::serde::ArrayParts;
use vortex_array::vtable::{ComputeVTable, EncodingVTable, SerdeVTable, StatisticsVTable};
use vortex_array::{Array, ArrayContext, ArrayRef, EmptyMetadata, Encoding, EncodingId};
use vortex_dtype::DType;
use vortex_error::VortexResult;
use wasmtime::{Engine, Extern, Func, Instance, Module, Store};

use crate::v1::WasmArray;

#[derive(Clone, Debug)]
pub struct WasmEncoding {
    id: EncodingId,
    engine: Engine,
    module: Module,
}

impl WasmEncoding {
    pub fn try_from(engine: Engine, id: EncodingId, buffer: &[u8]) -> VortexResult<Self> {
        let module = Module::from_binary(&engine, buffer)?;
        // Check that the module contains the right exports?
        Ok(Self { id, engine, module })
    }
}

impl Encoding for WasmEncoding {
    type Array = WasmArray;
    type Metadata = EmptyMetadata;
}

impl EncodingVTable for WasmEncoding {
    fn id(&self) -> EncodingId {
        self.id.clone()
    }
}

impl<'a> SerdeVTable<&'a dyn Array> for WasmEncoding {
    fn decode(
        &self,
        parts: &ArrayParts,
        ctx: &ArrayContext,
        dtype: DType,
        len: usize,
    ) -> VortexResult<ArrayRef> {
        struct State {
            ctx: ArrayContext,
        }
        let mut store = Store::new(&self.engine, State { ctx: ctx.clone() });
        let instance = Instance::new(&mut store, &self.module, &[
            Extern::Func(Func::new(&mut store, |state: &mut State, index: i32| {
                let ctx = &state.ctx;
                let array = ctx.get_array(index as usize);
                let len = array.len();
                Ok(len as i32)
            }, Func::type_of(&self.engine, wasm_bindgen::signature::WasmFunc::new(|i32, i32| i32)))),
        ])?;

        #[wasm_bindgen]
        extern "C" {
            /// Decompress a serialized Vortex array into a canonical form, returning the IPC representation.
            fn to_canonical(serialized_array: &[u8]) -> Vec<u8>;
        }

        let decoded = LazyCell::new(|| {


        })

        Ok(Arc::new(WasmArray {
            dtype,
            len,
            parts: parts.clone(),
            stats_set: Default::default(),
            vtable: ArcRef::new_arc(Arc::new(self.clone())),
            decoded: None,
            instance,
        }))
    }
}

impl ComputeVTable for WasmEncoding {}

impl<'a> StatisticsVTable<&'a dyn Array> for WasmEncoding {}
