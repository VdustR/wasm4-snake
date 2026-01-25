use crate::rng::Rng;
use crate::snake::{Direction, Point, GRID_SIZE, MAX_ENERGY, MIN_SNAKE_LENGTH};

/// Maximum number of enemies
pub const MAX_ENEMIES: usize = 8;
/// Maximum length for each enemy snake (kept small for memory constraints)
pub const MAX_ENEMY_LENGTH: usize = 30;

/// Enemy AI states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyAIState {
    Idle,           // Random movement
    Chasing,        // Chasing player
    Seeking,        // Seeking food
    GrabbingSupply, // Going for supply pack
    Fleeing,        // Running away from player
}

/// An enemy snake
pub struct EnemySnake {
    pub body: [Point; MAX_ENEMY_LENGTH],
    pub length: usize,
    pub direction: Direction,
    pub alive: bool,
    pub ai_state: EnemyAIState,
    pub color_index: u8,    // 1-3 for different colors
    pub move_timer: u8,     // Frames until next move
    pub decision_timer: u8, // Frames until next AI decision
    pub energy: u8,         // Energy for speed boost
    pub boost_timer: u16,   // Remaining boost frames
    pub slow_timer: u16,    // Remaining slow frames
    pub just_moved: bool,   // True if this enemy moved this frame
}

impl EnemySnake {
    /// Create a new enemy snake at a given position
    pub fn new(x: i32, y: i32, color_index: u8) -> Self {
        let mut body = [Point::default(); MAX_ENEMY_LENGTH];
        body[0] = Point::new(x, y);
        body[1] = Point::new(x.saturating_sub(1).max(0), y);
        body[2] = Point::new(x.saturating_sub(2).max(0), y);

        Self {
            body,
            length: MIN_SNAKE_LENGTH,
            direction: Direction::Right,
            alive: true,
            ai_state: EnemyAIState::Idle,
            color_index,
            move_timer: 0,
            decision_timer: 0,
            energy: 0,
            boost_timer: 0,
            slow_timer: 0,
            just_moved: false,
        }
    }

    /// Create a dead/inactive enemy (placeholder)
    pub const fn dead() -> Self {
        Self {
            body: [Point::new(0, 0); MAX_ENEMY_LENGTH],
            length: 0,
            direction: Direction::Right,
            alive: false,
            ai_state: EnemyAIState::Idle,
            color_index: 0,
            move_timer: 0,
            decision_timer: 0,
            energy: 0,
            boost_timer: 0,
            slow_timer: 0,
            just_moved: false,
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

    /// Move the snake in the current direction
    pub fn update(&mut self) {
        if !self.alive || self.length == 0 {
            return;
        }

        // Move each segment to the position of the segment in front
        for i in (1..self.length).rev() {
            self.body[i] = self.body[i - 1];
        }

        // Move head with wrapping
        let delta = self.direction.delta();
        self.body[0] = Point::new(
            (self.body[0].x + delta.x).rem_euclid(GRID_SIZE),
            (self.body[0].y + delta.y).rem_euclid(GRID_SIZE),
        );
    }

    /// Grow the snake by a certain amount
    pub fn grow(&mut self, amount: usize) {
        for _ in 0..amount {
            if self.length > 0 && self.length < MAX_ENEMY_LENGTH {
                self.body[self.length] = self.body[self.length - 1];
                self.length += 1;
            }
        }
    }

    /// Try to change direction (ignores 180-degree turns)
    pub fn set_direction(&mut self, new_dir: Direction) {
        if new_dir != self.direction.opposite() {
            self.direction = new_dir;
        }
    }

    /// Check if a point is occupied by this snake
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
    pub fn try_grow_or_energy(&mut self, max_length: usize) -> bool {
        if self.length < max_length && self.length < MAX_ENEMY_LENGTH {
            self.grow(1);
            true
        } else {
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

/// Manager for all enemy snakes
pub struct EnemyManager {
    pub enemies: [EnemySnake; MAX_ENEMIES],
    pub active_count: usize,
    pub spawn_timer: u16,
    color_cycle: u8,
}

impl EnemyManager {
    /// Create a new enemy manager
    pub const fn new() -> Self {
        Self {
            enemies: [const { EnemySnake::dead() }; MAX_ENEMIES],
            active_count: 0,
            spawn_timer: 0,
            color_cycle: 1,
        }
    }

    /// Reset all enemies
    pub fn reset(&mut self) {
        for enemy in &mut self.enemies {
            *enemy = EnemySnake::dead();
        }
        self.active_count = 0;
        self.spawn_timer = 0;
        self.color_cycle = 1;
    }

    /// Spawn a new enemy at a random position
    pub fn spawn(&mut self, rng: &mut Rng, player_body: &[Point], player_length: usize) -> bool {
        // Find an empty slot
        let slot = self.enemies.iter().position(|e| !e.alive);
        if slot.is_none() {
            return false;
        }
        let slot = slot.unwrap();

        // Find a spawn position not occupied by player or other enemies
        for _ in 0..100 {
            let x = rng.range(0, GRID_SIZE);
            let y = rng.range(0, GRID_SIZE);

            // Check not on player
            let on_player = player_body[..player_length].contains(&Point::new(x, y))
                || player_body[..player_length].contains(&Point::new(x - 1, y))
                || player_body[..player_length].contains(&Point::new(x - 2, y));

            // Check not on other enemies
            let on_enemy = self.enemies.iter().any(|e| {
                e.alive
                    && (e.contains(Point::new(x, y))
                        || e.contains(Point::new(x - 1, y))
                        || e.contains(Point::new(x - 2, y)))
            });

            if !on_player && !on_enemy && x >= 2 {
                self.enemies[slot] = EnemySnake::new(x, y, self.color_cycle);
                self.color_cycle = (self.color_cycle % 3) + 1;
                self.active_count += 1;
                return true;
            }
        }

        false
    }

    /// Update spawn timer and spawn if needed
    pub fn update_spawning(
        &mut self,
        rng: &mut Rng,
        spawn_interval: u16,
        max_enemies: usize,
        player_body: &[Point],
        player_length: usize,
    ) {
        if spawn_interval == 0 || self.active_count >= max_enemies {
            return;
        }

        self.spawn_timer += 1;
        if self.spawn_timer >= spawn_interval {
            self.spawn_timer = 0;
            self.spawn(rng, player_body, player_length);
        }
    }

    /// Kill an enemy at index
    pub fn kill(&mut self, index: usize) {
        if index < MAX_ENEMIES && self.enemies[index].alive {
            self.enemies[index].alive = false;
            self.enemies[index].length = 0;
            self.active_count = self.active_count.saturating_sub(1);
        }
    }
}

impl Default for EnemyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_snake_new() {
        let enemy = EnemySnake::new(10, 10, 1);
        assert!(enemy.alive);
        assert_eq!(enemy.length, 3);
        assert_eq!(enemy.head(), Point::new(10, 10));
    }

    #[test]
    fn test_enemy_snake_dead() {
        let enemy = EnemySnake::dead();
        assert!(!enemy.alive);
        assert_eq!(enemy.length, 0);
    }

    #[test]
    fn test_enemy_manager_new() {
        let manager = EnemyManager::new();
        assert_eq!(manager.active_count, 0);
        assert!(manager.enemies.iter().all(|e| !e.alive));
    }

    #[test]
    fn test_enemy_movement() {
        let mut enemy = EnemySnake::new(10, 10, 1);
        enemy.direction = Direction::Right;
        enemy.update();
        assert_eq!(enemy.head(), Point::new(11, 10));
    }

    #[test]
    fn test_enemy_grow() {
        let mut enemy = EnemySnake::new(10, 10, 1);
        let initial_length = enemy.length;
        enemy.grow(2);
        assert_eq!(enemy.length, initial_length + 2);
    }

    #[test]
    fn test_enemy_shrink() {
        let mut enemy = EnemySnake::new(10, 10, 1);
        enemy.grow(2);
        assert_eq!(enemy.length, 5);

        // Shrink should succeed
        assert!(enemy.shrink());
        assert_eq!(enemy.length, 4);

        // Shrink to min
        enemy.shrink();
        assert_eq!(enemy.length, 3);

        // Cannot shrink below min
        assert!(!enemy.shrink());
        assert_eq!(enemy.length, MIN_SNAKE_LENGTH);
    }

    #[test]
    fn test_enemy_energy() {
        let mut enemy = EnemySnake::new(10, 10, 1);
        assert_eq!(enemy.energy, 0);

        // Cannot use energy when empty
        assert!(!enemy.use_energy());

        // Add and use energy
        enemy.energy = 3;
        assert!(enemy.use_energy());
        assert_eq!(enemy.energy, 2);
    }

    #[test]
    fn test_enemy_try_grow_or_energy() {
        let mut enemy = EnemySnake::new(10, 10, 1);
        let max_length = 5;

        // Should grow when below max
        assert!(enemy.try_grow_or_energy(max_length));
        assert_eq!(enemy.length, 4);

        // Grow to max
        enemy.try_grow_or_energy(max_length);
        assert_eq!(enemy.length, 5);

        // At max, should gain energy
        assert!(!enemy.try_grow_or_energy(max_length));
        assert_eq!(enemy.energy, 1);
    }

    #[test]
    fn test_enemy_contains() {
        let enemy = EnemySnake::new(10, 10, 1);
        assert!(enemy.contains(Point::new(10, 10))); // Head
        assert!(enemy.contains(Point::new(9, 10))); // Body
        assert!(!enemy.contains(Point::new(0, 0))); // Empty
    }
}
