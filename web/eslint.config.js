// @ts-check
import eslintPluginAstro from 'eslint-plugin-astro';
import tsParser from '@typescript-eslint/parser';

export default [
  // Astro recommended config
  ...eslintPluginAstro.configs.recommended,

  // TypeScript files
  {
    files: ['**/*.ts', '**/*.tsx'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
      },
    },
    rules: {
      // TypeScript specific rules
      'no-unused-vars': 'off', // Handled by TypeScript
    },
  },

  // Astro files
  {
    files: ['**/*.astro'],
    rules: {
      // Astro specific rules
      'astro/no-set-html-directive': 'error',
    },
  },

  // Ignore patterns
  {
    ignores: ['dist/**', '.astro/**', 'node_modules/**', 'public/game/**'],
  },
];
