use crate::{Mapping, MessageId};

pub struct Element {
    pub id: MessageId,
    pub indices: Vec<usize>,
}

impl Element {
    pub fn new(id: MessageId, indices: Vec<usize>) -> Self {
        Self { id, indices }
    }
}

pub struct Permutations {}

impl Permutations {
    pub fn get_permutations(elements: Vec<Element>) -> Vec<Mapping> {
        let mut elements: Vec<_> = elements.into_iter().enumerate().collect();
        elements.sort_by_key(|e| e.1.id);
        let mut output = Vec::new();
        let mut current: Vec<MessageId> = vec![0; elements.len()];
        let mut used = vec![false; elements.len()];

        Permutations::permute(&elements, &mut output, &mut current, &mut used);

        output
    }

    fn permute(
        elements: &[(usize, Element)],
        output: &mut Vec<Mapping>,
        current: &mut Mapping,
        used: &mut Vec<bool>,
    ) {
        if let Some((origin, element)) = elements.first() {
            for &i in element.indices.iter() {
                if !used[i] {
                    used[i] = true;
                    current[i] = *origin;
                    Permutations::permute(&elements[1..], output, current, used);
                    used[i] = false;
                }
            }
        } else {
            // Push valid permutation to output
            output.push(current.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_permutation() {
        // Element 1 can be in position 1
        // Element 2 can be in position 0
        let elements = vec![Element::new(1, vec![1]), Element::new(2, vec![0])];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(permutations.len(), 1);
        assert_eq!(permutations[0], vec![1, 0]);
    }

    #[test]
    fn test_two_options_permutation() {
        // Element 1 can be in position 0 or 1
        // Element 2 can be in position 0 or 1
        let elements = vec![Element::new(1, vec![0, 1]), Element::new(2, vec![0, 1])];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(permutations.len(), 2);
        assert!(permutations.contains(&vec![0, 1]));
        assert!(permutations.contains(&vec![1, 0]));
    }

    #[test]
    fn test_two_and_one_option_permutation() {
        // Element 1 can only be in position 0 or 1
        // Element 2 can only be in position 0 or 1
        // Element 3 can only be in position 2
        let elements = vec![
            Element::new(1, vec![0, 1]),
            Element::new(2, vec![0, 1]),
            Element::new(3, vec![2]),
        ];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(permutations.len(), 2);
        assert_eq!(permutations[0], vec![0, 1, 2]);
        assert_eq!(permutations[1], vec![1, 0, 2]);
    }

    #[test]
    fn test_permutation_with_limited_options() {
        // Element 1 can only be in position 0
        // Element 2 can only be in position 1
        // Element 3 can only be in position 2
        let elements = vec![
            Element::new(1, vec![0]),
            Element::new(2, vec![1]),
            Element::new(3, vec![2]),
        ];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(permutations.len(), 1);
        assert_eq!(permutations[0], vec![0, 1, 2]);
    }

    #[test]
    fn test_no_valid_permutations() {
        // Element 1 can be in position 0
        // Element 2 can be in position 0 (conflict)
        let elements = vec![Element::new(1, vec![0]), Element::new(2, vec![0])];

        let permutations = Permutations::get_permutations(elements);

        // There should be no valid permutations
        assert_eq!(permutations.len(), 0);
    }
}
