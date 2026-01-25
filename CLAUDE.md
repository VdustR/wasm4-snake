# WASM-4 Snake Game

A snake game built for the [WASM-4](https://wasm4.org/) fantasy console using Rust.

## Development

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- w4 CLI (`npm install -g wasm4`)
- [lefthook](https://github.com/evilmartians/lefthook) for git hooks

### Setup Git Hooks

```bash
lefthook install
```

### Build

```bash
cargo build --release
```

### Run

```bash
w4 run target/wasm32-unknown-unknown/release/cart.wasm
```

### Watch Mode (Live Reload)

```bash
w4 watch
```

## WASM-4 Constraints

- **Display**: 160x160 pixels, 4 colors
- **Memory**: 64 KB RAM
- **Cartridge**: 64 KB max
- **Frame Rate**: 60 FPS
- **Input**: Gamepad (D-pad + 2 buttons)

## Project Structure

```
src/
├── lib.rs      # Main game entry point (update function)
├── wasm4.rs    # WASM-4 API bindings
├── alloc.rs    # Memory allocator (buddy-alloc)
├── game.rs     # Game state machine & main loop
├── snake.rs    # Snake logic (player & base)
├── food.rs     # Food system (3 sizes)
├── supply.rs   # Supply pack system (energy pickups)
├── rng.rs      # Random number generator
├── enemy.rs    # Enemy snake system
├── ai.rs       # AI pathfinding (BFS)
└── menu.rs     # Menu rendering
```

## Game Rules

### Controls
- **D-pad**: Move snake (Up/Down/Left/Right)
- **X button**: Activate speed boost (costs 1 energy, non-Classic only)
- **Z button**: Activate slowdown (free, cancels boost, non-Classic only)

### Energy System
- Maximum energy: 5 units
- Initial energy: 3 units
- Gain energy by:
  - Collecting supply packs (+1)
  - Eating food when at max length (+1)

### Speed Boost (X button)
- Duration: 2 seconds
- Cooldown: 5 seconds
- Cost: 1 energy
- Visual: Wave flash effect across body
- Audio: Music plays at 2x speed and pitch

### Slowdown (Z button)
- Duration: 2 seconds
- Cooldown: 5 seconds
- Cost: Free
- Visual: Head-only flash effect
- Special: Immediately cancels active boost

### Supply Packs
- Spawn every 10 seconds (50% chance)
- Despawn after 15 seconds if not collected
- Visual: Blinking yellow/green diamond
- Effect: +1 energy
- Note: Not available in Classic mode

### Classic Mode
- Collision = instant death (traditional snake)
- No length limit
- No boost/slowdown abilities
- No supply packs
- No enemies

### Battle Modes (Noob, Normal, Hell, Nightmare)
- Collision = shrink (death only at min length 3)
- Length limit varies by difficulty
- Energy system enabled
- Supply packs enabled
- Body attack: head hits body = attacker shrinks, victim grows

### AI Behavior (Battle Modes)
- States: Idle, Chasing, Seeking, GrabbingSupply, Fleeing
- Higher difficulty = more aggressive AI
- AI can use boost when chasing or fleeing
- AI can use slowdown for precise control near targets

## Code Conventions

- Use `unsafe` blocks only for WASM-4 API calls
- Keep the cart size under 64 KB
- Comment game logic in English

## Testing

- **Always use `w4 run` directly** to test game functionality (not web e2e)
- Web testing is only for landing page and PWA functionality
