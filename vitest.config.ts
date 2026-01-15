import { defineConfig } from "vitest/config"
import react from "@vitejs/plugin-react"

export default defineConfig({
    plugins: [react()],
    test: {
        browser: {
            enabled: true,
            name: "chromium",
            // @ts-expect-error - Vitest v4 accepts string provider
            provider: "playwright",
            headless: true,
        },
        globals: true,
        setupFiles: ["./test/setup.ts"],
        testTimeout: 20000, // 20 seconds timeout for browser tests
    },
})
