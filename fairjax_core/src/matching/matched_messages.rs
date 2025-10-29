use crate::CaseId;
use itertools::Itertools;
use itertools::traits::HomogeneousTuple;

#[allow(unused)]
pub struct MatchedMessages<M> {
    pub(super) case: CaseId,
    pub(super) messages: Vec<M>,
}

// Macro to generate tuple conversion methods using collect
macro_rules! impl_get_n_method {
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

    pub fn into_1(self) -> M {
        self.messages.into_iter().next().unwrap()
    }
    impl_get_n_method!(into_2, (M, M));
    impl_get_n_method!(into_3, (M, M, M));
    impl_get_n_method!(into_4, (M, M, M, M));
    impl_get_n_method!(into_5, (M, M, M, M, M));
    impl_get_n_method!(into_6, (M, M, M, M, M, M));
    impl_get_n_method!(into_7, (M, M, M, M, M, M, M));
    impl_get_n_method!(into_8, (M, M, M, M, M, M, M, M));
    impl_get_n_method!(into_9, (M, M, M, M, M, M, M, M, M));
    impl_get_n_method!(into_10, (M, M, M, M, M, M, M, M, M, M));
    impl_get_n_method!(into_11, (M, M, M, M, M, M, M, M, M, M, M));
    impl_get_n_method!(into_12, (M, M, M, M, M, M, M, M, M, M, M, M));
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
    fn test_into_1() {
        let mm: MatchedMessages<u32> = MatchedMessages {
            case: CaseId(1),
            messages: vec![10],
        };
        let tuple = mm.into_1();
        assert_eq!(tuple, 10);
    }

    #[test]
    fn test_into_3() {
        let mm: MatchedMessages<u32> = MatchedMessages {
            case: CaseId(2),
            messages: vec![1, 2, 3],
        };
        let tuple = mm.into_3();
        assert_eq!(tuple, (1, 2, 3));
    }

    #[test]
    fn test_into_5() {
        let mm: MatchedMessages<u32> = MatchedMessages {
            case: CaseId(3),
            messages: vec![5, 6, 7, 8, 9],
        };
        let tuple = mm.into_5();
        assert_eq!(tuple, (5, 6, 7, 8, 9));
    }
}
