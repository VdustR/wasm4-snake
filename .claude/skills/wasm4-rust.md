---
description: Best practices for WASM-4 game development with Rust. Use when writing or reviewing WASM-4 Rust code.
---

# WASM-4 Rust Best Practices

## Memory Management

### DO: Use fixed-size arrays
```rust
const MAX_ENTITIES: usize = 100;
struct Game {
    entities: [Entity; MAX_ENTITIES],
    entity_count: usize,
}
```

### DON'T: Use Vec in hot paths
```rust
// Avoid - causes heap fragmentation
struct Game {
    entities: Vec<Entity>,
}
```

### DO: Keep allocations minimal
- Total heap: 20 KB (4 KB fast + 16 KB main)
- Cartridge limit: 64 KB
- Consider `--no-default-features` to disable allocator entirely

### ⚠️ CRITICAL: Stack Size Limitations
WASM-4 has a very limited stack (~8-14KB). Large arrays in structs cause stack overflow during initialization because Rust creates temporary copies on the stack.

**Problem pattern:**
```rust
// BAD - creates ~3200 bytes on stack during initialization
const MAX_LENGTH: usize = 400;
pub fn new() -> Self {
    let body = [Point::default(); MAX_LENGTH]; // Stack overflow!
    Self { body, ... }
}
```

**Solutions:**
1. Keep array sizes small (< 100 elements per array)
2. Use `const {}` blocks for array initialization when possible
3. Split large structs into smaller parts
4. Recommended limits:
   - Player snake body: 50 elements max
   - Enemy snake body: 30 elements max
   - BFS queue: 32 elements max
   - Obstacle cache: 128 elements max

**Debug tip:** Look for "memory access out of bounds" at `_ZN...new...` in error traces - this indicates stack overflow during struct initialization.

## Cartridge Size Optimization

### Cargo.toml settings
```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller panic handling
strip = true         # Strip symbols
```

### Avoid
- Large dependencies
- Format strings (`format!` pulls in formatting machinery)
- Floating point operations (software emulation is large)

## Frame Rate Control

WASM-4 runs at 60 FPS. For games needing slower updates:

```rust
static mut FRAME_COUNT: u32 = 0;

fn update() {
    unsafe { FRAME_COUNT += 1; }

    // Update game logic every N frames
    if unsafe { FRAME_COUNT } % 15 == 0 {
        update_game_logic();
    }

    // Always render
    render();
}
```

## Input Handling

### DO: Detect button press (not hold)
```rust
static mut PREV_GAMEPAD: u8 = 0;

fn update() {
    let gamepad = unsafe { *GAMEPAD1 };
    let just_pressed = gamepad & (gamepad ^ unsafe { PREV_GAMEPAD });

    if just_pressed & BUTTON_UP != 0 {
        // Handle single press
    }

    unsafe { PREV_GAMEPAD = gamepad; }
}
```

## Drawing

### Color palette
```rust
unsafe {
    (*PALETTE)[0] = 0x1a1c2c; // Background
    (*PALETTE)[1] = 0x5d275d; // Color 1
    (*PALETTE)[2] = 0xb13e53; // Color 2
    (*PALETTE)[3] = 0xf6c64f; // Color 3
}
```

### DRAW_COLORS format
```rust
// 0xABCD where:
// A = stroke color (0 = transparent)
// B = fill color
// C, D = for sprites

unsafe { *DRAW_COLORS = 0x43; } // Stroke=4, Fill=3
```

## Random Numbers

No built-in RNG. Use Linear Congruential Generator:

```rust
struct Rng(u32);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0
    }

    fn range(&mut self, min: i32, max: i32) -> i32 {
        min + (self.next() % (max - min) as u32) as i32
    }
}
```

## Sound

```rust
// Eat sound - short high tone
tone(440 | (880 << 16), 5, 80, TONE_PULSE1);

// Game over - descending tone
tone(440 | (110 << 16), 60, 80, TONE_TRIANGLE);
```

## Testing

Separate pure logic from WASM-4 APIs:

```rust
// In snake.rs - testable
impl Snake {
    pub fn would_collide(&self, point: Point) -> bool {
        self.body[..self.length].contains(&point)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_collision() {
        let snake = Snake::new();
        assert!(!snake.would_collide(Point::new(0, 0)));
    }
}
```

Run tests on native target:
```bash
cargo test
```

## Common Patterns

### Game state enum
```rust
enum GameState { Menu, Playing, Paused, GameOver }
```

### Global game instance
```rust
static mut GAME: Option<Game> = None;

#[no_mangle]
fn start() {
    unsafe { GAME = Some(Game::new()); }
}

#[no_mangle]
fn update() {
    unsafe {
        if let Some(game) = GAME.as_mut() {
            game.update();
        }
    }
}
```

## Resources

- [WASM-4 Docs](https://wasm4.org/docs)
- [WASM-4 Snake Tutorial](https://wasm4.org/docs/tutorials/snake/goal)
- [Rust WASM Size Optimization](https://rustwasm.github.io/book/game-of-life/code-size.html)
