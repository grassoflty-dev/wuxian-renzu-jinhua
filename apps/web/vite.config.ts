import { defineConfig } from "vite";
import { runtimeAssetBundlePlugin } from "./scripts/runtime-asset-bundle.mjs";
import { sceneDefinitionBundlePlugin } from "./scripts/scene-definition-bundle.mjs";
import { buildIdentityPlugin } from "./scripts/build-identity.mjs";
import { bundleIdentityPlugin } from "./scripts/bundle-identity.mjs";

export default defineConfig({
  base: "./",
  publicDir: false,
  build: { outDir: "dist", emptyOutDir: true },
  clearScreen: false,
  plugins: [runtimeAssetBundlePlugin(), sceneDefinitionBundlePlugin(), buildIdentityPlugin(), bundleIdentityPlugin()],
});
