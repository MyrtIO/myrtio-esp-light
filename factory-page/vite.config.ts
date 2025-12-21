import { viteSingleFile } from "vite-plugin-singlefile"
import { defineConfig } from "vite";

export default defineConfig({
    plugins: [viteSingleFile()]
});
