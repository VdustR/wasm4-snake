/// Simple Linear Congruential Generator for random numbers
/// Uses parameters from Numerical Recipes
pub struct Rng {
    state: u32,
}

impl Rng {
    /// Create a new RNG with the given seed
    pub const fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Re-seed the generator
    pub fn seed(&mut self, seed: u32) {
        self.state = seed;
    }

    /// Generate the next random u32
    pub fn next_u32(&mut self) -> u32 {
        // LCG parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    /// Generate a random i32 in the range [min, max)
    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u32;
        min + (self.next_u32() % range) as i32
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(12345)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u32(), rng2.next_u32());
        }
    }

    #[test]
    fn test_different_seeds_different_output() {
        let mut rng1 = Rng::new(1);
        let mut rng2 = Rng::new(2);

        // Very unlikely to be equal with different seeds
        assert_ne!(rng1.next_u32(), rng2.next_u32());
    }

    #[test]
    fn test_range_bounds() {
        let mut rng = Rng::new(12345);

        for _ in 0..1000 {
            let value = rng.range(0, 20);
            assert!(value >= 0 && value < 20, "value {} out of range", value);
        }
    }

    #[test]
    fn test_range_negative() {
        let mut rng = Rng::new(12345);

        for _ in 0..1000 {
            let value = rng.range(-10, 10);
            assert!(value >= -10 && value < 10, "value {} out of range", value);
        }
    }

    #[test]
    fn test_range_single_value() {
        let mut rng = Rng::new(12345);
        // When min == max, should return min
        assert_eq!(rng.range(5, 5), 5);
    }

    #[test]
    fn test_reseed() {
        let mut rng = Rng::new(42);
        let first = rng.next_u32();

        rng.seed(42);
        let after_reseed = rng.next_u32();

        assert_eq!(first, after_reseed);
    }
}
