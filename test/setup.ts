import { cleanup } from "@testing-library/react"
import { setupWorker } from "msw/browser"
import { afterAll, afterEach, beforeAll } from "vitest"
import { handlers } from "../api/mocks.ts"

// Create MSW worker for browser mode
export const worker = setupWorker(...handlers)

// Start MSW before all tests
beforeAll(async () => {
  await worker.start({
    onUnhandledRequest: "warn",
  })
})

// Cleanup after each test
afterEach(() => {
  cleanup()
  // Reset handlers to default state
  worker.resetHandlers()
})

// Stop MSW after all tests
afterAll(() => {
  worker.stop()
})
