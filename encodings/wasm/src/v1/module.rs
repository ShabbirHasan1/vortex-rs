use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// Decompress a serialized Vortex array into a canonical form, returning the IPC representation.
    fn to_canonical(serialized_array: &[u8]) -> Vec<u8>;
}
