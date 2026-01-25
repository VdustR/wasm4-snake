# WASM-4 Snake

A classic Snake game built for the [WASM-4](https://wasm4.org) fantasy console using Rust, with a modern PWA landing page built with Astro.

## Play Online

**[https://vdustr.github.io/wasm4-snake/](https://vdustr.github.io/wasm4-snake/)**

## Features

- Classic snake gameplay with screen wrapping
- 5 difficulty levels: Classic, Noob, Normal, Hell, Nightmare
- Battle modes with AI enemies (BFS pathfinding)
- Energy system for speed boost/slowdown abilities
- 3 food sizes with different rewards
- High score persistence per difficulty
- Sound effects and background music (toggleable)
- PWA support (offline play, installable)

See [Game Design](docs/GAME_DESIGN.md) for complete game mechanics.

## Quick Start

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- w4 CLI (`npm install -g wasm4`)
- [lefthook](https://github.com/evilmartians/lefthook) for git hooks

### Build & Run

```bash
# Clone and setup
git clone https://github.com/VdustR/wasm4-snake.git
cd wasm4-snake
rustup target add wasm32-unknown-unknown
lefthook install

# Build and run
cargo build --release
w4 run target/wasm32-unknown-unknown/release/cart.wasm

# Or use watch mode
w4 watch
```

### Web Development

```bash
cd web
corepack enable
pnpm install
pnpm dev       # Start at http://localhost:4321
```

See [web/README.md](web/README.md) for web documentation.

## Documentation

| Document | Description |
|----------|-------------|
| [Game Design](docs/GAME_DESIGN.md) | Complete game mechanics and rules |
| [Architecture](docs/ARCHITECTURE.md) | Technical design decisions |
| [Development](docs/DEVELOPMENT.md) | Development guide and commands |
| [Workflow](docs/WORKFLOW.md) | Complete development workflow |
| [Web README](web/README.md) | Astro landing page documentation |

## Deployment

Push to `main` branch triggers GitHub Actions:
1. Rust formatting and linting checks
2. Build WASM-4 game and run tests
3. Build Astro site
4. Deploy to GitHub Pages

Enable GitHub Pages: **Settings** > **Pages** > Source: "GitHub Actions"

## Author

**VdustR (ViPro)** - [@VdustR](https://github.com/VdustR)

## License

MIT

## Links

- [WASM-4 Documentation](https://wasm4.org/docs)
- [WASM-4 Snake Tutorial](https://wasm4.org/docs/tutorials/snake/goal)
- [Astro Documentation](https://docs.astro.build)
