import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"
import { viteWebfontDownload } from "vite-plugin-webfont-dl"

export default defineConfig({
  cacheDir: "node_modules/.vite",
  plugins: [
    react(),
    viteWebfontDownload([
      "https://fonts.googleapis.com/css2?family=Google+Sans+Flex:opsz,wdth,wght@6..144,87.5,1..1000&display=swap",
      "https://fonts.googleapis.com/css2?family=Noto+Color+Emoji&display=swap",
    ]),
  ],
  server: {
    proxy: {
      "/api/v1": {
        target: "http://localhost:8080",
      },
      "/docs": {
        target: "http://localhost:8080",
      },
    },
  },
})
