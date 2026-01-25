import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import icon from 'astro-icon';

// Use ASTRO_BASE env var in CI, fallback to /wasm4-snake for local dev
const base = process.env.ASTRO_BASE || '/wasm4-snake';

export default defineConfig({
  // Site URL for canonical URLs and OG images
  site: 'https://vdustr.dev',
  base: base,
  build: {
    // Output to dist/ folder
    outDir: './dist',
  },
  integrations: [
    sitemap(),
    icon({
      include: {
        pixelarticons: ['*'],
      },
    }),
  ],
  vite: {
    build: {
      // Inline small assets for better performance
      assetsInlineLimit: 4096,
    },
  },
});
