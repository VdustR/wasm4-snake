# Dependency security dispositions

## Security baseline

The web package uses pnpm overrides for the smallest compatible patched
transitive versions. Keep the manifest and generated lockfile together. These
updates do not change the Astro version, application code, or Node.js minimum.

## Patched Vite development tooling

The web package pins transitive Vite to 8.2.4 through the existing pnpm override
mechanism. Astro 7.2.10 accepts this version through its `^8.0.13` dependency.
The generated lockfile replaces Vite 8.2.2 without upgrading Astro or unrelated
packages.

Vite 8.2.4 is the official patched version for
[editor CLI argument handling](https://github.com/advisories/GHSA-r825-w8mm-5pxw),
[Windows path existence checks](https://github.com/advisories/GHSA-587h-7wmj-h5pm),
and [Windows command argument escaping](https://github.com/advisories/GHSA-698v-4r3x-3vcv).
These advisories concern development-server editor integration on Windows or
WSL under the conditions in each advisory. The production site is static output
on GitHub Pages; it does not run the Vite development server.

The hosted Dependabot update reported that it could only resolve Vite 8.2.2,
despite finding the patched tarball. The explicit override makes the selected
patch reproducible through the repository's declared pnpm 9.15.4 toolchain.
This fixes the repository resolution; it does not establish the cause of the
hosted updater's failure or claim that the updater itself was repaired.

## Advisory paths not used by this repository

These are repository-specific applicability decisions, not claims that the
upstream packages are patched or that the advisory databases are empty.

### http-cache-semantics — GHSA-ch52-4w7c-c8xp

[The advisory](https://github.com/advisories/GHSA-ch52-4w7c-c8xp) describes
reuse of shared-cache responses when a client supplies `Cache-Control:
max-stale`. It lists no patched version. The upstream maintainer
[disputes the report's caching premise](https://github.com/kornelski/http-cache-semantics/issues/56#issuecomment-5975759591).

This repository does not use the affected request path:

- `web/astro.config.mjs` has static output by default and no SSR adapter.
- The deployment workflow uploads `web/dist` to GitHub Pages. There is no
  application server receiving visitor cache directives.
- The application does not import `astro:assets` or fetch remote images through
  Astro's image pipeline.
- Installed Astro 7.2.10 references `CachePolicy` only in its build-time remote
  image loader. That loader constructs its own `Request`, gates expiration on
  `storable()` and `timeToLive()`, and does not call
  `satisfiesWithoutRevalidation()` or forward visitor `max-stale` directives.

Keep `http-cache-semantics` at its existing version. Upgrading merely beyond the
advisory's affected range would not prove the reported behavior was fixed.
The repository alert can be dismissed as **vulnerable code not used**.
Reassess this decision before adding SSR, shared response caching, remote-image
processing, or forwarding visitor cache headers.

### braces — GHSA-vfj7-8cjw-p6xm

[The advisory](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm) describes stack
exhaustion when parsing deeply nested brace patterns. There is no patched
version, and the installed package still reproduces that failure.

The vulnerable pattern input is not exposed by the current tooling:

- `lint-staged` supplies the fixed, shallow patterns in `web/package.json`.
  Candidate filenames are matched against those patterns; they are not parsed
  as glob patterns.
- The other path is Astro ESLint parser through `fast-glob` and `micromatch`.
  Project glob discovery requires `parserOptions.project`, which is absent
  from `web/eslint.config.js`.

Keep the unpatched advisory visible in raw `pnpm audit` output. Do not suppress
the audit or claim this dependency was fixed. Reassess if a tool begins accepting
untrusted glob patterns, or if ESLint project glob configuration changes.
