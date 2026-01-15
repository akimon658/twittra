import { defineConfig } from "vitest/config"
import react from "@vitejs/plugin-react"
import { playwright } from "@vitest/browser-playwright"

export default defineConfig({
    plugins: [react()],
    test: {
        browser: {
            enabled: true,
            provider: playwright(),
            instances: [
                { browser: "chromium" },
            ],
            headless: true,
            screenshotFailures: false, // Disable automatic screenshots
        },
        globals: true,
        setupFiles: ["./test/setup.ts"],
        testTimeout: 20000,
    },
})
