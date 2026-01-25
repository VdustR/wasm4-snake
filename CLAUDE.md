# WASM-4 Snake Game

A snake game built for the [WASM-4](https://wasm4.org/) fantasy console using Rust.

## Development

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- w4 CLI (`npm install -g wasm4`)

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
└── alloc.rs    # Memory allocator (buddy-alloc)
```

## Code Conventions

- Use `unsafe` blocks only for WASM-4 API calls
- Keep the cart size under 64 KB
- Comment game logic in English
