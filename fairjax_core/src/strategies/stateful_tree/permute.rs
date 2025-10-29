use crate::Mapping;

pub struct Element {
    pub indices: Vec<usize>,
}

impl Element {
    pub fn new(indices: Vec<usize>) -> Self {
        Self { indices }
    }
}

pub struct Permutations {}

impl Permutations {
    pub fn get_permutations<const C: usize>(elements: [Element; C]) -> Vec<Mapping<C>> {
        let elements: Vec<_> = elements.into_iter().enumerate().collect();
        let mut output = Vec::new();
        let mut current = [0; C];
        let mut used = [false; C];

        Permutations::permute(&elements, &mut output, &mut current, &mut used);

        output
    }

    fn permute<const C: usize>(
        elements: &[(usize, Element)],
        output: &mut Vec<Mapping<C>>,
        current: &mut [usize; C],
        used: &mut [bool; C],
    ) {
        if let Some((origin, element)) = elements.first() {
            for &i in element.indices.iter() {
                if !used[i] {
                    used[i] = true;
                    (*current)[i] = *origin;
                    Permutations::permute(&elements[1..], output, current, used);
                    used[i] = false;
                }
            }
        } else {
            // Push valid permutation to output
            output.push(Mapping::new(current.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_simple_permutation() {
        // Element 1 can be in position 1
        // Element 2 can be in position 0
        let elements = [Element::new(vec![1]), Element::new(vec![0])];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(1, permutations.len());
        assert_eq!(Mapping::new([1, 0]), permutations[0]);
    }

    #[test]
    fn test_two_options_permutation() {
        // Element 1 can be in position 0 or 1
        // Element 2 can be in position 0 or 1
        let elements = [Element::new(vec![0, 1]), Element::new(vec![0, 1])];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(2, permutations.len());
        assert_eq!(Mapping::new([0, 1]), permutations[0]);
        assert_eq!(Mapping::new([1, 0]), permutations[1]);
    }

    #[test]
    fn test_two_and_one_option_permutation() {
        // Element 1 can only be in position 0 or 1
        // Element 2 can only be in position 0 or 1
        // Element 3 can only be in position 2
        let elements = [
            Element::new(vec![0, 1]),
            Element::new(vec![0, 1]),
            Element::new(vec![2]),
        ];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(2, permutations.len());
        assert_eq!(Mapping::new([0, 1, 2]), permutations[0]);
        assert_eq!(Mapping::new([1, 0, 2]), permutations[1]);
    }

    #[test]
    fn test_permutation_with_limited_options() {
        // Element 1 can only be in position 0
        // Element 2 can only be in position 1
        // Element 3 can only be in position 2
        let elements = [
            Element::new(vec![0]),
            Element::new(vec![1]),
            Element::new(vec![2]),
        ];

        let permutations = Permutations::get_permutations(elements);

        assert_eq!(1, permutations.len());
        assert_eq!(Mapping::new([0, 1, 2]), permutations[0]);
    }
}
