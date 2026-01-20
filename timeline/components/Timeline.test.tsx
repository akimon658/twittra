import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import {
  mockApiError,
  renderWithProviders,
  screen,
  waitFor,
} from "../../test/utils.tsx"
import { Timeline } from "./Timeline.tsx"

// Mock socket.io-client
const mockSocket = {
  on: vi.fn(),
  off: vi.fn(),
  emit: vi.fn(),
  close: vi.fn(),
}

vi.mock("socket.io-client", () => ({
  io: vi.fn(() => mockSocket),
}))

describe("Timeline", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

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
    const { container } = renderWithProviders(<Timeline />)

    // Should show skeleton loaders (they have data-visible="true")
    const skeletons = container.querySelectorAll('[data-visible="true"]')
    expect(skeletons.length).toBeGreaterThan(0)
  })

  it("shows error state on API failure", async () => {
    // Override default mock to return error
    mockApiError("/api/v1/timeline", 500)

    const { container } = renderWithProviders(<Timeline />)

    await waitFor(
      () => {
        // Check for error alert (Mantine Alert component)
        const alert = container.querySelector('[class*="Alert"]')
        expect(alert).toBeInTheDocument()
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

  describe("Socket.io Integration", () => {
    it("registers socket event listener for messages_updated", async () => {
      renderWithProviders(<Timeline />)

      await waitFor(() => {
        expect(mockSocket.on).toHaveBeenCalledWith(
          "messages_updated",
          expect.any(Function),
        )
      })
    })

    it("cleans up socket listener on unmount", async () => {
      const { unmount } = renderWithProviders(<Timeline />)

      await waitFor(() => {
        expect(mockSocket.on).toHaveBeenCalled()
      })

      unmount()

      expect(mockSocket.off).toHaveBeenCalledWith(
        "messages_updated",
        expect.any(Function),
      )
    })
  })
})
