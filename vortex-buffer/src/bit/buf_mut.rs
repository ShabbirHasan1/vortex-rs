use vortex_error::VortexExpect;

use crate::{buffer_mut, BitBuffer, BufferMut, ByteBufferMut};

/// A mutable bitset buffer that allows random access to individual bits for set and get.
///
///
/// # Example
/// ```
/// use vortex_buffer::BitBufferMut;
///
/// let mut bools = BitBufferMut::new_unset(10);
/// bools.set_to(9, true);
/// for i in 0..9 {
///    assert!(!bools.value(i));
/// }
/// assert!(bools.value(9));
///
/// // Freeze into a new bools vector.
/// let bools = bools.freeze();
/// ```
///
/// See also: [`crate::BitBuffer`].
pub struct BitBufferMut {
    buffer: ByteBufferMut,
    len: usize,
    capacity: usize,
}

impl BitBufferMut {
    /// Create a new empty mutable bit buffer with requested capacity (in bits).
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BufferMut::with_capacity(capacity.div_ceil(8)),
            len: 0,
            capacity,
        }
    }

    /// Create a new mutable buffer with requested `len` and all bits set to `true`.
    pub fn new_set(len: usize) -> Self {
        Self {
            buffer: buffer_mut![0xFF; len.div_ceil(8)],
            capacity: len,
            len,
        }
    }

    /// Create a new mutable buffer with requested `len` and all bits set to `false`.
    pub fn new_unset(len: usize) -> Self {
        Self {
            buffer: buffer_mut![0u8; len.div_ceil(8)],
            capacity: len,
            len,
        }
    }

    /// Get the current populated length of the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True if the buffer has length 0.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the value at the requested index.
    pub fn value(&self, index: usize) -> bool {
        assert!(index < self.len, "index {index} exceeds len {}", self.len);

        let word = self.buffer[index / 8];
        let bit = word & (1 << (index % 8));

        bit != 0
    }

    /// Set the bit at `index` to the given boolean value.
    ///
    /// This operation is checked so if `index` exceeds the buffer length, this will panic.
    pub fn set_to(&mut self, index: usize, value: bool) {
        if value {
            self.set(index);
        } else {
            self.unset(index);
        }
    }

    /// Set a position to `true`.
    ///
    /// This operation is checked so if `index` exceeds the buffer length, this will panic.
    pub fn set(&mut self, index: usize) {
        assert!(index < self.len, "index {index} exceeds len {}", self.len);

        // SAFETY: checked by assertion
        unsafe { self.set_unchecked(index) };
    }

    /// Set a position to `false`.
    ///
    /// This operation is checked so if `index` exceeds the buffer length, this will panic.
    pub fn unset(&mut self, index: usize) {
        assert!(index < self.len, "index {index} exceeds len {}", self.len);

        // SAFETY: checked by assertion
        unsafe { self.unset_unchecked(index) };
    }

    /// Set the bit at `index` to `true` without checking bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `index` does not exceed the largest bit index in the backing buffer.
    pub unsafe fn set_unchecked(&mut self, index: usize) {
        let word_index = index / 8;
        let bit_index = index % 8;
        // SAFETY: checked by caller
        unsafe {
            let word = self.buffer.as_mut_ptr().add(word_index);
            word.write(*word | 1 << bit_index);
        }
    }

    /// Unset the bit at `index` without checking bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `index` does not exceed the largest bit index in the backing buffer.
    pub unsafe fn unset_unchecked(&mut self, index: usize) {
        let word_index = index / 8;
        let bit_index = index % 8;

        // SAFETY: checked by caller
        unsafe {
            let word = self.buffer.as_mut_ptr().add(word_index);
            word.write(*word & !(1 << bit_index));
        }
    }

    /// Append a new boolean into the bit buffer, incrementing the length.
    ///
    /// Panics if the buffer is full.
    pub fn append(&mut self, value: bool) {
        if value {
            self.append_true()
        } else {
            self.append_false()
        }
    }

    /// Append a new true value to the buffer.
    ///
    /// Panics if there is no remaining capacity.
    pub fn append_true(&mut self) {
        assert!(
            self.len < self.capacity,
            "cannot append to full BitBufferMut"
        );

        if self.len % 8 == 0 {
            // Push a new word that starts with 1
            self.buffer.push(1u8);
        } else {
            // Push a 1 bit into the current word.
            let word = self.buffer.last_mut().vortex_expect("buffer is not empty");
            *word |= 1 << (self.len % 8);
        }

        self.len += 1;
    }

    /// Append a new false value to the buffer.
    ///
    /// Panics if there is no remaining capacity.
    pub fn append_false(&mut self) {
        assert!(
            self.len < self.capacity,
            "cannot append to full BitBufferMut"
        );

        if self.len % 8 == 0 {
            // push new word that starts with 0
            self.buffer.push(0u8);
        }

        self.len += 1;
    }
    /// Append several boolean values into the bit buffer. After this operation,
    /// the length will be incremented by `n`.
    ///
    /// Panics if the buffer does not have `n` slots left.
    pub fn append_n(&mut self, value: bool, n: usize) {
        // Implementation is largely borrowed from arrow::BooleanBufferBuilder::append_n
        assert!(
            self.len.saturating_add(n) <= self.capacity,
            "cannot append {n} entries to BitBufferMut with len {} capacity {}",
            self.len,
            self.capacity
        );

        match value {
            true => {
                let new_len = self.len + n;
                let new_len_bytes = new_len.div_ceil(8);
                let cur_remainder = self.len % 8;
                let new_remainder = new_len % 8;

                if cur_remainder != 0 {
                    // Pad cur_remainder high bits with 1s
                    *self
                        .buffer
                        .as_mut_slice()
                        .last_mut()
                        .vortex_expect("buffer is not empty") |= !((1 << cur_remainder) - 1);
                }

                // Push several full bytes.
                if new_len_bytes > self.buffer.len() {
                    // Push full bytes, except for the final byte.
                    self.buffer.push_n(0xFF, new_len_bytes - self.buffer.len());
                }

                // Patch zeros into remainder of last byte pushed
                if new_remainder > 0 {
                    // Set the new_remainder LSB to 1
                    *self
                        .buffer
                        .as_mut_slice()
                        .last_mut()
                        .vortex_expect("buffer is not empty") &= (1 << new_remainder) - 1;
                }
            }
            false => {
                let new_len = self.len + n;
                let new_len_bytes = new_len.div_ceil(8);

                // push new 0 bytes.
                if new_len_bytes > self.buffer.len() {
                    self.buffer.push_n(0, new_len_bytes - self.buffer.len());
                }
            }
        }

        self.len += n;
    }

    /// Freeze the buffer in its current state into an immutable `BoolBuffer`.
    pub fn freeze(self) -> BitBuffer {
        BitBuffer::new(self.buffer.freeze().into_byte_buffer(), self.len)
    }
}

#[cfg(test)]
mod tests {
    use crate::bit::buf_mut::BitBufferMut;

    #[test]
    fn test_bits_mut() {
        let mut bools = BitBufferMut::new_unset(10);
        bools.set_to(0, true);
        bools.set_to(9, true);

        let bools = bools.freeze();
        assert!(bools.value(0));
        for i in 1..=8 {
            assert!(!bools.value(i));
        }
        assert!(bools.value(9));
    }

    #[test]
    fn test_append_n() {
        let mut bools = BitBufferMut::new(10);
        assert_eq!(bools.len(), 0);
        assert!(bools.is_empty());

        bools.append(true);
        bools.append_n(false, 8);
        bools.append_n(true, 1);

        let bools = bools.freeze();

        assert_eq!(bools.true_count(), 2);
        assert!(bools.value(0));
        assert!(bools.value(9));
    }
}
