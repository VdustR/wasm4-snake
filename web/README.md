# WASM-4 Snake - Landing Page

A modern, responsive landing page for the WASM-4 Snake game built with Astro.

## Features

- 🎮 Embedded WASM-4 game
- 🎨 8-bit retro design with custom fonts
- 📱 Responsive design (mobile-first)
- 🔍 SEO optimized (meta tags, Open Graph, Schema.org)
- 📲 PWA support (offline play, installable)
- ⚡ Zero JavaScript by default (Astro)

## Quick Start

```bash
# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview
```

## Prerequisites

- **Node.js** 22+ (use `.nvmrc`)
- **pnpm** 9+ (via Corepack)

```bash
# Setup with nvm
nvm use

# Enable pnpm via Corepack
corepack enable

# Install dependencies
pnpm install
```

## Development

### Project Structure

```
web/
├── public/                 # Static assets
│   ├── game/               # WASM-4 bundled HTML (from build)
│   │   └── index.html
│   ├── favicon.svg         # Browser favicon
│   ├── icon-192.svg        # PWA icon (small)
│   ├── icon-512.svg        # PWA icon (large)
│   ├── og-image.svg        # Social share image
│   ├── manifest.json       # PWA manifest
│   └── sw.js               # Service worker
│
├── src/
│   ├── components/         # Reusable Astro components
│   │   └── GameEmbed.astro # WASM-4 game iframe wrapper
│   │
│   ├── layouts/            # Page layouts
│   │   └── Layout.astro    # Base layout with SEO/PWA
│   │
│   ├── pages/              # Route pages
│   │   └── index.astro     # Homepage
│   │
│   ├── styles/             # Global styles
│   │   └── global.css      # CSS variables, typography, etc.
│   │
│   └── env.d.ts            # TypeScript environment types
│
├── astro.config.mjs        # Astro configuration
├── package.json            # Dependencies and scripts
├── tsconfig.json           # TypeScript configuration
├── eslint.config.js        # ESLint flat config
├── .prettierrc             # Prettier configuration
├── .prettierignore         # Prettier ignore patterns
└── .nvmrc                  # Node.js version
```

### Running with the Game

To test the full site with the embedded game locally:

```bash
# 1. Build the WASM-4 game (from project root)
cd ..
cargo build --release

# 2. Bundle game into web/public/game/
mkdir -p web/public/game
w4 bundle target/wasm32-unknown-unknown/release/cart.wasm \
    --title "WASM-4 Snake" \
    --html web/public/game/index.html

# 3. Start Astro dev server
cd web
pnpm dev
```

### Commands

| Command                 | Description                                 |
| ----------------------- | ------------------------------------------- |
| `pnpm dev`              | Start dev server at `http://localhost:4321` |
| `pnpm build`            | Build for production to `./dist/`           |
| `pnpm preview`          | Preview production build locally            |
| `pnpm check`            | Run Astro type checking                     |
| `pnpm lint`             | Run ESLint                                  |
| `pnpm lint:fix`         | Run ESLint with auto-fix                    |
| `pnpm format`           | Format code with Prettier                   |
| `pnpm format:check`     | Check code formatting                       |
| `pnpm exec lint-staged` | Run lint-staged on staged files             |

### lint-staged (Pre-commit)

lint-staged runs automatically on `git commit` (via `.githooks/pre-commit`) for staged web files:

```json
{
  "lint-staged": {
    "*.{js,ts}": ["eslint --fix", "prettier --write"],
    "*.astro": ["eslint --fix", "prettier --write"],
    "*.{json,md,css}": ["prettier --write"]
  }
}
```

## Configuration

### Astro Config (`astro.config.mjs`)

```javascript
export default defineConfig({
  site: 'https://example.github.io/wasm4-snake',
  base: '/wasm4-snake',
  // ...
});
```

Update `site` and `base` for your deployment URL.

### PWA Manifest (`public/manifest.json`)

Update `start_url` and icon paths if your base path changes.

### Service Worker (`public/sw.js`)

Update `BASE_PATH` constant if your deployment path changes.

## Design System

### Color Palette (WASM-4)

| Variable         | Hex       | Usage            |
| ---------------- | --------- | ---------------- |
| `--color-bg`     | `#1a1c2c` | Background       |
| `--color-purple` | `#5d275d` | Accents          |
| `--color-green`  | `#38b764` | Snake body       |
| `--color-yellow` | `#f6c64f` | Snake head, food |

### Typography

- **Headings**: "Press Start 2P" (pixel font)
- **Body**: "VT323" (retro terminal font)

Fonts loaded from Google Fonts in `global.css`.

### Responsive Breakpoints

```css
/* Mobile (default) */
/* Tablet: 768px */
/* Desktop: 1024px */
```

## SEO

### Meta Tags

The `Layout.astro` component includes:

- Primary meta tags (title, description)
- Open Graph tags (Facebook, LinkedIn)
- Twitter Card tags
- Canonical URL
- Schema.org structured data (VideoGame type)

### Sitemap

Generated automatically by `@astrojs/sitemap` integration.

## PWA

### Features

- **Offline support**: Service worker caches essential assets
- **Installable**: manifest.json enables "Add to Home Screen"
- **Theme color**: Matches app design

### Testing PWA

1. Build: `pnpm build`
2. Serve with HTTPS (required for service workers)
3. Open Chrome DevTools → Application → Service Workers

## Deployment

### GitHub Pages (Automatic)

Push to `main` branch triggers GitHub Actions:

1. Builds WASM-4 game
2. Bundles game to `web/public/game/`
3. Builds Astro site
4. Deploys to GitHub Pages

### Manual Deployment

```bash
# Build
pnpm build

# Deploy dist/ folder to any static host
```

## Accessibility

- All images have `alt` text
- Keyboard navigation supported
- Color contrast meets WCAG AA
- Reduced motion respected

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## License

MIT
