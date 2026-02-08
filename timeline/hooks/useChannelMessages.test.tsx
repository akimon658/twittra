import { renderHook, waitFor } from "@testing-library/react"
import {
    afterEach,
    beforeEach,
    describe,
    expect,
    it,
    type Mock,
    vi,
} from "vitest"
import { getMessage } from "../../api/message/message.ts"
import { getChannelMessages } from "../../api/channel/channel.ts"
import { createMockMessage } from "../../test/factories.ts"
import { createTestWrapper } from "../../test/utils.tsx"
import { useChannelMessages } from "./useChannelMessages.ts"

// Mock API modules
vi.mock("../../api/message/message.ts", () => ({
    getMessage: vi.fn(),
}))

vi.mock("../../api/channel/channel.ts", () => ({
    getChannelMessages: vi.fn(),
}))

describe("useChannelMessages", () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    afterEach(() => {
        vi.resetAllMocks()
    })

    it("fetches latest messages when no messageId is provided", async () => {
        const mockMessages = [
            createMockMessage({ id: "msg-1", createdAt: new Date("2024-01-15T11:00:00Z") }),
            createMockMessage({ id: "msg-2", createdAt: new Date("2024-01-15T12:00:00Z") }),
        ]

            ; (getMessage as Mock).mockResolvedValue({ data: null })
            ; (getChannelMessages as Mock).mockResolvedValue({ data: mockMessages })

        const { result } = renderHook(
            () => useChannelMessages({ channelId: "channel-1" }),
            { wrapper: createTestWrapper() }
        )

        await waitFor(() => {
            expect(result.current.messages).toHaveLength(2)
        })

        expect(getChannelMessages).toHaveBeenCalledWith(
            "channel-1",
            expect.objectContaining({ order: "desc" }),
            expect.any(Object)
        )
    })

    it("fetches anchor message and surrounding messages when messageId is provided", async () => {
        const anchorMessage = createMockMessage({
            id: "anchor-msg",
            createdAt: new Date("2024-01-15T12:00:00Z"),
        })

        const olderMessages = [
            createMockMessage({ id: "older-1", createdAt: new Date("2024-01-15T11:00:00Z") }),
            createMockMessage({ id: "older-2", createdAt: new Date("2024-01-15T11:30:00Z") }),
        ]

        const newerMessages = [
            createMockMessage({ id: "newer-1", createdAt: new Date("2024-01-15T12:30:00Z") }),
        ]

            ; (getMessage as Mock).mockResolvedValue({ data: anchorMessage })
            ; (getChannelMessages as Mock)
                .mockResolvedValueOnce({ data: olderMessages }) // until call
                .mockResolvedValueOnce({ data: newerMessages }) // since call

        const { result } = renderHook(
            () => useChannelMessages({ channelId: "channel-1", messageId: "anchor-msg" }),
            { wrapper: createTestWrapper() }
        )

        await waitFor(() => {
            expect(result.current.messages.length).toBeGreaterThan(0)
        })

        expect(getMessage).toHaveBeenCalledWith("anchor-msg", expect.any(Object))

        expect(getChannelMessages).toHaveBeenCalledWith(
            "channel-1",
            expect.objectContaining({ until: anchorMessage.createdAt, order: "desc" }),
            expect.any(Object)
        )
        expect(getChannelMessages).toHaveBeenCalledWith(
            "channel-1",
            expect.objectContaining({ since: anchorMessage.createdAt, order: "asc" }),
            expect.any(Object)
        )
    })

    it("returns anchorMessageId when messageId is provided", async () => {
        ; (getMessage as Mock).mockResolvedValue({ data: createMockMessage({ id: "anchor-msg" }) })
            ; (getChannelMessages as Mock).mockResolvedValue({ data: [] })

        const { result } = renderHook(
            () => useChannelMessages({ channelId: "channel-1", messageId: "anchor-msg" }),
            { wrapper: createTestWrapper() }
        )

        await waitFor(() => {
            expect(result.current.anchorMessageId).toBe("anchor-msg")
        })
    })

    it("provides pagination handlers", async () => {
        ; (getMessage as Mock).mockResolvedValue({ data: null })
            ; (getChannelMessages as Mock).mockResolvedValue({
                data: [createMockMessage()],
            })

        const { result } = renderHook(
            () => useChannelMessages({ channelId: "channel-1" }),
            { wrapper: createTestWrapper() }
        )

        await waitFor(() => {
            expect(result.current.messages).toHaveLength(1)
        })

        expect(typeof result.current.fetchOlderMessages).toBe("function")
        expect(typeof result.current.fetchNewerMessages).toBe("function")
        expect(typeof result.current.hasPreviousPage).toBe("boolean")
        expect(typeof result.current.hasNextPage).toBe("boolean")
    })
})
