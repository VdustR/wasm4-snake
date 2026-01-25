/// Grid size: 160px screen / 8px per cell = 20 cells
pub const GRID_SIZE: i32 = 20;
/// Maximum snake length (reduced for stack safety in WASM-4)
pub const MAX_SNAKE_LENGTH: usize = 50;
/// Minimum snake length (initial state, death threshold)
pub const MIN_SNAKE_LENGTH: usize = 3;
/// Maximum energy for speed boost (5 = can boost 5 times)
pub const MAX_ENERGY: u8 = 5;
/// Initial energy when game starts
pub const INITIAL_ENERGY: u8 = 3;

/// A point on the game grid
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Movement direction
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Convert direction to movement delta
    pub const fn delta(self) -> Point {
        match self {
            Direction::Up => Point::new(0, -1),
            Direction::Down => Point::new(0, 1),
            Direction::Left => Point::new(-1, 0),
            Direction::Right => Point::new(1, 0),
        }
    }

    /// Get the opposite direction
    pub const fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// The snake entity
pub struct Snake {
    /// Body segments, head is at index 0
    pub body: [Point; MAX_SNAKE_LENGTH],
    /// Current length of the snake
    pub length: usize,
    /// Current movement direction
    pub direction: Direction,
    /// Energy for speed boost
    pub energy: u8,
}

impl Snake {
    /// Create a new snake at the center of the screen
    pub fn new() -> Self {
        let mut body = [Point::default(); MAX_SNAKE_LENGTH];
        // Start with MIN_SNAKE_LENGTH segments in the middle, moving right
        body[0] = Point::new(10, 10); // Head
        body[1] = Point::new(9, 10);
        body[2] = Point::new(8, 10);

        Self {
            body,
            length: MIN_SNAKE_LENGTH,
            direction: Direction::Right,
            energy: INITIAL_ENERGY,
        }
    }

    /// Get the head position
    pub fn head(&self) -> Point {
        self.body[0]
    }

    /// Peek at where the head would be after moving (without actually moving)
    pub fn peek_next_head(&self) -> Point {
        let delta = self.direction.delta();
        Point::new(
            (self.body[0].x + delta.x).rem_euclid(GRID_SIZE),
            (self.body[0].y + delta.y).rem_euclid(GRID_SIZE),
        )
    }

    /// Move the snake one step in the current direction
    pub fn update(&mut self) {
        // Move each segment to the position of the segment in front
        for i in (1..self.length).rev() {
            self.body[i] = self.body[i - 1];
        }

        // Move head in the current direction with wrapping
        let delta = self.direction.delta();
        self.body[0] = Point::new(
            (self.body[0].x + delta.x).rem_euclid(GRID_SIZE),
            (self.body[0].y + delta.y).rem_euclid(GRID_SIZE),
        );
    }

    /// Grow the snake by one segment
    pub fn grow(&mut self) {
        if self.length < MAX_SNAKE_LENGTH {
            // Duplicate the tail segment; it will separate on next update
            self.body[self.length] = self.body[self.length - 1];
            self.length += 1;
        }
    }

    /// Try to change direction (ignores 180-degree turns)
    pub fn set_direction(&mut self, new_dir: Direction) {
        if new_dir != self.direction.opposite() {
            self.direction = new_dir;
        }
    }

    /// Check if head collides with any body segment
    pub fn collides_with_self(&self) -> bool {
        let head = self.head();
        self.body[1..self.length].contains(&head)
    }

    /// Check if a point is occupied by the snake
    pub fn contains(&self, point: Point) -> bool {
        self.body[..self.length].contains(&point)
    }

    /// Shrink the snake by one segment. Returns false if at minimum length (would die).
    pub fn shrink(&mut self) -> bool {
        if self.length <= MIN_SNAKE_LENGTH {
            return false; // Would die
        }
        self.length -= 1;
        true
    }

    /// Try to grow or gain energy based on length limit.
    /// - If below max_length: grow normally
    /// - If at max_length: gain energy instead
    ///
    /// Returns true if grew, false if gained energy (or at max energy)
    pub fn try_grow_or_energy(&mut self, max_length: usize) -> bool {
        if self.length < max_length && self.length < MAX_SNAKE_LENGTH {
            self.grow();
            true
        } else {
            // At max length, gain energy instead
            if self.energy < MAX_ENERGY {
                self.energy += 1;
            }
            false
        }
    }

    /// Use one energy point. Returns false if no energy available.
    pub fn use_energy(&mut self) -> bool {
        if self.energy > 0 {
            self.energy -= 1;
            true
        } else {
            false
        }
    }
}

impl Default for Snake {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_equality() {
        let p1 = Point::new(5, 10);
        let p2 = Point::new(5, 10);
        let p3 = Point::new(5, 11);

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Up.delta(), Point::new(0, -1));
        assert_eq!(Direction::Down.delta(), Point::new(0, 1));
        assert_eq!(Direction::Left.delta(), Point::new(-1, 0));
        assert_eq!(Direction::Right.delta(), Point::new(1, 0));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn test_snake_initial_state() {
        let snake = Snake::new();
        assert_eq!(snake.length, 3);
        assert_eq!(snake.head(), Point::new(10, 10));
        assert_eq!(snake.direction, Direction::Right);
    }

    #[test]
    fn test_snake_movement() {
        let mut snake = Snake::new();
        let initial_head = snake.head();
        snake.update();
        assert_eq!(snake.head(), Point::new(initial_head.x + 1, initial_head.y));
    }

    #[test]
    fn test_snake_cannot_reverse() {
        let mut snake = Snake::new();
        assert_eq!(snake.direction, Direction::Right);

        // Try to go left (opposite) - should be ignored
        snake.set_direction(Direction::Left);
        assert_eq!(snake.direction, Direction::Right);

        // Can go up
        snake.set_direction(Direction::Up);
        assert_eq!(snake.direction, Direction::Up);
    }

    #[test]
    fn test_snake_grow() {
        let mut snake = Snake::new();
        let initial_length = snake.length;
        snake.grow();
        assert_eq!(snake.length, initial_length + 1);
    }

    #[test]
    fn test_snake_wrap_around_right() {
        let mut snake = Snake::new();
        snake.body[0] = Point::new(GRID_SIZE - 1, 10);
        snake.set_direction(Direction::Right);
        snake.update();
        assert_eq!(snake.head().x, 0);
    }

    #[test]
    fn test_snake_wrap_around_left() {
        let mut snake = Snake::new();
        snake.body[0] = Point::new(0, 10);
        // Initial direction is Right, change to Up first, then Left
        snake.set_direction(Direction::Up);
        snake.set_direction(Direction::Left);
        snake.update();
        assert_eq!(snake.head().x, GRID_SIZE - 1);
    }

    #[test]
    fn test_snake_wrap_around_top() {
        let mut snake = Snake::new();
        snake.body[0] = Point::new(10, 0);
        snake.set_direction(Direction::Up);
        snake.update();
        assert_eq!(snake.head().y, GRID_SIZE - 1);
    }

    #[test]
    fn test_snake_wrap_around_bottom() {
        let mut snake = Snake::new();
        snake.body[0] = Point::new(10, GRID_SIZE - 1);
        snake.set_direction(Direction::Down);
        snake.update();
        assert_eq!(snake.head().y, 0);
    }

    #[test]
    fn test_self_collision_false() {
        let snake = Snake::new();
        assert!(!snake.collides_with_self());
    }

    #[test]
    fn test_self_collision_true() {
        let mut snake = Snake::new();
        // Manually create a collision scenario
        snake.body[0] = Point::new(5, 5);
        snake.body[1] = Point::new(5, 5);
        snake.length = 2;
        assert!(snake.collides_with_self());
    }

    #[test]
    fn test_contains() {
        let snake = Snake::new();
        assert!(snake.contains(Point::new(10, 10))); // Head
        assert!(snake.contains(Point::new(9, 10))); // Body
        assert!(!snake.contains(Point::new(0, 0))); // Empty space
    }

    #[test]
    fn test_snake_shrink() {
        let mut snake = Snake::new();
        // Grow first so we can shrink
        snake.grow();
        snake.grow();
        assert_eq!(snake.length, 5);

        // Shrink should succeed
        assert!(snake.shrink());
        assert_eq!(snake.length, 4);

        // Shrink to min length
        snake.shrink();
        assert_eq!(snake.length, 3);

        // Cannot shrink below min length
        assert!(!snake.shrink());
        assert_eq!(snake.length, MIN_SNAKE_LENGTH);
    }

    #[test]
    fn test_snake_energy() {
        let mut snake = Snake::new();
        assert_eq!(snake.energy, INITIAL_ENERGY);

        // Use all initial energy
        for _ in 0..INITIAL_ENERGY {
            assert!(snake.use_energy());
        }
        assert_eq!(snake.energy, 0);

        // Cannot use energy when empty
        assert!(!snake.use_energy());

        // Manually add energy
        snake.energy = 5;
        assert!(snake.use_energy());
        assert_eq!(snake.energy, 4);
    }

    #[test]
    fn test_snake_try_grow_or_energy() {
        let mut snake = Snake::new();
        let max_length = 5;

        // Should grow when below max (energy unchanged)
        assert!(snake.try_grow_or_energy(max_length));
        assert_eq!(snake.length, 4);
        assert_eq!(snake.energy, INITIAL_ENERGY);

        // Grow to max
        snake.try_grow_or_energy(max_length);
        assert_eq!(snake.length, 5);

        // At max length, should gain energy instead
        assert!(!snake.try_grow_or_energy(max_length));
        assert_eq!(snake.length, 5); // Length unchanged
        assert_eq!(snake.energy, INITIAL_ENERGY + 1); // Energy gained

        // Can gain up to MAX_ENERGY
        for _ in 0..15 {
            snake.try_grow_or_energy(max_length);
        }
        assert_eq!(snake.energy, MAX_ENERGY);
    }
}
