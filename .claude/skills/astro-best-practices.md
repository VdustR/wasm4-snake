# Astro Best Practices (v5+)

Best practices for developing static sites and web applications with Astro 5.

## When to Use This Skill

- Building static landing pages or marketing sites
- Creating PWA-enabled web applications
- Optimizing for SEO and performance
- Working with component-based architecture

## Astro 5 Key Features

### Content Layer (New)

The Astro Content Layer provides a unified, type-safe API for managing content:

```javascript
// astro.config.mjs
import { defineConfig } from 'astro/config';

export default defineConfig({
  experimental: {
    contentLayer: true,
  },
});
```

Benefits:
- Markdown builds up to **5x faster**
- MDX builds up to **2x faster**
- Memory usage reduced by **25-50%**

### Server Islands

For dynamic content (user avatars, shopping carts, reviews):

```astro
---
// Deferred server-rendered component
---
<UserProfile server:defer />
```

Benefits:
- More aggressive page caching
- Better performance for static content
- Dynamic personalization where needed

### Simplified Output Modes

Astro 5 merges `hybrid` and `static` into a single mode:

```javascript
// astro.config.mjs
export default defineConfig({
  // No mode needed - just add adapter for SSR routes
  adapter: netlify(), // optional
});
```

### Type-Safe Environment Variables

```javascript
// astro.config.mjs
export default defineConfig({
  experimental: {
    env: {
      schema: {
        API_URL: envField.string({ context: 'server', access: 'public' }),
        API_KEY: envField.string({ context: 'server', access: 'secret' }),
      },
    },
  },
});
```

```astro
---
import { API_URL } from 'astro:env/server';
---
```

## Project Structure

```
web/
├── public/              # Static assets (copied as-is)
│   ├── favicon.svg
│   ├── manifest.json    # PWA manifest
│   └── sw.js            # Service worker
├── src/
│   ├── components/      # Reusable Astro/framework components
│   ├── layouts/         # Page layouts with common structure
│   ├── pages/           # File-based routing (each file = route)
│   ├── styles/          # Global CSS and design tokens
│   └── env.d.ts         # TypeScript environment types
├── astro.config.mjs     # Astro configuration
├── package.json         # Dependencies and scripts
├── eslint.config.js     # ESLint flat config
├── .prettierrc          # Prettier config
└── tsconfig.json        # TypeScript configuration
```

## Core Principles

### 1. Zero JavaScript by Default

Astro ships zero JavaScript by default. Only add client-side JS when needed:

```astro
<!-- Static by default -->
<Button>Click me</Button>

<!-- Add interactivity only when needed -->
<Counter client:load />
```

### 2. Islands Architecture

Use Astro's islands for interactive components:

| Directive | When JS Loads |
|-----------|---------------|
| `client:load` | Immediately on page load |
| `client:idle` | After page becomes idle |
| `client:visible` | When component enters viewport |
| `client:media` | When media query matches |
| `client:only` | Only render on client (SSR skip) |

### 3. Component Props with TypeScript

Always type your component props:

```astro
---
interface Props {
  title: string;
  description?: string;
  variant?: 'primary' | 'secondary';
}

const { title, description, variant = 'primary' } = Astro.props;
---
```

## SEO Best Practices

### Meta Tags Template

```astro
---
const { title, description, image } = Astro.props;
const canonicalURL = new URL(Astro.url.pathname, Astro.site);
---

<head>
  <!-- Primary -->
  <title>{title}</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={canonicalURL} />

  <!-- Open Graph -->
  <meta property="og:type" content="website" />
  <meta property="og:url" content={canonicalURL} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:image" content={image} />

  <!-- Twitter -->
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={title} />
  <meta name="twitter:description" content={description} />
  <meta name="twitter:image" content={image} />
</head>
```

### Structured Data (Schema.org)

Add JSON-LD for rich search results:

```astro
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "WebApplication",
  "name": "App Name",
  "description": "Description",
  "applicationCategory": "Game"
}
</script>
```

## PWA Configuration

### manifest.json

```json
{
  "name": "Full App Name",
  "short_name": "Short",
  "description": "Description",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#000000",
  "icons": [
    { "src": "/icon-192.svg", "sizes": "192x192", "type": "image/svg+xml" },
    { "src": "/icon-512.svg", "sizes": "512x512", "type": "image/svg+xml" }
  ]
}
```

### Service Worker Best Practices

```javascript
// Version for cache busting
const VERSION = 'v1';
const CACHE_NAME = `app-${VERSION}`;

// Only cache essential assets
const ASSETS = ['/', '/index.html'];

self.addEventListener('install', (e) => {
  e.waitUntil(
    caches.open(CACHE_NAME).then(c => c.addAll(ASSETS))
  );
  self.skipWaiting();
});

self.addEventListener('activate', (e) => {
  // Clean old caches
  e.waitUntil(
    caches.keys().then(keys =>
      Promise.all(
        keys.filter(k => k !== CACHE_NAME).map(k => caches.delete(k))
      )
    )
  );
  self.clients.claim();
});

self.addEventListener('fetch', (e) => {
  e.respondWith(
    caches.match(e.request).then(r => r || fetch(e.request))
  );
});
```

**Key PWA Guidelines:**
- Cache wisely - don't over-cache
- Use version numbers for updates
- Always provide offline fallback
- Don't cache the service worker itself
- Test across different network conditions

## Performance Optimization

### 1. Image Optimization

Use `<Image>` component from `astro:assets`:

```astro
---
import { Image } from 'astro:assets';
import myImage from '../assets/image.png';
---

<Image src={myImage} alt="Description" width={800} />
```

### 2. CSS Best Practices

- Use scoped styles by default
- Define CSS custom properties in `:root`
- Use `clamp()` for responsive typography:

```css
h1 {
  font-size: clamp(1.5rem, 4vw, 3rem);
}
```

### 3. Preload Critical Assets

```astro
<link rel="preload" href="/fonts/main.woff2" as="font" crossorigin />
```

## Responsive Design (RWD)

### Mobile-First Breakpoints

```css
/* Mobile first (default) */
.container { padding: 1rem; }

/* Tablet */
@media (min-width: 768px) {
  .container { padding: 2rem; }
}

/* Desktop */
@media (min-width: 1024px) {
  .container { padding: 3rem; }
}
```

### Fluid Typography and Spacing

```css
:root {
  --spacing-sm: clamp(0.5rem, 1vw, 1rem);
  --spacing-md: clamp(1rem, 2vw, 2rem);
  --spacing-lg: clamp(2rem, 4vw, 4rem);
}
```

## Linting & Formatting

### ESLint (Flat Config)

```javascript
// eslint.config.js
import eslintPluginAstro from 'eslint-plugin-astro';

export default [
  ...eslintPluginAstro.configs.recommended,
  {
    ignores: ['dist/**', '.astro/**', 'node_modules/**'],
  },
];
```

### Prettier

```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "plugins": ["prettier-plugin-astro"],
  "overrides": [
    { "files": "*.astro", "options": { "parser": "astro" } }
  ]
}
```

### Scripts

```json
{
  "scripts": {
    "lint": "eslint . --ext .js,.ts,.astro",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "check": "astro check"
  }
}
```

## Upgrading Best Practices

1. **Backup first**: Create a backup branch before upgrading
2. **Update dependencies**: Ensure package.json and lockfiles are updated
3. **Test thoroughly**: Run locally, check for build errors
4. **Review breaking changes**: Check migration guide for Astro 5

## Accessibility Checklist

- [ ] All images have `alt` text
- [ ] Links have descriptive text (not "click here")
- [ ] Color contrast meets WCAG AA (4.5:1 for text)
- [ ] Interactive elements are keyboard accessible
- [ ] Form inputs have associated labels
- [ ] Page has proper heading hierarchy (h1 → h2 → h3)
- [ ] Reduced motion supported: `@media (prefers-reduced-motion: reduce)`

## Useful Integrations

| Integration | Purpose |
|-------------|---------|
| `@astrojs/sitemap` | Auto-generate sitemap.xml |
| `@astrojs/check` | TypeScript checking |
| `eslint-plugin-astro` | Linting for Astro files |
| `prettier-plugin-astro` | Formatting for Astro files |

## References

- [Astro 5.0 Release](https://astro.build/blog/astro-5/)
- [Astro Documentation](https://docs.astro.build)
- [MDN Web Docs - PWA](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps)
- [web.dev - Service Workers](https://web.dev/learn/pwa/service-workers)
