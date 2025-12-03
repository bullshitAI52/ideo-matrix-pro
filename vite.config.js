import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  root: "src",              // 指向包含 index.html 的目录
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    outDir: "../dist",     // 构建输出到仓库根的 dist，src-tauri/tauri.conf.json 已指向 ../dist
    emptyOutDir: true,
    target: ["es2021", "chrome100", "safari13"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
