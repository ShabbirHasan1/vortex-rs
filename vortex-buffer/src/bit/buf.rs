use crate::{buffer, ByteBuffer};

/// An immutable bitset stored as a packed byte buffer.
pub struct BitBuffer {
    buffer: ByteBuffer,
    len: usize,
    offset: usize,
}

impl BitBuffer {
    /// Create a new `BoolBuffer` backed by a [`ByteBuffer`] with `len` bits in view.
    ///
    /// Panics if the buffer is not large enough to hold `len` bits.
    pub fn new(buffer: ByteBuffer, len: usize) -> Self {
        assert!(
            buffer.len() * 8 >= len,
            "provided ByteBuffer not large enough to back BoolBuffer with len {len}"
        );

        Self {
            buffer,
            len,
            offset: 0,
        }
    }

    /// Create a new `BoolBuffer` backed by a [`ByteBuffer`] with `len` bits in view, starting at the
    /// given `offset` (in bits).
    ///
    /// Panics if the buffer is not large enough to hold `len` bits or if the offset is greater than
    pub fn new_with_offset(buffer: ByteBuffer, len: usize, offset: usize) -> Self {
        assert!(
            offset < len,
            "cannot construct new BoolBuffer with offset {offset} len {len}"
        );
        assert!(
            buffer.len() * 8 >= len - offset,
            "provided ByteBuffer not large enough to back BoolBuffer with offset {offset} len {len}"
        );

        Self {
            buffer,
            len,
            offset,
        }
    }

    /// Create a new `BoolBuffer` of length `len` where all bits are set (true).
    pub fn new_set(len: usize) -> Self {
        let words = len.div_ceil(8);
        let buffer = buffer![0xFF; words];

        Self {
            buffer,
            len,
            offset: 0,
        }
    }

    /// Create a new `BoolBuffer` of length `len` where all bits are unset (false).
    pub fn new_unset(len: usize) -> Self {
        let words = len.div_ceil(8);
        let buffer = buffer![0u8; words];

        Self {
            buffer,
            len,
            offset: 0,
        }
    }

    /// Get the logical length of this `BoolBuffer`.
    ///
    /// This may differ from the physical length of the backing buffer, for example if it was
    /// created using the `new_with_offset` constructor, or if it was sliced.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the `BoolBuffer` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Offset of start of the buffer in bits.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Retrieve the value at the given index.
    ///
    /// Panics if the index is out of bounds.
    pub fn value(&self, index: usize) -> bool {
        assert!(index < self.len, "index {index} exceeds len {}", self.len);

        let word_index = (self.offset + index) / 8;
        let bit_index = (self.offset + index) % 8;
        let word = self.buffer[word_index];
        let bit = word & (1 << bit_index);

        bit != 0
    }

    /// Create a new zero-copy slice of this BoolBuffer that begins at the `start` index and extends
    /// for `len` bits.
    ///
    /// Panics if the slice would extend beyond the end of the buffer.
    pub fn slice(&self, start: usize, len: usize) -> Self {
        assert!(
            start + len <= self.len,
            "slice of len {len} starting at {start} exceeds len {}",
            self.len
        );

        Self {
            buffer: self.buffer.clone(),
            len,
            offset: self.offset + start,
        }
    }

    /// Get the number of set bits in the buffer.
    pub fn true_count(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let first_word = self.offset.div_ceil(8);
        let last_word = (self.offset + self.len).div_ceil(8);

        let mut ones = 0;
        for word in first_word..last_word {
            let mut mask = 0xFF;
            if word == first_word {
                // Mask off the top bits
                mask &= 0xFF << (self.offset % 8);
            }

            if word == last_word {
                // Mask off the bottom bits
                mask &= 0xFF >> (8 - (self.offset + self.len) % 8);
            }

            ones += (self.buffer[word] & mask).count_ones();
        }

        ones as usize
    }

    /// Get the number of unset bits in the buffer.
    pub fn false_count(&self) -> usize {
        self.len - self.true_count()
    }
}

// Conversions

impl BitBuffer {
    /// Consumes this `BoolBuffer` and returns the backing `Buffer<u64>` with any offset
    /// and length information applied.
    pub fn into_inner(self) -> ByteBuffer {
        let word_start = self.offset / 8;
        let word_end = (self.offset + self.len).div_ceil(8);

        self.buffer.slice(word_start..word_end)
    }
}

#[cfg(test)]
mod tests {
    use crate::bit::BitBuffer;
    use crate::{buffer, ByteBuffer};

    #[test]
    fn test_bool() {
        // Create a new Buffer<u64> of length 1024 where the 8th bit is set.
        let buffer: ByteBuffer = buffer![1 << 7; 1024];
        let bools = BitBuffer::new(buffer, 1024 * 8);

        // sanity checks
        assert_eq!(bools.len(), 1024 * 8);
        assert!(!bools.is_empty());
        assert_eq!(bools.true_count(), 1024);
        assert_eq!(bools.false_count(), 1024 * 7);

        // Check all of the values
        for word in 0..1024 {
            for bit in 0..8 {
                if bit == 7 {
                    assert!(bools.value(word * 8 + bit));
                } else {
                    assert!(!bools.value(word * 8 + bit));
                }
            }
        }

        // Slice the buffer to create a new subset view.
        let sliced = bools.slice(64, 8);

        // sanity checks
        assert_eq!(sliced.len(), 8);
        assert!(!sliced.is_empty());
        assert_eq!(sliced.true_count(), 1);
        assert_eq!(sliced.false_count(), 7);

        // Check all of the values like before
        for bit in 0..8 {
            if bit == 7 {
                assert!(sliced.value(bit));
            } else {
                assert!(!sliced.value(bit));
            }
        }
    }
}
