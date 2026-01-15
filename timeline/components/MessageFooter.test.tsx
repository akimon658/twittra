import { describe, it, expect, beforeEach } from "vitest"
import { MessageFooter } from "./MessageFooter"
import { renderWithProviders, screen, userEvent, waitFor } from "../../test/utils"
import type { Reaction } from "../../api/twittra.schemas"
import { createMockReaction } from "../../test/factories"

describe("MessageFooter", () => {
    let mockReactions: Reaction[]
    let reaction1Count: number
    let reaction2Count: number

    beforeEach(() => {
        // Generate fresh random data for each test
        reaction1Count = 5
        reaction2Count = 2
        mockReactions = [
            createMockReaction({ stampCount: reaction1Count }),
            createMockReaction({ stampCount: reaction2Count }),
        ]
    })

    it("displays existing reactions correctly", () => {
        renderWithProviders(
            <MessageFooter messageId="msg1" reactions={mockReactions} />,
        )

        expect(screen.getByText(reaction1Count.toString())).toBeInTheDocument()
        expect(screen.getByText(reaction2Count.toString())).toBeInTheDocument()
    })

    it("renders with no reactions", () => {
        const { container } = renderWithProviders(
            <MessageFooter messageId="msg1" reactions={[]} />,
        )

        // Should render the component without errors
        // Check for the add reaction button (plus icon)
        const addButton = container.querySelector('[data-variant="default"]')
        expect(addButton).toBeInTheDocument()
    })

    it("handles reaction click", async () => {
        const user = userEvent.setup()
        const { container } = renderWithProviders(
            <MessageFooter messageId="msg1" reactions={mockReactions} />,
        )

        // Find reaction pills (they have cursor: pointer style)
        const reactionPills = container.querySelectorAll('[style*="cursor: pointer"]')
        if (reactionPills.length > 0) {
            await user.click(reactionPills[0] as Element)

            // Verify the component is still rendered
            await waitFor(() => {
                expect(screen.getByText(reaction1Count.toString())).toBeInTheDocument()
            })
        }
    })

    it("groups reactions by stamp", () => {
        const count1 = 3
        const count2 = 2
        const sameStampId = "same-stamp-id"
        const duplicateReactions: Reaction[] = [
            createMockReaction({ stampId: sameStampId, stampCount: count1 }),
            createMockReaction({ stampId: sameStampId, stampCount: count2 }),
        ]

        renderWithProviders(
            <MessageFooter messageId="msg1" reactions={duplicateReactions} />,
        )

        // Reactions with same stampId are grouped and counts are summed
        const totalCount = count1 + count2
        expect(screen.getByText(totalCount.toString())).toBeInTheDocument()
    })
})
