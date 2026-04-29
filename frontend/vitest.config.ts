import path from "path";
import { defineConfig } from "vitest/config";

export default defineConfig({
  esbuild: {
    jsx: "automatic",
    jsxFactory: "React.createElement",
    jsxFragment: "React.Fragment",
  },
  test: {
    environment: "jsdom",
    globals: true,
    transformMode: {
      web: [/\.tsx?$/],
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "."),
    },
  },
});
