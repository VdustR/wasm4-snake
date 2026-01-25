# WASM-4 Snake Game

A snake game built for the [WASM-4](https://wasm4.org/) fantasy console using Rust.

## Quick Reference

```bash
# Build
cargo build --release

# Run
w4 run target/wasm32-unknown-unknown/release/cart.wasm

# Watch mode
w4 watch

# Test (native target)
cargo test --target $(rustc --print host-tuple)
```

## Project Structure

```
src/
├── lib.rs      # Entry point (update function)
├── wasm4.rs    # WASM-4 API bindings
├── alloc.rs    # Memory allocator (buddy-alloc)
├── game.rs     # Game state machine, main loop, Difficulty enum
├── snake.rs    # Snake logic (player & base)
├── food.rs     # Food system (3 sizes)
├── supply.rs   # Supply pack system
├── rng.rs      # Random number generator
├── enemy.rs    # Enemy snake, EnemyAIState enum
├── ai.rs       # AI pathfinding (BFS)
└── menu.rs     # Menu rendering
```

## Game Design Reference

**See [docs/GAME_DESIGN.md](docs/GAME_DESIGN.md) for complete game mechanics**, including:
- Game modes (Classic vs Battle)
- Difficulty parameters
- Combat system (shrink/grow)
- AI behavior and parameters

## Code Conventions

- Use `unsafe` blocks only for WASM-4 API calls
- Keep the cart size under 64 KB
- Comment game logic in English
- All game constants in `game.rs` (durations, cooldowns, etc.)

## Testing

- **Always use `w4 run` directly** to test game functionality
- Web testing is only for landing page and PWA functionality
- Run `cargo test` for unit tests (uses native target)

## Documentation Maintenance

When game mechanics or features change:

1. **Update `docs/GAME_DESIGN.md`** - Single source of truth for game rules
2. Update `README.md` only if the change affects the public overview
3. Update `docs/ARCHITECTURE.md` for technical design changes
4. Update this file (`CLAUDE.md`) only for development workflow changes

Documentation hierarchy:
| File | Purpose | Update when... |
|------|---------|----------------|
| `docs/GAME_DESIGN.md` | Complete game mechanics | Game rules change |
| `docs/CHEAT.md` | Cheat mode features | Cheat features change |
| `README.md` | Public overview | Features/setup change |
| `docs/ARCHITECTURE.md` | Technical design | Architecture decisions |
| `docs/DEVELOPMENT.md` | Development commands | Tools/workflow change |
| `CLAUDE.md` | AI development context | Code structure changes |

Note: Cheat mode exists but is intentionally not documented in public-facing files (README, GAME_DESIGN).
