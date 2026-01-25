# Development Workflow

Complete development workflow for the WASM-4 Snake project, covering both game development (Rust) and web development (Astro).

## Table of Contents

- [Environment Setup](#environment-setup)
- [Daily Development Flow](#daily-development-flow)
- [Game Development (Rust/WASM-4)](#game-development-rustwasm-4)
- [Web Development (Astro)](#web-development-astro)
- [Testing Strategy](#testing-strategy)
- [Pre-Commit Checks](#pre-commit-checks)
- [Deployment](#deployment)
- [Troubleshooting](#troubleshooting)

---

## Environment Setup

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable | Game development |
| Node.js | 22+ | Web development, WASM-4 CLI |
| pnpm | 9+ | Package management |
| w4 CLI | latest | WASM-4 development |

### Initial Setup

```bash
# 1. Clone repository
git clone https://github.com/YOUR_USERNAME/wasm4-snake.git
cd wasm4-snake

# 2. Setup Rust toolchain
rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy

# 3. Setup git hooks
git config core.hooksPath .githooks

# 4. Setup Node.js (using nvm)
cd web
nvm use
corepack enable
pnpm install
cd ..

# 5. Install WASM-4 CLI globally
npm install -g wasm4

# 6. Verify setup
cargo build --release
cargo test --target $(rustc --print host-tuple)
```

### VSCode Configuration

The project is configured to use **local versions** of all tools to avoid version mismatches:

| Tool | Local Path | VSCode Setting |
|------|------------|----------------|
| TypeScript | `web/node_modules/typescript` | `typescript.tsdk` |
| ESLint | `web/node_modules/eslint` | `eslint.nodePath` |
| Prettier | `web/node_modules/prettier` | `prettier.prettierPath` |

This ensures:
- Consistent behavior across all team members
- No breakage from global tool updates
- Exact version control via `package.json`

---

## Daily Development Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      Development Cycle                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Pull latest       git pull origin main                      │
│         │                                                       │
│         ▼                                                       │
│  2. Create branch     git checkout -b feature/my-feature        │
│         │                                                       │
│         ▼                                                       │
│  3. Develop           [Edit code]                               │
│         │                                                       │
│         ▼                                                       │
│  4. Test locally      cargo test && w4 run ...                  │
│         │                                                       │
│         ▼                                                       │
│  5. Commit            git add . && git commit                   │
│         │             (pre-commit hooks run automatically)      │
│         ▼                                                       │
│  6. Push & PR         git push && gh pr create                  │
│         │                                                       │
│         ▼                                                       │
│  7. CI passes         GitHub Actions runs tests                 │
│         │                                                       │
│         ▼                                                       │
│  8. Merge             Squash and merge to main                  │
│         │                                                       │
│         ▼                                                       │
│  9. Auto-deploy       GitHub Pages updated automatically        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Game Development (Rust/WASM-4)

### Quick Commands

```bash
# Build game
cargo build --release

# Run game locally (opens browser at localhost:4444)
w4 run target/wasm32-unknown-unknown/release/cart.wasm

# Watch mode (auto-rebuild on save)
w4 watch

# Run tests
cargo test --target $(rustc --print host-tuple)

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Check cartridge size (must be < 64KB)
ls -la target/wasm32-unknown-unknown/release/cart.wasm
```

### File Structure

```
src/
├── lib.rs      # Entry point (start, update)
├── wasm4.rs    # WASM-4 API bindings
├── alloc.rs    # Memory allocator
├── game.rs     # Game state & main loop
├── snake.rs    # Snake logic
├── food.rs     # Food logic
└── rng.rs      # Random number generator
```

### WASM-4 Constraints

- **Display**: 160×160 pixels, 4 colors
- **Memory**: 64 KB RAM
- **Cartridge**: 64 KB max size
- **Frame Rate**: 60 FPS
- **Audio**: 4 channels (2× pulse, triangle, noise)

---

## Web Development (Astro)

### Quick Commands

```bash
cd web

# Install dependencies
pnpm install

# Start development server (http://localhost:4321)
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview
```

### File Structure

```
web/
├── public/
│   ├── game/           # WASM-4 bundled HTML (generated)
│   ├── favicon.svg
│   ├── icon-*.svg      # PWA icons
│   ├── manifest.json   # PWA manifest
│   └── sw.js           # Service worker
├── src/
│   ├── components/     # Astro components
│   ├── layouts/        # Page layouts
│   ├── pages/          # Routes (file-based)
│   ├── styles/         # Global CSS
│   └── env.d.ts        # TypeScript types
├── astro.config.mjs
├── package.json
└── tsconfig.json
```

### Local Development with Game

To test the full site with the game:

```bash
# 1. Build the game
cargo build --release

# 2. Bundle into web/public/game/
mkdir -p web/public/game
w4 bundle target/wasm32-unknown-unknown/release/cart.wasm \
    --title "WASM-4 Snake" \
    --html web/public/game/index.html

# 3. Start Astro dev server
cd web && pnpm dev
```

---

## Testing Strategy

### Rust Tests

Tests run on native target (not WASM):

```bash
# Run all tests
cargo test --target $(rustc --print host-tuple)

# Run specific test
cargo test test_snake_movement

# Run with output
cargo test -- --nocapture
```

### Test Coverage

| Module | Tested |
|--------|--------|
| `snake.rs` | Point, Direction, Snake movement, collision |
| `rng.rs` | Determinism, seeding, range bounds |
| `food.rs` | Spawn position, not on snake |
| `game.rs` | Game state enum |

### What's NOT Testable

- WASM-4 rendering (uses unsafe FFI)
- Audio (WASM-4 API)
- Input handling (requires WASM-4 runtime)

---

## Pre-Commit Checks

The `.githooks/pre-commit` script runs automatically on `git commit`:

### What's Checked

| Language | Tool | Action |
|----------|------|--------|
| Rust | `cargo fmt` | Check formatting |
| Rust | `clippy` | Linting (warnings as errors) |
| Web | `lint-staged` | ESLint + Prettier on staged files |

### lint-staged Configuration

Web files are checked with lint-staged (only staged files):

```json
// web/package.json
{
  "lint-staged": {
    "*.{js,ts}": ["eslint --fix", "prettier --write"],
    "*.astro": ["eslint --fix", "prettier --write"],
    "*.{json,md,css}": ["prettier --write"]
  }
}
```

### Enable Git Hooks

```bash
git config core.hooksPath .githooks
```

### Bypass (Emergency Only)

```bash
git commit --no-verify -m "message"
```

### Run Manually

```bash
# Rust checks
cargo fmt --check
cargo clippy -- -D warnings

# Web checks (in web/ directory)
cd web && pnpm exec lint-staged
```

---

## Deployment

### Automatic (GitHub Actions)

Push to `main` branch triggers:

1. Rust format check (`cargo fmt --check`)
2. Clippy linting
3. WASM build
4. Unit tests
5. Bundle WASM-4 → HTML
6. Build Astro site
7. Deploy to GitHub Pages

### Manual Deployment

```bash
# 1. Build everything
cargo build --release
cargo test

# 2. Bundle game
mkdir -p web/public/game
w4 bundle target/wasm32-unknown-unknown/release/cart.wasm \
    --title "WASM-4 Snake" \
    --html web/public/game/index.html

# 3. Build web
cd web
pnpm install
pnpm build

# 4. Output is in web/dist/
```

### GitHub Pages Setup

1. Go to repository **Settings** → **Pages**
2. Set **Source** to "GitHub Actions"
3. Push to `main` to trigger deployment

---

## Troubleshooting

### Cargo Build Fails

```bash
# Ensure WASM target is installed
rustup target add wasm32-unknown-unknown

# Clean and rebuild
cargo clean
cargo build --release
```

### Tests Fail to Compile

```bash
# Tests must run on native target, not WASM
cargo test --target $(rustc --print host-tuple)
```

### Cartridge Too Large

Check size:
```bash
ls -la target/wasm32-unknown-unknown/release/cart.wasm
# Must be < 65536 bytes
```

Optimization tips:
- Enable LTO in `Cargo.toml`: `lto = true`
- Use `opt-level = "z"` for size
- Avoid `format!()` macro (use manual string building)
- Use fixed arrays instead of Vec

### Pre-Commit Hook Not Running

```bash
# Enable hooks
git config core.hooksPath .githooks

# Make hook executable
chmod +x .githooks/pre-commit
```

### Web Dev Server Issues

```bash
cd web

# Clear cache and reinstall
rm -rf node_modules pnpm-lock.yaml .astro
pnpm install
pnpm dev
```

### PWA Not Working

1. Check `manifest.json` paths match deployment URL
2. Verify service worker registered in browser DevTools
3. Check HTTPS is enabled (required for service workers)

---

## References

- [WASM-4 Documentation](https://wasm4.org/docs)
- [Astro Documentation](https://docs.astro.build)
- [Rust Book](https://doc.rust-lang.org/book/)
- [GitHub Actions](https://docs.github.com/en/actions)
