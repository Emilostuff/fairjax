use crate::CaseId;
use itertools::Itertools;
use itertools::traits::HomogeneousTuple;

#[allow(unused)]
pub struct MatchedMessages<M> {
    pub(super) case: CaseId,
    pub(super) messages: Vec<M>,
}

// Macro to generate tuple conversion methods using collect_tuple
macro_rules! impl_tuple_method {
    ($name:ident, $tuple:ty) => {
        pub fn $name(self) -> $tuple
        where
            $tuple: HomogeneousTuple<Item = M>,
        {
            self.messages.into_iter().collect_tuple().unwrap()
        }
    };
}

impl<M> MatchedMessages<M> {
    pub fn case_id(&self) -> &CaseId {
        &self.case
    }

    impl_tuple_method!(into_1_tuple, (M,));
    impl_tuple_method!(into_2_tuple, (M, M));
    impl_tuple_method!(into_3_tuple, (M, M, M));
    impl_tuple_method!(into_4_tuple, (M, M, M, M));
    impl_tuple_method!(into_5_tuple, (M, M, M, M, M));
    impl_tuple_method!(into_6_tuple, (M, M, M, M, M, M));
    impl_tuple_method!(into_7_tuple, (M, M, M, M, M, M, M));
    impl_tuple_method!(into_8_tuple, (M, M, M, M, M, M, M, M));
    impl_tuple_method!(into_9_tuple, (M, M, M, M, M, M, M, M, M));
    impl_tuple_method!(into_10_tuple, (M, M, M, M, M, M, M, M, M, M));
    impl_tuple_method!(into_11_tuple, (M, M, M, M, M, M, M, M, M, M, M));
    impl_tuple_method!(into_12_tuple, (M, M, M, M, M, M, M, M, M, M, M, M));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CaseId;

    #[test]
    fn test_case_id() {
        let mm: MatchedMessages<u32> = MatchedMessages {
            case: CaseId(42),
            messages: vec![1],
        };
        assert_eq!(mm.case_id(), &CaseId(42));
    }

    #[test]
    fn test_into_1_tuple() {
        let mm: MatchedMessages<u32> = MatchedMessages {
            case: CaseId(1),
            messages: vec![10],
        };
        let tuple = mm.into_1_tuple();
        assert_eq!(tuple, (10,));
    }

    #[test]
    fn test_into_3_tuple() {
        let mm: MatchedMessages<u32> = MatchedMessages {
            case: CaseId(2),
            messages: vec![1, 2, 3],
        };
        let tuple = mm.into_3_tuple();
        assert_eq!(tuple, (1, 2, 3));
    }

    #[test]
    fn test_into_5_tuple() {
        let mm: MatchedMessages<u32> = MatchedMessages {
            case: CaseId(3),
            messages: vec![5, 6, 7, 8, 9],
        };
        let tuple = mm.into_5_tuple();
        assert_eq!(tuple, (5, 6, 7, 8, 9));
    }
}
