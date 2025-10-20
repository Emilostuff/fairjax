pub mod scenarios {
    pub mod many_to_one;
    pub mod pairs;
    pub mod sums;
    pub mod transitive;
}

use rand::SeedableRng;

#[derive(Clone, Debug, PartialEq)]
pub struct MatchTrace<T: PartialEq + Clone + std::fmt::Debug> {
    pattern_no: usize,
    messages: Vec<T>,
}

impl<T: PartialEq + Clone + std::fmt::Debug> MatchTrace<T> {
    pub fn new(pattern_no: usize, messages: Vec<T>) -> Self {
        MatchTrace {
            pattern_no,
            messages,
        }
    }
}

impl<T: PartialEq + Clone + std::fmt::Debug> std::fmt::Display for MatchTrace<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.pattern_no, self.messages)
    }
}

pub fn get_rng(seed: Option<u64>) -> rand::rngs::StdRng {
    if let Some(seed) = seed {
        rand::rngs::StdRng::seed_from_u64(seed)
    } else {
        rand::rngs::StdRng::from_os_rng()
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use rand::seq::SliceRandom;

    use super::*;

    #[test]
    fn test_get_rng_idempotence() {
        let seed = Some(42u64);
        let mut rng1 = get_rng(seed);
        let mut rng2 = get_rng(seed);

        // Generate a sequence of random numbers from both RNGs and compare
        let mut vals1: Vec<u32> = (0..20).map(|_| rng1.random_range(0..10000)).collect();
        let mut vals2: Vec<u32> = (0..20).map(|_| rng2.random_range(0..10000)).collect();

        assert_eq!(
            vals1, vals2,
            "RNGs with the same seed should produce the same sequence"
        );

        // shuffle the vectors
        vals1.shuffle(&mut rng1);
        vals2.shuffle(&mut rng2);

        assert_eq!(vals1, vals2, "Shuffle should produce the same sequence");
    }
}
