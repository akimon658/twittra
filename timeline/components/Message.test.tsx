import { describe, it, expect } from "vitest"
import { MessageItem } from "./Message"
import { renderWithProviders, screen } from "../../test/utils"
import type { MessageListItem } from "../../api/twittra.schemas"

describe("MessageItem", () => {
    const mockMessage: MessageListItem = {
        id: "123e4567-e89b-12d3-a456-426614174000",
        content: "Test message content",
        userId: "user-123",
        user: {
            id: "user-123",
            handle: "testuser",
            displayName: "Test User",
        },
        channelId: "channel-123",
        createdAt: new Date("2024-01-01T12:00:00Z"),
        updatedAt: new Date("2024-01-01T12:00:00Z"),
        reactions: [],
    }

    it("renders message content", () => {
        renderWithProviders(<MessageItem message={mockMessage} />)

        expect(screen.getByText(/Test message content/i)).toBeInTheDocument()
    })

    it("displays author information", () => {
        renderWithProviders(<MessageItem message={mockMessage} />)

        expect(screen.getByText("Test User")).toBeInTheDocument()
    })

    it("renders reactions", () => {
        const messageWithReactions: MessageListItem = {
            ...mockMessage,
            reactions: [
                { stampId: "stamp1", userId: "user1", stampCount: 3 },
            ],
        }

        renderWithProviders(<MessageItem message={messageWithReactions} />)

        expect(screen.getByText("3")).toBeInTheDocument()
    })

    it("renders without user information", () => {
        const messageWithoutUser: MessageListItem = {
            ...mockMessage,
            user: undefined,
        }

        renderWithProviders(<MessageItem message={messageWithoutUser} />)

        // Should still render the message content
        expect(screen.getByText(/Test message content/i)).toBeInTheDocument()
    })
})
