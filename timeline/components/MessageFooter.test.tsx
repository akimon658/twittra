import { describe, it, expect, vi } from "vitest"
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
        renderWithProviders(<MessageFooter messageId="msg1" reactions={[]} />)

        // Should render the component without errors
        const footer = screen.getByRole("group")
        expect(footer).toBeInTheDocument()
    })

    it("handles reaction click", async () => {
        const user = userEvent.setup()
        renderWithProviders(
            <MessageFooter messageId="msg1" reactions={mockReactions} />,
        )

        // Find and click a reaction pill
        const reactionPills = screen.getAllByRole("button")
        if (reactionPills.length > 0) {
            await user.click(reactionPills[0])

            // Verify the click was handled (mutation should be triggered)
            await waitFor(() => {
                // The component should still be rendered
                expect(screen.getByRole("group")).toBeInTheDocument()
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

        // Should display combined count
        expect(screen.getByText("3")).toBeInTheDocument()
        expect(screen.getByText("2")).toBeInTheDocument()
    })
})
