import {
    RouterProvider,
    createMemoryHistory,
    createRootRoute,
    createRoute,
    createRouter,
} from "@tanstack/react-router"
import { afterEach, beforeEach, describe, expect, it, type Mock, vi } from "vitest"
import { getMessage } from "../../api/message/message.ts"
import { getChannelMessages } from "../../api/channel/channel.ts"
import { createMockMessage } from "../../test/factories.ts"
import { renderWithProviders, screen, waitFor } from "../../test/utils.tsx"
import { ChannelView } from "./ChannelView.tsx"

// Mock API modules
vi.mock("../../api/message/message.ts", () => ({
    getMessage: vi.fn(),
    useMarkMessagesAsRead: () => ({ mutate: vi.fn() }),
    useAddMessageStamp: () => ({ mutate: vi.fn() }),
    useRemoveMessageStamp: () => ({ mutate: vi.fn() }),
}))

vi.mock("../../api/channel/channel.ts", () => ({
    getChannelMessages: vi.fn(),
}))

vi.mock("@mantine/hooks", () => ({
    useIntersection: () => ({
        ref: { current: null },
        entry: { isIntersecting: true },
    }),
    useMediaQuery: () => true, // Desktop mode
}))

// Create test router with channel route
const createTestRouter = (channelId: string, messageId?: string) => {
    const rootRoute = createRootRoute()
    const channelRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/channels/$channelId",
        validateSearch: (search: Record<string, unknown>) => ({
            messageId: typeof search.messageId === "string" ? search.messageId : undefined,
        }),
        component: () => <ChannelView />,
    })

    const routeTree = rootRoute.addChildren([channelRoute])

    const searchParams = messageId ? `?messageId=${messageId}` : ""
    const history = createMemoryHistory({
        initialEntries: [`/channels/${channelId}${searchParams}`],
    })

    return createRouter({
        routeTree,
        history,
    })
}

describe("ChannelView", () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    afterEach(() => {
        vi.resetAllMocks()
    })

    it("renders messages from API", async () => {
        const mockMessages = [
            createMockMessage({ id: "msg-1", content: "Test message 1" }),
            createMockMessage({ id: "msg-2", content: "Test message 2" }),
        ]

            ; (getMessage as Mock).mockResolvedValue({ data: null })
            ; (getChannelMessages as Mock).mockResolvedValue({ data: mockMessages })

        const router = createTestRouter("test-channel-id")
        renderWithProviders(<></>, { router })

        await waitFor(
            () => {
                const articles = screen.queryAllByRole("article")
                expect(articles.length).toBeGreaterThan(0)
            },
            { timeout: 3000 }
        )
    })

    it("displays back button", async () => {
        const mockMessages = [createMockMessage()]

            ; (getMessage as Mock).mockResolvedValue({ data: null })
            ; (getChannelMessages as Mock).mockResolvedValue({ data: mockMessages })

        const router = createTestRouter("test-channel-id")
        renderWithProviders(<></>, { router })

        await waitFor(() => {
            const backButton = screen.getByRole("button", { name: /戻る/i })
            expect(backButton).toBeTruthy()
        })
    })

    it("fetches anchor message when messageId is provided", async () => {
        const anchorMessage = createMockMessage({ id: "anchor-msg" })

            ; (getMessage as Mock).mockResolvedValue({ data: anchorMessage })
            ; (getChannelMessages as Mock).mockResolvedValue({ data: [] })

        const router = createTestRouter("test-channel-id", "anchor-msg")
        renderWithProviders(<></>, { router })

        await waitFor(() => {
            expect(getMessage).toHaveBeenCalledWith("anchor-msg", expect.any(Object))
        })
    })

    it("shows error state on API failure", async () => {
        ; (getMessage as Mock).mockResolvedValue({ data: null })
            ; (getChannelMessages as Mock).mockRejectedValue(new Error("API Error"))

        const router = createTestRouter("test-channel-id")
        const { container } = renderWithProviders(<></>, { router })

        await waitFor(
            () => {
                // Check for error alert
                const alert = container.querySelector('[class*="Alert"]')
                expect(alert).toBeTruthy()
            },
            { timeout: 3000 }
        )
    })
})
