import { describe, it, expect, beforeEach } from "vitest"
import { MessageItem } from "./Message"
import { renderWithProviders, screen } from "../../test/utils"
import type { MessageListItem } from "../../api/twittra.schemas"
import { createMockMessage, createMockReaction } from "../../test/factories"

describe("MessageItem", () => {
    let mockMessage: MessageListItem

    beforeEach(() => {
        // Generate fresh random data for each test
        mockMessage = createMockMessage()
    })

    it("renders message content", () => {
        renderWithProviders(<MessageItem message={mockMessage} />)

        expect(screen.getByText(mockMessage.content)).toBeInTheDocument()
    })

    it("displays author information", () => {
        renderWithProviders(<MessageItem message={mockMessage} />)

        expect(screen.getByText(mockMessage.user!.displayName)).toBeInTheDocument()
    })

    it("renders reactions", () => {
        const reactionCount = 3
        const messageWithReactions = createMockMessage({
            reactions: [
                createMockReaction({ stampCount: reactionCount }),
            ],
        })

        renderWithProviders(<MessageItem message={messageWithReactions} />)

        expect(screen.getByText(reactionCount.toString())).toBeInTheDocument()
    })

    it("renders without user information", () => {
        const messageWithoutUser = createMockMessage({
            user: undefined,
        })

        renderWithProviders(<MessageItem message={messageWithoutUser} />)

        // Should still render the message content
        expect(screen.getByText(messageWithoutUser.content)).toBeInTheDocument()
    })
})
