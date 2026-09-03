use crate::snake::{Direction, Point, GRID_SIZE};

/// BFS queue size (must be power of 2 for efficiency, kept small for stack safety)
const QUEUE_SIZE: usize = 32;

/// A node in the BFS search
#[derive(Clone, Copy)]
struct BfsNode {
    pos: Point,
    first_dir: Direction, // The first direction taken from start
    depth: u8,
}

/// Pathfinder using BFS
pub struct PathFinder {
    visited: [[bool; GRID_SIZE as usize]; GRID_SIZE as usize],
    queue: [BfsNode; QUEUE_SIZE],
    queue_head: usize,
    queue_tail: usize,
}

impl PathFinder {
    /// Create a new pathfinder
    pub const fn new() -> Self {
        Self {
            visited: [[false; GRID_SIZE as usize]; GRID_SIZE as usize],
            queue: [BfsNode {
                pos: Point::new(0, 0),
                first_dir: Direction::Right,
                depth: 0,
            }; QUEUE_SIZE],
            queue_head: 0,
            queue_tail: 0,
        }
    }

    /// Reset the pathfinder state
    fn reset(&mut self) {
        for row in &mut self.visited {
            row.fill(false);
        }
        self.queue_head = 0;
        self.queue_tail = 0;
    }

    /// Check if a position is valid (not an obstacle)
    fn is_valid(&self, pos: Point, obstacles: &[Point]) -> bool {
        if pos.x < 0 || pos.x >= GRID_SIZE || pos.y < 0 || pos.y >= GRID_SIZE {
            return false;
        }
        if self.visited[pos.y as usize][pos.x as usize] {
            return false;
        }
        !obstacles.contains(&pos)
    }

    /// Find the best direction to reach a target, avoiding obstacles
    /// Returns None if no path found within max_depth
    pub fn find_direction(
        &mut self,
        from: Point,
        to: Point,
        obstacles: &[Point],
        max_depth: u8,
    ) -> Option<Direction> {
        self.reset();

        // Handle same position
        if from == to {
            return None;
        }

        // Mark start as visited (with bounds check)
        if from.x >= 0 && from.x < GRID_SIZE && from.y >= 0 && from.y < GRID_SIZE {
            self.visited[from.y as usize][from.x as usize] = true;
        }

        // Enqueue all valid neighbors from start
        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];

        for dir in directions {
            let delta = dir.delta();
            let next = Point::new(
                (from.x + delta.x).rem_euclid(GRID_SIZE),
                (from.y + delta.y).rem_euclid(GRID_SIZE),
            );

            if self.is_valid(next, obstacles) {
                self.visited[next.y as usize][next.x as usize] = true;
                self.queue[self.queue_tail] = BfsNode {
                    pos: next,
                    first_dir: dir,
                    depth: 1,
                };
                self.queue_tail = (self.queue_tail + 1) % QUEUE_SIZE;

                // Check if we already found target
                if next == to {
                    return Some(dir);
                }
            }
        }

        // BFS
        while self.queue_head != self.queue_tail {
            let node = self.queue[self.queue_head];
            self.queue_head = (self.queue_head + 1) % QUEUE_SIZE;

            if node.depth >= max_depth {
                continue;
            }

            for dir in directions {
                let delta = dir.delta();
                let next = Point::new(
                    (node.pos.x + delta.x).rem_euclid(GRID_SIZE),
                    (node.pos.y + delta.y).rem_euclid(GRID_SIZE),
                );

                if self.is_valid(next, obstacles) {
                    self.visited[next.y as usize][next.x as usize] = true;

                    // Found target
                    if next == to {
                        return Some(node.first_dir);
                    }

                    // Enqueue if we have space
                    if (self.queue_tail + 1) % QUEUE_SIZE != self.queue_head {
                        self.queue[self.queue_tail] = BfsNode {
                            pos: next,
                            first_dir: node.first_dir,
                            depth: node.depth + 1,
                        };
                        self.queue_tail = (self.queue_tail + 1) % QUEUE_SIZE;
                    }
                }
            }
        }

        // No path found, try to move towards target anyway
        self.simple_direction(from, to, obstacles)
    }

    /// Simple direction towards target (fallback when BFS fails)
    fn simple_direction(&self, from: Point, to: Point, obstacles: &[Point]) -> Option<Direction> {
        let dx = to.x - from.x;
        let dy = to.y - from.y;

        // Prioritize the axis with larger distance
        let directions = if dx.abs() > dy.abs() {
            if dx > 0 {
                [
                    Direction::Right,
                    Direction::Down,
                    Direction::Up,
                    Direction::Left,
                ]
            } else {
                [
                    Direction::Left,
                    Direction::Down,
                    Direction::Up,
                    Direction::Right,
                ]
            }
        } else if dy > 0 {
            [
                Direction::Down,
                Direction::Right,
                Direction::Left,
                Direction::Up,
            ]
        } else {
            [
                Direction::Up,
                Direction::Right,
                Direction::Left,
                Direction::Down,
            ]
        };

        for dir in directions {
            let delta = dir.delta();
            let next = Point::new(
                (from.x + delta.x).rem_euclid(GRID_SIZE),
                (from.y + delta.y).rem_euclid(GRID_SIZE),
            );
            if !obstacles.contains(&next) {
                return Some(dir);
            }
        }

        None
    }

    /// Find any safe direction (avoid obstacles)
    pub fn find_safe_direction(
        &self,
        from: Point,
        obstacles: &[Point],
        current_dir: Direction,
    ) -> Option<Direction> {
        // First try continuing in current direction
        let delta = current_dir.delta();
        let next = Point::new(
            (from.x + delta.x).rem_euclid(GRID_SIZE),
            (from.y + delta.y).rem_euclid(GRID_SIZE),
        );
        if !obstacles.contains(&next) {
            return Some(current_dir);
        }

        // Try other directions
        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];

        for dir in directions {
            if dir == current_dir.opposite() {
                continue;
            }
            let d = dir.delta();
            let n = Point::new(
                (from.x + d.x).rem_euclid(GRID_SIZE),
                (from.y + d.y).rem_euclid(GRID_SIZE),
            );
            if !obstacles.contains(&n) {
                return Some(dir);
            }
        }

        None
    }
}

impl Default for PathFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pathfinder_new() {
        let pf = PathFinder::new();
        assert_eq!(pf.queue_head, 0);
        assert_eq!(pf.queue_tail, 0);
    }

    #[test]
    fn test_pathfinder_direct_path() {
        let mut pf = PathFinder::new();
        let from = Point::new(5, 5);
        let to = Point::new(7, 5);
        let obstacles = [];

        let dir = pf.find_direction(from, to, &obstacles, 10);
        assert_eq!(dir, Some(Direction::Right));
    }

    #[test]
    fn test_pathfinder_with_obstacle() {
        let mut pf = PathFinder::new();
        let from = Point::new(5, 5);
        let to = Point::new(7, 5);
        let obstacles = [Point::new(6, 5)]; // Block direct path

        let dir = pf.find_direction(from, to, &obstacles, 10);
        // Should go around (up or down)
        assert!(dir == Some(Direction::Up) || dir == Some(Direction::Down));
    }

    #[test]
    fn test_safe_direction() {
        let pf = PathFinder::new();
        let from = Point::new(5, 5);
        let obstacles = [Point::new(6, 5)]; // Block right

        let dir = pf.find_safe_direction(from, &obstacles, Direction::Right);
        // Should find an alternative
        assert!(dir.is_some());
        assert_ne!(dir, Some(Direction::Right));
    }
}
