import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

export default defineConfig({
  cacheDir: "node_modules/.vite",
  plugins: [
    react(),
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
