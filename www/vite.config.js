import { defineConfig } from "vite";

export default defineConfig({
  // Static Pages site. Unknown paths should 404, not fall back to index.html.
  appType: "mpa",
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
