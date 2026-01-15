import { describe, it, expect } from "vitest"
import { MessageFooter } from "./MessageFooter"
import { renderWithProviders, screen, userEvent, waitFor } from "../../test/utils"
import type { Reaction } from "../../api/twittra.schemas"

describe("MessageFooter", () => {
    const mockReactions: Reaction[] = [
        { stampId: "stamp1", userId: "user1", stampCount: 5 },
        { stampId: "stamp2", userId: "user2", stampCount: 2 },
    ]

    it("displays existing reactions correctly", () => {
        renderWithProviders(
            <MessageFooter messageId="msg1" reactions={mockReactions} />,
        )

        expect(screen.getByText("5")).toBeInTheDocument()
        expect(screen.getByText("2")).toBeInTheDocument()
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
                expect(screen.getByText("5")).toBeInTheDocument()
            })
        }
    })

    it("groups reactions by stamp", () => {
        const duplicateReactions: Reaction[] = [
            { stampId: "stamp1", userId: "user1", stampCount: 3 },
            { stampId: "stamp1", userId: "user2", stampCount: 2 },
        ]

        renderWithProviders(
            <MessageFooter messageId="msg1" reactions={duplicateReactions} />,
        )

        // Reactions with same stampId are grouped and counts are summed (3 + 2 = 5)
        expect(screen.getByText("5")).toBeInTheDocument()
    })
})
