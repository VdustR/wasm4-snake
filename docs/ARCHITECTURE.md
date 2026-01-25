# Architecture

## Overview

This Snake game is built for the WASM-4 fantasy console. See [GAME_DESIGN.md](GAME_DESIGN.md) for WASM-4 constraints and game mechanics.

## Module Structure

```
src/
├── lib.rs      # Entry point: start() and update() exports
├── wasm4.rs    # WASM-4 API bindings (drawing, input, sound)
├── alloc.rs    # buddy-alloc memory allocator (20KB heap)
├── game.rs     # Game state machine and main loop
├── snake.rs    # Snake body, movement, collision detection
├── food.rs     # Food spawning and positioning
└── rng.rs      # Linear Congruential Generator for randomness
```

## Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Input     │────▶│    Game     │────▶│   Render    │
│  (Gamepad)  │     │   Logic     │     │  (WASM-4)   │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                    ┌──────┴──────┐
                    ▼             ▼
              ┌─────────┐   ┌─────────┐
              │  Snake  │   │  Food   │
              └─────────┘   └─────────┘
```

## Key Design Decisions

### 1. Fixed-Size Array for Snake Body

**Decision**: Use `[Point; 400]` instead of `Vec<Point>`

**Rationale**:
- Avoids heap fragmentation in constrained memory
- Predictable memory usage: 400 * 8 bytes = 3.2 KB
- 400 = 20x20 grid, maximum possible snake length
- No runtime allocation failures

### 2. Frame Rate Control

**Decision**: Update game logic every 15 frames

**Rationale**:
- WASM-4 runs at 60 FPS (too fast for snake)
- 60 / 15 = 4 moves per second (comfortable pace)
- Input is read every frame for responsiveness

### 3. Screen Wrapping vs Wall Collision

**Decision**: Screen wrapping (snake appears on opposite side)

**Rationale**:
- Classic snake behavior
- Simpler implementation
- More forgiving gameplay

### 4. Simple LCG Random Number Generator

**Decision**: Implement custom RNG instead of using crate

**Rationale**:
- `no_std` environment has no standard RNG
- LCG is simple and sufficient for food placement
- Keeps cartridge size minimal

## Memory Layout

```
WASM-4 Memory Map (64 KB total):
┌────────────────────┐ 0x0000
│    Reserved        │
├────────────────────┤ 0x0004
│    PALETTE (16B)   │
├────────────────────┤ 0x0014
│   DRAW_COLORS (2B) │
├────────────────────┤ 0x0016
│   GAMEPADS (4B)    │
├────────────────────┤ 0x001A
│   MOUSE (5B)       │
├────────────────────┤ 0x00A0
│   FRAMEBUFFER      │
│   (6400 bytes)     │
├────────────────────┤ 0x19A0
│                    │
│   User Memory      │
│   (buddy-alloc)    │
│                    │
└────────────────────┘ 0xFFFF
```

## Game State Machine

```
    ┌──────────┐
    │ MainMenu │◀────────────────────────────┐
    └────┬─────┘                             │
         │                                   │
    ┌────┴────┐                              │
    ▼         ▼                              │
┌──────────┐  ┌────────────────┐             │
│ Settings │  │ DifficultySelect│             │
└────┬─────┘  └───────┬────────┘             │
     │                │                      │
     │ (back)         │ (select)             │
     ▼                ▼                      │
┌──────────┐     ┌─────────┐                 │
│ MainMenu │     │ Playing │◀────┐           │
└──────────┘     └────┬────┘     │           │
                      │     eat food         │
                      │          │           │
                      │ collision            │
                      ▼                      │
                 ┌──────────┐   menu         │
                 │ GameOver │────────────────┘
                 └──────────┘
```

## Testing Strategy

Since WASM-4 APIs are unavailable in native test environment, we separate:

**Testable (pure logic)**:
- `Point` operations
- `Direction` transformations
- `Snake` movement and collision
- `Rng` determinism and range

**Not testable (WASM-4 dependent)**:
- Rendering functions
- Sound playback
- Input reading

Tests run with `cargo test` on native target (not WASM).
