use crate::rng::Rng;
use crate::snake::{Point, GRID_SIZE};

/// Frames between spawn attempts (10 seconds at 60 FPS)
pub const SUPPLY_SPAWN_INTERVAL: u16 = 600;
/// Chance to spawn (50% = spawn when random < 50)
pub const SUPPLY_SPAWN_CHANCE: u8 = 50;
/// Frames until supply despawns if not collected (15 seconds)
pub const SUPPLY_LIFETIME: u16 = 900;

/// A supply pack that grants energy when collected
pub struct Supply {
    pub position: Point,
    pub active: bool,
    spawn_timer: u16,
    lifetime_timer: u16,
}

impl Supply {
    /// Create a new inactive supply
    pub const fn new() -> Self {
        Self {
            position: Point::new(0, 0),
            active: false,
            spawn_timer: 0,
            lifetime_timer: 0,
        }
    }

    /// Reset the supply state
    pub fn reset(&mut self) {
        self.active = false;
        self.spawn_timer = 0;
        self.lifetime_timer = 0;
    }

    /// Update spawn timer and try to spawn if conditions are met.
    /// Returns true if a new supply spawned this frame.
    ///
    /// # Arguments
    /// * `rng` - Random number generator
    /// * `snake_body` - Player snake body segments
    /// * `snake_length` - Player snake length
    /// * `food_pos` - Food position to avoid
    /// * `enemy_check` - Closure to check if position collides with enemies
    pub fn update_spawning<F>(
        &mut self,
        rng: &mut Rng,
        snake_body: &[Point],
        snake_length: usize,
        food_pos: Point,
        enemy_check: F,
    ) -> bool
    where
        F: Fn(Point) -> bool,
    {
        // Update lifetime if active
        if self.active {
            self.lifetime_timer += 1;
            if self.lifetime_timer >= SUPPLY_LIFETIME {
                self.active = false;
                self.lifetime_timer = 0;
            }
            return false;
        }

        // Update spawn timer
        self.spawn_timer += 1;
        if self.spawn_timer < SUPPLY_SPAWN_INTERVAL {
            return false;
        }
        self.spawn_timer = 0;

        // 50% chance to spawn
        if rng.range(0, 100) >= SUPPLY_SPAWN_CHANCE as i32 {
            return false;
        }

        // Find a valid spawn position
        self.spawn(rng, snake_body, snake_length, food_pos, enemy_check)
    }

    /// Try to spawn at a random valid position
    fn spawn<F>(
        &mut self,
        rng: &mut Rng,
        snake_body: &[Point],
        snake_length: usize,
        food_pos: Point,
        enemy_check: F,
    ) -> bool
    where
        F: Fn(Point) -> bool,
    {
        // Try up to 50 times to find a valid position
        for _ in 0..50 {
            let x = rng.range(0, GRID_SIZE);
            let y = rng.range(0, GRID_SIZE);
            let pos = Point::new(x, y);

            // Check not on player snake
            if snake_body[..snake_length].contains(&pos) {
                continue;
            }

            // Check not on food
            if pos == food_pos {
                continue;
            }

            // Check not on enemies
            if enemy_check(pos) {
                continue;
            }

            // Valid position found
            self.position = pos;
            self.active = true;
            self.lifetime_timer = 0;
            return true;
        }

        false
    }

    /// Check if the supply is at the given position
    pub fn is_at(&self, pos: Point) -> bool {
        self.active && self.position == pos
    }

    /// Collect the supply (deactivate it)
    pub fn collect(&mut self) {
        self.active = false;
        self.lifetime_timer = 0;
    }

    /// Get the flash color index for rendering (alternates for blinking effect)
    /// Returns 0 for yellow phase, 1 for green phase
    pub fn flash_phase(&self, frame_count: u32) -> u8 {
        // Blink every 8 frames
        if (frame_count / 8).is_multiple_of(2) {
            0 // Yellow phase
        } else {
            1 // Green phase
        }
    }
}

impl Default for Supply {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supply_new() {
        let supply = Supply::new();
        assert!(!supply.active);
        assert_eq!(supply.spawn_timer, 0);
    }

    #[test]
    fn test_supply_reset() {
        let mut supply = Supply::new();
        supply.active = true;
        supply.spawn_timer = 100;
        supply.lifetime_timer = 50;

        supply.reset();

        assert!(!supply.active);
        assert_eq!(supply.spawn_timer, 0);
        assert_eq!(supply.lifetime_timer, 0);
    }

    #[test]
    fn test_supply_is_at() {
        let mut supply = Supply::new();
        supply.position = Point::new(5, 5);

        // Not active, should return false
        assert!(!supply.is_at(Point::new(5, 5)));

        // Active, should return true at position
        supply.active = true;
        assert!(supply.is_at(Point::new(5, 5)));
        assert!(!supply.is_at(Point::new(5, 6)));
    }

    #[test]
    fn test_supply_collect() {
        let mut supply = Supply::new();
        supply.active = true;
        supply.position = Point::new(5, 5);
        supply.lifetime_timer = 100;

        supply.collect();

        assert!(!supply.active);
        assert_eq!(supply.lifetime_timer, 0);
    }

    #[test]
    fn test_supply_flash_phase() {
        let supply = Supply::new();

        // Test alternating phases
        assert_eq!(supply.flash_phase(0), 0); // Yellow
        assert_eq!(supply.flash_phase(8), 1); // Green
        assert_eq!(supply.flash_phase(16), 0); // Yellow
        assert_eq!(supply.flash_phase(24), 1); // Green
    }
}
