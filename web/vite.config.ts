import { resolve } from "node:path";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

const selectedTarget = process.env.AGENTIDE_RENDERER_TARGET;
const inputs = {
  index: resolve(import.meta.dirname, "index.html"),
  vanilla: resolve(import.meta.dirname, "renderers/vanilla/index.html"),
  vue: resolve(import.meta.dirname, "renderers/vue/index.html"),
};

export default defineConfig({
  plugins: [vue()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input:
        selectedTarget === "vanilla" || selectedTarget === "vue"
          ? { [selectedTarget]: inputs[selectedTarget] }
          : inputs,
    },
  },
});
