use vortex_dtype::DType;
use wasm_bindgen::prelude::*;

#[repr(C)]
pub struct WArray {
    len: usize,
    dtype: DType,
}

#[wasm_bindgen]
extern "C" {}
