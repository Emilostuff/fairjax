use super::matched_ids::MatchedIds;
use super::matched_messages::MatchedMessages;
use crate::{CaseId, Store};

/// A successful case match containing the originating case ID and the message IDs that satisfied the pattern
pub struct CaseMatch {
    pub case: CaseId,
    pub messages: MatchedIds,
}

impl CaseMatch {
    /// Create a new case match
    pub fn new(case: CaseId, messages: MatchedIds) -> Self {
        CaseMatch { case, messages }
    }

    /// Return the length or number of messages in the CaseMatch
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Compute and return the fairest match from a collection of CaseMatches
    pub fn get_fairest(mut matches: Vec<CaseMatch>) -> Option<CaseMatch> {
        let max_pattern_len = matches.iter().map(|m| m.len()).max()?;

        // Find fairest match across cases
        let mut sorted_matches: Vec<_> = matches
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.messages.sorted(max_pattern_len)))
            .collect();
        sorted_matches.sort_by(|(_, a), (_, b)| a.cmp(&b));

        let fairest_match_index = *sorted_matches.first().map(|(i, _)| i)?;

        // Take fairest CaseMatch from vector efficiently
        Some(matches.swap_remove(fairest_match_index))
    }

    /// Convert a CaseMatch object to a MatchedMessages object by
    /// retrieving and removing messages from the store
    pub fn to_messages<M>(self, store: &mut Store<M>) -> MatchedMessages<M> {
        // Retrieve messages from store, and permanently remove
        let messages = self
            .messages
            .0
            .into_iter()
            .map(|id| store.remove(&id).unwrap())
            .collect();

        MatchedMessages {
            case: self.case,
            messages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_get_fairest_prefers_older_messages() {
        // Case 1 - Messages [1, 5]
        let case1 = CaseMatch::new(CaseId(1), MatchedIds(vec![id!(1), id!(5)]));
        // Case 2 - Messages [2, 3, 4, 5]
        let case2 = CaseMatch::new(CaseId(2), MatchedIds(vec![id!(2), id!(3), id!(4), id!(5)]));

        let result = CaseMatch::get_fairest(vec![case1, case2]).unwrap();

        // Case 2 contains the oldest messages
        assert_eq!(CaseId(1), result.case);
    }

    #[test]
    fn test_get_fairest_one_is_prefix() {
        // Case 1 - Messages [1, 2]
        let case1 = CaseMatch::new(CaseId(1), MatchedIds(vec![id!(1), id!(2)]));
        // Case 2 - Messages [1, 2, 3]
        let case2 = CaseMatch::new(CaseId(2), MatchedIds(vec![id!(1), id!(2), id!(3)]));

        let result = CaseMatch::get_fairest(vec![case1, case2]).unwrap();

        // Case 2 should be chosen as it consumes the same messaages as case 1
        // but additionally consumes more messages
        assert_eq!(CaseId(2), result.case);
    }

    #[test]
    fn test_get_fairest_same_messages_matched() {
        // Case 1 - Messages [1, 2, 3]
        let case1 = CaseMatch::new(CaseId(1), MatchedIds(vec![id!(1), id!(2), id!(3)]));
        // Case 2 - Messages [1, 2, 3]
        let case2 = CaseMatch::new(CaseId(2), MatchedIds(vec![id!(1), id!(2), id!(3)]));

        let result = CaseMatch::get_fairest(vec![case1, case2]).unwrap();

        // Case 1 should be chosen as it is the first case in the join-pattern
        assert_eq!(CaseId(1), result.case);
    }

    #[test]
    fn test_get_fairest_prefers_older_messages_many_patterns() {
        // Case 1 - Messages [7, 5, 10]
        let case1 = CaseMatch::new(CaseId(1), MatchedIds(vec![id!(7), id!(5), id!(10)]));
        // Case 2 - Messages [1, 4, 10]
        let case2 = CaseMatch::new(CaseId(2), MatchedIds(vec![id!(1), id!(4), id!(10)]));
        // Case 3 - Messages [2, 3, 10]
        let case3 = CaseMatch::new(CaseId(3), MatchedIds(vec![id!(2), id!(3), id!(10)]));
        // Case 4 - Messages [1, 5, 10]
        let case4 = CaseMatch::new(CaseId(4), MatchedIds(vec![id!(10), id!(1), id!(5)]));

        let result = CaseMatch::get_fairest(vec![case1, case2, case3, case4]).unwrap();

        // Case 2 contains the oldest messages
        assert_eq!(CaseId(2), result.case);
    }

    #[test]
    fn test_get_fairest_empty_list() {
        let matches: Vec<CaseMatch> = vec![];
        let result = CaseMatch::get_fairest(matches);

        assert!(result.is_none());
    }
}
