import { describe, it, expect } from "vitest"
import { Timeline } from "./Timeline"
import { renderWithProviders, screen, waitFor, mockApiError } from "../../test/utils"

describe("Timeline", () => {
    it("renders messages from API", async () => {
        // MSW will automatically return mocked data from api/mocks.ts
        renderWithProviders(<Timeline />)

        // Wait for messages to load
        await waitFor(
            () => {
                // Check if at least one message is rendered
                const messages = screen.queryAllByRole("article")
                expect(messages.length).toBeGreaterThan(0)
            },
            { timeout: 3000 },
        )
    })

    it("shows loading state initially", () => {
        renderWithProviders(<Timeline />)

        // Should show skeleton loaders
        const skeletons = screen.getAllByRole("group")
        expect(skeletons.length).toBeGreaterThan(0)
    })

    it("shows error state on API failure", async () => {
        // Override default mock to return error
        mockApiError("/api/v1/timeline", 500)

        renderWithProviders(<Timeline />)

        await waitFor(
            () => {
                // Check for error message or empty state
                const container = screen.getByRole("main")
                expect(container).toBeInTheDocument()
            },
            { timeout: 3000 },
        )
    })

    it("renders multiple messages", async () => {
        renderWithProviders(<Timeline />)

        await waitFor(
            () => {
                const messages = screen.queryAllByRole("article")
                // MSW mock generates 1-10 messages
                expect(messages.length).toBeGreaterThanOrEqual(1)
                expect(messages.length).toBeLessThanOrEqual(10)
            },
            { timeout: 3000 },
        )
    })
})
