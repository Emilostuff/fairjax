use crate::{Mapping, MessageId};

/// Lexicographically sorted representation of matched message IDs for a case (padded to max pattern size across all cases).
/// This enables fairness comparisons across all cases.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct MatchedIdsSorted(pub(super) Vec<MessageId>);

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct MatchedIds(pub(super) Vec<MessageId>);

impl MatchedIds {
    #[cfg(test)]
    pub fn new(message_ids: Vec<MessageId>) -> Self {
        Self(message_ids)
    }

    pub fn from<const C: usize>(message_ids: [MessageId; C], mapping: Mapping<C>) -> Self {
        Self((0..C).map(|i| message_ids[mapping.get(i)]).collect())
    }

    /// Get Ids sorted lexicographically and padded
    pub fn sorted(&self, output_len: usize) -> MatchedIdsSorted {
        if output_len < self.len() {
            panic!("The output length cannot be less than the number of message IDs")
        }

        let mut output = self.0.clone();

        // Sort lexicographically
        output.sort();

        // Pad with max Ids
        while output.len() < output_len {
            output.push(MessageId::max());
        }

        MatchedIdsSorted(output)
    }

    pub fn contains(&self, id: &MessageId) -> bool {
        self.0.contains(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_from_with_identity_mapping() {
        let message_ids = [id!(10), id!(20), id!(30)];
        let mapping = Mapping::new([0, 1, 2]);

        let matched_ids = MatchedIds::from(message_ids, mapping);

        assert_eq!(vec![id!(10), id!(20), id!(30)], matched_ids.0);
    }

    #[test]
    fn test_from_with_reordered_mapping() {
        let message_ids = [id!(1), id!(2), id!(3)];
        let mapping = Mapping::new([2, 0, 1]);

        let matched_ids = MatchedIds::from(message_ids, mapping);

        assert_eq!(vec![id!(3), id!(1), id!(2)], matched_ids.0);
    }

    #[test]
    fn test_sorted_basic_no_padding() {
        let matched_ids = MatchedIds::new(vec![id!(3), id!(1), id!(2)]);
        let sorted = matched_ids.sorted(3);

        assert_eq!(vec![id!(1), id!(2), id!(3)], sorted.0);
    }

    #[test]
    fn test_sorted_with_padding() {
        let matched_ids = MatchedIds::new(vec![id!(3), id!(1)]);
        let sorted = matched_ids.sorted(4);

        assert_eq!(
            vec![id!(1), id!(3), MessageId::max(), MessageId::max()],
            sorted.0
        );
    }

    #[test]
    #[should_panic]
    fn test_output_length_less_than_message_count_causes_panic() {
        let matched_ids = MatchedIds::new(vec![id!(1), id!(2), id!(3)]);

        // We must not return only a portion of the message IDs
        matched_ids.sorted(2);
    }
}
