# Development Guide

## Prerequisites

1. **Rust toolchain**
   ```bash
   rustup target add wasm32-unknown-unknown
   rustup component add rustfmt clippy
   ```

2. **WASM-4 CLI**
   ```bash
   npm install -g wasm4
   ```

3. **Setup git hooks** (optional but recommended)
   ```bash
   git config core.hooksPath .githooks
   ```

## Commands

### Build
```bash
cargo build --release
```

### Run locally
```bash
w4 run target/wasm32-unknown-unknown/release/cart.wasm
```

### Watch mode (live reload)
```bash
w4 watch
```

### Run tests
```bash
cargo test --target $(rustc --print host-tuple)
```

### Linting & Formatting
```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Run clippy linter
cargo clippy

# Run clippy with warnings as errors
cargo clippy -- -D warnings
```

### Create HTML bundle
```bash
w4 bundle target/wasm32-unknown-unknown/release/cart.wasm \
    --title "WASM-4 Snake" \
    --html dist/index.html
```

### Check cartridge size
```bash
ls -la target/wasm32-unknown-unknown/release/cart.wasm
# Must be < 65536 bytes (64 KB)
```

## Git Hooks

This project includes pre-commit hooks for code quality:

```bash
# Enable git hooks
git config core.hooksPath .githooks
```

The pre-commit hook will:
1. Check code formatting (`cargo fmt --check`)
2. Run clippy linter

## Debugging

Use `trace()` to print debug messages:
```rust
use crate::wasm4::trace;
trace("debug message");
```

View output in browser console or `w4 run` terminal.

## Common Issues

### Cartridge too large
- Enable LTO: `lto = true` in Cargo.toml
- Use `opt-level = "z"` for size optimization
- Avoid unnecessary dependencies
- Use fixed arrays instead of Vec when possible

### Memory corruption
- Check array bounds manually
- Verify snake length doesn't exceed MAX_SNAKE_LENGTH
- Use `unsafe` blocks only for WASM-4 API calls

### Game too fast/slow
- Adjust frame counter divisor in `game.rs`
- Current: 15 frames = ~4 moves/second

### Tests not compiling
- Run tests on native target: `cargo test --target $(rustc --print host-tuple)`
- WASM-specific code is excluded from tests via `#[cfg(target_arch = "wasm32")]`

## File Descriptions

| File | Purpose |
|------|---------|
| `src/lib.rs` | Entry point, exports `start()` and `update()` |
| `src/wasm4.rs` | WASM-4 API bindings |
| `src/alloc.rs` | Memory allocator configuration |
| `src/game.rs` | Main game loop and state |
| `src/snake.rs` | Snake movement and collision |
| `src/food.rs` | Food placement |
| `src/rng.rs` | Random number generator |
| `rustfmt.toml` | Rustfmt configuration |
| `.clippy.toml` | Clippy configuration |
| `.githooks/pre-commit` | Pre-commit hook script |
