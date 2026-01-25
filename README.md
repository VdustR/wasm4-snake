# WASM-4 Snake

A classic Snake game built for the [WASM-4](https://wasm4.org) fantasy console using Rust, with a modern PWA landing page built with Astro.

## Play Online

After enabling GitHub Pages, your game will be available at:
`https://vdustr.github.io/wasm4-snake/`

## Features

### Game
- 🐍 Classic snake gameplay with screen wrapping
- 🎮 Menu system (main menu, difficulty select, pause, game over)
- 🎯 5 difficulty levels with AI enemies
- 🤖 AI-controlled enemy snakes with BFS pathfinding
- 🍎 3 food sizes (Small +1, Medium +2, Large +3)
- ⚡ Energy system for speed boosts
- 🏆 High score persistence per difficulty
- 🔊 Sound effects (eat, game over) - toggleable
- 🎵 Background music (8-bit melody loop) - toggleable
- ⚙️ Settings persistence (sound preferences saved)
- 🎨 4-color retro palette

### Game Modes

#### Classic Mode
- Traditional snake rules: collision = instant death
- No length limit, grow indefinitely
- Free speed control (no energy cost)
- No enemies, pure survival

#### Battle Modes (Noob, Normal, Hell, Nightmare)
- **Collision = Shrink**: Hitting obstacles shrinks your snake by 1
- **Death Threshold**: Die only when at minimum length (3) and collide
- **Length Limit**: Max length varies by difficulty (12-20)
- **Energy System**:
  - Speed boost consumes 1 energy
  - Gain energy by eating food at max length
  - Max energy: 10
- **Body Attack** (Player vs Enemy):
  - Your head hits enemy body: you shrink -1, enemy grows +1
  - Enemy head hits your body: enemy shrinks -1, you grow +1
- **Enemy vs Enemy Combat**:
  - Enemies also attack each other using the same rules
  - Head-to-head collision: both enemies shrink
  - Head hits body: attacker shrinks -1, defender grows +1
  - Enemies die when shrinking below minimum length (3)

### Difficulty Levels

| Level | Enemies | Max Length | AI Behavior |
|-------|---------|------------|-------------|
| Classic | 0 | ∞ | Traditional instant-death |
| Noob | 2 | 20 | Low aggression, basic AI |
| Normal | 3 | 18 | Balanced gameplay |
| Hell | 5 | 15 | Aggressive AI with energy use |
| Nightmare | 8 | 12 | Maximum chaos, smart energy use |

### Web
- 📱 Responsive design (mobile-first)
- 🔍 SEO optimized (meta tags, Open Graph, Schema.org)
- 📲 PWA support (offline play, installable)
- ⚡ Zero JavaScript by default (Astro)

## Controls

| Key | Action |
|-----|--------|
| Arrow Keys / D-pad | Move snake / Navigate menu |
| X (Button 1) | Select / Speed Up (costs 1 energy in battle modes) |
| Z (Button 2) | Back / Speed Down (free) |
| X + Z together | Pause game (access sound settings) |

## Quick Start

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable | Game development |
| Node.js | 22+ | Web development |
| pnpm | 9+ | Package management |
| w4 CLI | latest | WASM-4 development |

### Setup

```bash
# Clone repository
git clone https://github.com/VdustR/wasm4-snake.git
cd wasm4-snake

# Setup Rust
rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy

# Setup git hooks
git config core.hooksPath .githooks

# Build game
cargo build --release

# Run game locally
w4 run target/wasm32-unknown-unknown/release/cart.wasm
```

### Web Development

```bash
cd web
nvm use        # Use correct Node version
corepack enable
pnpm install
pnpm dev       # Start at http://localhost:4321
```

See [web/README.md](web/README.md) for detailed web documentation.

## Project Structure

```
wasm4-snake/
├── src/                    # Rust game source
│   ├── lib.rs              # Entry point
│   ├── wasm4.rs            # WASM-4 API bindings
│   ├── alloc.rs            # Memory allocator
│   ├── game.rs             # Game state & loop
│   ├── snake.rs            # Snake logic
│   ├── food.rs             # Food logic (with sizes)
│   ├── rng.rs              # Random number generator
│   ├── enemy.rs            # Enemy snake system
│   ├── ai.rs               # AI pathfinding (BFS)
│   └── menu.rs             # Menu rendering
│
├── web/                    # Astro landing page
│   ├── public/             # Static assets
│   ├── src/                # Astro components & pages
│   └── README.md           # Web documentation
│
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md     # Design decisions
│   ├── DEVELOPMENT.md      # Development guide
│   └── WORKFLOW.md         # Complete workflow
│
├── .claude/skills/         # Claude Code skills
│   ├── wasm4-rust.md       # WASM-4 best practices
│   └── astro-best-practices.md
│
├── .github/workflows/      # CI/CD
│   └── deploy.yml          # GitHub Pages deployment
│
├── .vscode/                # VSCode settings
├── .githooks/              # Git hooks (pre-commit)
├── Cargo.toml              # Rust dependencies
└── README.md               # This file
```

## Development Commands

### Game (Rust)

```bash
cargo build --release       # Build game
cargo test --target $(rustc --print host-tuple)  # Run tests
cargo fmt --check           # Check formatting
cargo clippy                # Lint code
w4 run target/wasm32-unknown-unknown/release/cart.wasm  # Run game
w4 watch                    # Watch mode
```

### Web (Astro)

```bash
cd web
pnpm dev                    # Development server
pnpm build                  # Production build
pnpm preview                # Preview build
```

## WASM-4 Constraints

| Resource | Limit |
|----------|-------|
| Display | 160×160 pixels |
| Colors | 4-color palette |
| Memory | 64 KB RAM |
| Cartridge | 64 KB max |
| Frame Rate | 60 FPS |
| Audio | 4 channels |

## Deployment

### Automatic (GitHub Actions)

1. Push to `main` branch
2. GitHub Actions:
   - Checks Rust formatting and linting
   - Builds WASM-4 game
   - Runs unit tests
   - Bundles game as HTML
   - Builds Astro site
   - Deploys to GitHub Pages

### Enable GitHub Pages

1. Go to repository **Settings** → **Pages**
2. Set **Source** to "GitHub Actions"

## Documentation

- [Development Guide](docs/DEVELOPMENT.md) - Commands and debugging
- [Architecture](docs/ARCHITECTURE.md) - Design decisions
- [Workflow](docs/WORKFLOW.md) - Complete development flow
- [Web README](web/README.md) - Astro landing page

## Author

**VdustR (ViPro)**

- GitHub: [@VdustR](https://github.com/VdustR)

## License

MIT

## Links

- [WASM-4 Documentation](https://wasm4.org/docs)
- [WASM-4 Snake Tutorial](https://wasm4.org/docs/tutorials/snake/goal)
- [Astro Documentation](https://docs.astro.build)
