/// A lookup table that defines how to reorder messages from their stored sequence
/// into a pattern-specific sequence. For pattern position `i`, `mapping[i]` gives
/// the stored sequence index where that message should be retrieved from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mapping<const C: usize>(pub [usize; C]);

impl<const C: usize> Mapping<C> {
    #[inline]
    /// Create a new Mapping from an array of indices
    pub fn new(indices: [usize; C]) -> Self {
        Mapping(indices)
    }

    #[inline]
    /// Return the value stored at `index`, out of range queries will panic
    pub fn get(&self, index: usize) -> usize {
        self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_new_and_get() {
        let indices = [2, 0, 1];
        let mapping = Mapping::<3>::new(indices);
        assert_eq!(mapping.get(0), 2);
        assert_eq!(mapping.get(1), 0);
        assert_eq!(mapping.get(2), 1);
    }

    #[test]
    fn test_mapping_equality() {
        let a = Mapping::<3>::new([1, 2, 0]);
        let b = Mapping::<3>::new([1, 2, 0]);
        let c = Mapping::<3>::new([0, 1, 2]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    #[should_panic]
    fn test_mapping_get_out_of_bounds() {
        let mapping = Mapping::<2>::new([0, 1]);

        // This should panic as index 2 is out of bounds
        mapping.get(2);
    }
}
