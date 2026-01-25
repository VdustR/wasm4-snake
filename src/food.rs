use crate::rng::Rng;
use crate::snake::{Point, Snake, GRID_SIZE};

/// Food sizes
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FoodSize {
    Small,  // +1 segment, most common
    Medium, // +2 segments
    Large,  // +3 segments, rarest
}

impl FoodSize {
    /// Get growth amount for this food size
    pub const fn growth_amount(&self) -> usize {
        match self {
            FoodSize::Small => 1,
            FoodSize::Medium => 2,
            FoodSize::Large => 3,
        }
    }

    /// Get score value for this food size
    pub const fn score_value(&self) -> u32 {
        match self {
            FoodSize::Small => 10,
            FoodSize::Medium => 25,
            FoodSize::Large => 50,
        }
    }

    /// Get visual size (in pixels, for rendering)
    pub const fn visual_size(&self) -> u32 {
        match self {
            FoodSize::Small => 4,
            FoodSize::Medium => 6,
            FoodSize::Large => 8,
        }
    }

    /// Random food size based on probability
    /// Small: 60%, Medium: 30%, Large: 10%
    pub fn random(rng: &mut Rng) -> Self {
        let roll = rng.range(0, 100);
        if roll < 60 {
            FoodSize::Small
        } else if roll < 90 {
            FoodSize::Medium
        } else {
            FoodSize::Large
        }
    }
}

/// Food that the snake can eat
pub struct Food {
    pub position: Point,
    pub size: FoodSize,
}

impl Food {
    /// Create food at a random position not occupied by the snake
    pub fn new(rng: &mut Rng, snake: &Snake) -> Self {
        let (position, size) = Self::random_position_and_size(rng, snake);
        Self { position, size }
    }

    /// Move food to a new random position
    pub fn respawn(&mut self, rng: &mut Rng, snake: &Snake) {
        let (position, size) = Self::random_position_and_size(rng, snake);
        self.position = position;
        self.size = size;
    }

    /// Find a random position not occupied by the snake
    fn random_position_and_size(rng: &mut Rng, snake: &Snake) -> (Point, FoodSize) {
        let size = FoodSize::random(rng);

        // Safety limit to prevent infinite loop if grid is nearly full
        for _ in 0..1000 {
            let x = rng.range(0, GRID_SIZE);
            let y = rng.range(0, GRID_SIZE);
            let point = Point::new(x, y);

            if !snake.contains(point) {
                return (point, size);
            }
        }

        // Fallback: return corner (should rarely happen)
        (Point::new(0, 0), size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_not_on_snake() {
        let mut rng = Rng::new(12345);
        let snake = Snake::new();
        let food = Food::new(&mut rng, &snake);

        assert!(!snake.contains(food.position));
    }

    #[test]
    fn test_food_within_bounds() {
        let mut rng = Rng::new(12345);
        let snake = Snake::new();

        for _ in 0..100 {
            let food = Food::new(&mut rng, &snake);
            assert!(food.position.x >= 0 && food.position.x < GRID_SIZE);
            assert!(food.position.y >= 0 && food.position.y < GRID_SIZE);
        }
    }

    #[test]
    fn test_food_respawn() {
        let mut rng = Rng::new(12345);
        let snake = Snake::new();
        let mut food = Food::new(&mut rng, &snake);

        food.respawn(&mut rng, &snake);

        assert!(!snake.contains(food.position));
        assert!(food.position.x >= 0 && food.position.x < GRID_SIZE);
        assert!(food.position.y >= 0 && food.position.y < GRID_SIZE);
    }

    #[test]
    fn test_food_size_growth() {
        assert_eq!(FoodSize::Small.growth_amount(), 1);
        assert_eq!(FoodSize::Medium.growth_amount(), 2);
        assert_eq!(FoodSize::Large.growth_amount(), 3);
    }

    #[test]
    fn test_food_size_score() {
        assert_eq!(FoodSize::Small.score_value(), 10);
        assert_eq!(FoodSize::Medium.score_value(), 25);
        assert_eq!(FoodSize::Large.score_value(), 50);
    }

    #[test]
    fn test_food_size_distribution() {
        let mut rng = Rng::new(12345);
        let mut small = 0;
        let mut medium = 0;
        let mut large = 0;

        for _ in 0..1000 {
            match FoodSize::random(&mut rng) {
                FoodSize::Small => small += 1,
                FoodSize::Medium => medium += 1,
                FoodSize::Large => large += 1,
            }
        }

        // Roughly: Small ~60%, Medium ~30%, Large ~10%
        assert!(small > 500 && small < 700); // ~60%
        assert!(medium > 200 && medium < 400); // ~30%
        assert!(large > 50 && large < 200); // ~10%
    }
}
