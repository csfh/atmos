# Atmos site

Product landing page for [Atmos](https://github.com/csfh/atmos), a Quickshell preferences window for [Omarchy](https://omarchy.org).

This subtree is the Vite source for [atmos.csfh.dev](https://atmos.csfh.dev). Screenshots live in `public/screenshots/` (WebP + PNG). The same PNG set is in [`docs/screenshots/`](../docs/screenshots/) for the repository README.

## Local development

```bash
cd site
npm install
npm run dev
```

That starts Vite at http://localhost:5173.

```bash
cd site
npm run build
npm run preview
```

`npm run build` writes static files to `site/dist/`. `npm run preview` serves that output locally.

## Cloudflare Pages

Vanilla Vite static site. No Workers or Functions. Suggested dashboard settings:

| Setting | Value |
| --- | --- |
| Root directory | `site` |
| Production branch | `main` |
| Framework preset | None (or **React (Vite)** — same command and output) |
| Build command | `npm run build` |
| Build output directory | `dist` |
| Node.js | `22` (optional `NODE_VERSION=22`; `.node-version` is in this directory) |

New Pages projects on the V3 build image already default to Node 22. `_headers`, `robots.txt`, `sitemap.xml`, and `404.html` are static files in `public/`.

## License

MIT. Copyright (c) 2026 Christoffer Hallas. See the repository [LICENSE](../LICENSE).

Instrument Serif and IBM Plex are SIL OFL. See [public/fonts/NOTICE](public/fonts/NOTICE).
