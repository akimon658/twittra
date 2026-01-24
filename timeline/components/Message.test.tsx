import { beforeEach, describe, expect, it, vi } from "vitest"
import type { MessageListItem } from "../../api/twittra.schemas.ts"
import { createMockMessage, createMockReaction } from "../../test/factories.ts"
import { renderWithProviders, screen } from "../../test/utils.tsx"
import { MessageItem } from "./Message.tsx"

describe("MessageItem", () => {
  let mockMessage: MessageListItem

  beforeEach(() => {
    // Generate fresh random data for each test
    mockMessage = createMockMessage()
  })

  it("renders message content", () => {
    renderWithProviders(
      <MessageItem message={mockMessage} onRead={vi.fn()} />,
    )

    expect(screen.getByText(mockMessage.content)).toBeInTheDocument()
  })

  it("displays author information", () => {
    renderWithProviders(
      <MessageItem message={mockMessage} onRead={vi.fn()} />,
    )

    expect(screen.getByText(mockMessage.user!.displayName)).toBeInTheDocument()
  })

  it("renders reactions", () => {
    const reactionCount = 3
    const messageWithReactions = createMockMessage({
      reactions: [
        createMockReaction({ stampCount: reactionCount }),
      ],
    })

    renderWithProviders(
      <MessageItem message={messageWithReactions} onRead={vi.fn()} />,
    )

    expect(screen.getByText(reactionCount.toString())).toBeInTheDocument()
  })

  it("renders without user information", () => {
    const messageWithoutUser = createMockMessage({
      user: undefined,
    })

    renderWithProviders(
      <MessageItem message={messageWithoutUser} onRead={vi.fn()} />,
    )

    // Should still render the message content
    expect(screen.getByText(messageWithoutUser.content)).toBeInTheDocument()
  })
})
